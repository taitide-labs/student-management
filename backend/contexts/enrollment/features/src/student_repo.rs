use sqlx::{Row, SqliteConnection};
use student_management_contracts::{
    EnrollmentStatus, IntegrationEvent, OperatorPrincipal, ProblemDetails, StudentSummary,
};
use student_management_enrollment_domain::{EnrollmentChangeDraft, Student};
use student_management_infra::{
    AppState, InfraError, enqueue_event, infra_error_to_problem, save_idempotency,
};
use student_management_primitives::{NonEmptyText, PositiveVersion, StudentNo};

pub struct StatusPersistCtx<'a> {
    pub state: &'a AppState,
    pub operator: &'a OperatorPrincipal,
    pub student: &'a Student,
    pub homeroom_name: &'a str,
    pub expected_version: i64,
    pub change: EnrollmentChangeDraft,
    pub event: Option<IntegrationEvent>,
    pub scope: &'a str,
    pub idempotency_key: &'a str,
    pub payload_hash: &'a str,
    pub now: &'a str,
}

pub async fn load_student(
    conn: &mut SqliteConnection,
    student_no: &str,
) -> Result<(Student, String), ProblemDetails> {
    let row = sqlx::query(
        "SELECT s.student_no, s.full_name, s.homeroom_id, h.name AS homeroom_name, \
         s.status, s.enrolled_on, s.version \
         FROM students s JOIN homerooms h ON h.id = s.homeroom_id WHERE s.student_no = ?",
    )
    .bind(student_no)
    .fetch_optional(conn)
    .await
    .map_err(InfraError::Database)
    .map_err(|err| infra_error_to_problem(&err))?
    .ok_or_else(|| {
        ProblemDetails::not_found("student_not_found", format!("学生 {student_no} 不存在"))
    })?;

    let status_raw: String = row
        .try_get("status")
        .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?;
    let version_raw: i64 = row
        .try_get("version")
        .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?;
    let full_name_raw: String = row
        .try_get("full_name")
        .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?;
    let homeroom_name: String = row
        .try_get("homeroom_name")
        .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?;

    let status = parse_status(&status_raw)?;
    let version = u64::try_from(version_raw)
        .ok()
        .and_then(|val| PositiveVersion::try_new(val).ok())
        .ok_or_else(|| ProblemDetails::bad_request("invalid_version", "学籍版本号异常"))?;

    Ok((
        Student {
            student_no: StudentNo::try_new(student_no).map_err(|err| {
                ProblemDetails::bad_request("invalid_student_no", err.to_string())
            })?,
            full_name: NonEmptyText::try_new("学生姓名", &full_name_raw, 64)
                .map_err(|err| ProblemDetails::bad_request("invalid_full_name", err.to_string()))?,
            homeroom_id: row
                .try_get("homeroom_id")
                .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?,
            status,
            enrolled_on: row
                .try_get("enrolled_on")
                .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?,
            version,
        },
        homeroom_name,
    ))
}

pub async fn persist_status_change(
    conn: &mut SqliteConnection,
    ctx: StatusPersistCtx<'_>,
) -> Result<StudentSummary, ProblemDetails> {
    let res = sqlx::query(
        "UPDATE students SET status = ?, version = ?, updated_at = ? \
         WHERE student_no = ? AND version = ?",
    )
    .bind(status_to_db(ctx.student.status))
    .bind(ctx.student.version.get() as i64)
    .bind(ctx.now)
    .bind(ctx.student.student_no.as_str())
    .bind(ctx.expected_version)
    .execute(&mut *conn)
    .await
    .map_err(InfraError::Database)
    .map_err(|err| infra_error_to_problem(&err))?;

    if res.rows_affected() == 0 {
        return Err(ProblemDetails::conflict(
            "version_conflict",
            "学生档案版本号已变更",
        ));
    }

    sqlx::query(
        "INSERT INTO enrollment_change_records \
         (id, student_no, action, from_summary, to_summary, reason, operator_name, occurred_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(ctx.state.ids.next_id("chg"))
    .bind(ctx.student.student_no.as_str())
    .bind(ctx.change.action)
    .bind(ctx.change.from_summary)
    .bind(ctx.change.to_summary)
    .bind(ctx.change.reason)
    .bind(&ctx.operator.display_name)
    .bind(ctx.now)
    .execute(&mut *conn)
    .await
    .map_err(InfraError::Database)
    .map_err(|err| infra_error_to_problem(&err))?;

    if let Some(event) = ctx.event {
        enqueue_event(conn, &event)
            .await
            .map_err(|err| infra_error_to_problem(&err))?;
    }

    let summary = StudentSummary {
        student_no: ctx.student.student_no.as_str().to_owned(),
        full_name: ctx.student.full_name.as_str().to_owned(),
        homeroom_id: ctx.student.homeroom_id.clone(),
        homeroom_name: ctx.homeroom_name.to_owned(),
        status: ctx.student.status,
        enrolled_on: ctx.student.enrolled_on.clone(),
        version: ctx.student.version.get(),
    };

    save_idempotency(
        conn,
        ctx.scope,
        ctx.idempotency_key,
        ctx.payload_hash,
        &summary,
        ctx.now,
    )
    .await
    .map_err(|err| infra_error_to_problem(&err))?;

    Ok(summary)
}

const fn status_to_db(status: EnrollmentStatus) -> &'static str {
    match status {
        EnrollmentStatus::Active => "active",
        EnrollmentStatus::Suspended => "suspended",
        EnrollmentStatus::Withdrawn => "withdrawn",
        EnrollmentStatus::Graduated => "graduated",
    }
}

fn parse_status(raw: &str) -> Result<EnrollmentStatus, ProblemDetails> {
    match raw {
        "active" => Ok(EnrollmentStatus::Active),
        "suspended" => Ok(EnrollmentStatus::Suspended),
        "withdrawn" => Ok(EnrollmentStatus::Withdrawn),
        "graduated" => Ok(EnrollmentStatus::Graduated),
        _ => Err(ProblemDetails::bad_request(
            "invalid_status",
            "学籍状态异常",
        )),
    }
}
