use std::num::NonZeroU32;

use sqlx::{Row, SqliteConnection};
use student_management_contracts::{
    EnrollStudentRequest, OperatorPrincipal, ProblemDetails, Role, StudentSummary,
};
use student_management_enrollment_domain::Homeroom;
use student_management_infra::{
    AppState, IdempotencyOutcome, InfraError, check_idempotency, ensure_role, hash_payload,
    infra_error_to_problem, save_idempotency,
};
use student_management_primitives::{GradeYear, HomeroomName, PositiveVersion, StudentNo};

use super::{command::EnrollStudentCommand, workflow};

pub async fn execute(
    state: &AppState,
    operator: &OperatorPrincipal,
    req: EnrollStudentRequest,
) -> Result<StudentSummary, ProblemDetails> {
    ensure_role(operator, &[Role::Admin])?;
    let payload_hash = hash_payload(&req).map_err(|err| infra_error_to_problem(&err))?;
    let cmd = EnrollStudentCommand::try_from(req)?;

    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(InfraError::Database)
        .map_err(|err| infra_error_to_problem(&err))?;

    if let IdempotencyOutcome::Replay(cached) = check_idempotency::<StudentSummary>(
        &mut tx,
        "enrollment::cmd_enroll_student",
        cmd.idempotency_key.as_str(),
        &payload_hash,
    )
    .await
    .map_err(|err| infra_error_to_problem(&err))?
    {
        return Ok(cached);
    }

    let (homeroom, active_count) =
        load_homeroom_with_occupancy(&mut tx, cmd.homeroom_id.as_str()).await?;
    let seq = allocate_student_sequence(&mut tx, homeroom.grade_year).await?;
    let student_no = StudentNo::from_sequence(homeroom.grade_year, seq)
        .map_err(|err| ProblemDetails::conflict("student_no_exhausted", err.to_string()))?;

    let today = state.clock.today_iso();
    let now = state.clock.now_rfc3339();
    let (student, change) = workflow::enroll(&homeroom, active_count, student_no, &cmd, today)?;

    sqlx::query(
        "INSERT INTO students (student_no, full_name, homeroom_id, status, enrolled_on, version, updated_at) VALUES (?, ?, ?, 'active', ?, ?, ?)",
    )
    .bind(student.student_no.as_str())
    .bind(student.full_name.as_str())
    .bind(&student.homeroom_id)
    .bind(&student.enrolled_on)
    .bind(student.version.get() as i64)
    .bind(&now)
    .execute(&mut *tx)
    .await
    .map_err(InfraError::Database)
    .map_err(|err| infra_error_to_problem(&err))?;

    sqlx::query(
        "INSERT INTO enrollment_change_records (id, student_no, action, from_summary, to_summary, reason, operator_name, occurred_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(state.ids.next_id("chg"))
    .bind(student.student_no.as_str())
    .bind(change.action)
    .bind(change.from_summary)
    .bind(change.to_summary)
    .bind(change.reason)
    .bind(&operator.display_name)
    .bind(&now)
    .execute(&mut *tx)
    .await
    .map_err(InfraError::Database)
    .map_err(|err| infra_error_to_problem(&err))?;

    let summary = StudentSummary {
        student_no: student.student_no.as_str().to_owned(),
        full_name: student.full_name.as_str().to_owned(),
        homeroom_id: homeroom.id,
        homeroom_name: homeroom.name.as_str().to_owned(),
        status: student.status,
        enrolled_on: student.enrolled_on,
        version: student.version.get(),
    };

    save_idempotency(
        &mut tx,
        "enrollment::cmd_enroll_student",
        cmd.idempotency_key.as_str(),
        &payload_hash,
        &summary,
        &now,
    )
    .await
    .map_err(|err| infra_error_to_problem(&err))?;

    tx.commit()
        .await
        .map_err(InfraError::Database)
        .map_err(|err| infra_error_to_problem(&err))?;
    Ok(summary)
}

async fn load_homeroom_with_occupancy(
    conn: &mut SqliteConnection,
    homeroom_id: &str,
) -> Result<(Homeroom, u32), ProblemDetails> {
    let row = sqlx::query(
        "SELECT h.id, h.name, h.grade_year, h.capacity, h.version, \
         (SELECT COUNT(*) FROM students s WHERE s.homeroom_id = h.id AND s.status = 'active') AS active_count \
         FROM homerooms h WHERE h.id = ?",
    )
    .bind(homeroom_id)
    .fetch_optional(conn)
    .await
    .map_err(InfraError::Database)
    .map_err(|err| infra_error_to_problem(&err))?
    .ok_or_else(|| ProblemDetails::not_found("homeroom_not_found", format!("行政班 {homeroom_id} 不存在")))?;

    let grade_year_raw: i64 = row
        .try_get("grade_year")
        .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?;
    let capacity_raw: Option<i64> = row
        .try_get("capacity")
        .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?;
    let version_raw: i64 = row
        .try_get("version")
        .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?;
    let active_raw: i64 = row
        .try_get("active_count")
        .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?;
    let name_raw: String = row
        .try_get("name")
        .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?;
    let id: String = row
        .try_get("id")
        .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?;

    let grade_year = u16::try_from(grade_year_raw)
        .map_err(|_| ProblemDetails::bad_request("invalid_grade_year", "行政班年级数值溢出"))
        .and_then(|val| {
            GradeYear::try_new(val)
                .map_err(|err| ProblemDetails::bad_request("invalid_grade_year", err.to_string()))
        })?;
    let name = HomeroomName::try_new(&name_raw)
        .map_err(|err| ProblemDetails::bad_request("invalid_homeroom_name", err.to_string()))?;
    let capacity = capacity_raw
        .map(|cap| {
            u32::try_from(cap)
                .ok()
                .and_then(NonZeroU32::new)
                .ok_or_else(|| {
                    ProblemDetails::bad_request("invalid_capacity", "行政班容量数据异常")
                })
        })
        .transpose()?;
    let version = u64::try_from(version_raw)
        .ok()
        .and_then(|val| PositiveVersion::try_new(val).ok())
        .ok_or_else(|| ProblemDetails::bad_request("invalid_version", "行政班版本号异常"))?;
    let active_count = u32::try_from(active_raw)
        .map_err(|_| ProblemDetails::bad_request("invalid_active_count", "在籍人数计数异常"))?;

    Ok((
        Homeroom {
            id,
            name,
            grade_year,
            capacity,
            version,
        },
        active_count,
    ))
}

async fn allocate_student_sequence(
    conn: &mut SqliteConnection,
    grade_year: GradeYear,
) -> Result<u32, ProblemDetails> {
    let row = sqlx::query(
        "INSERT INTO student_no_sequences (grade_year, last_seq) VALUES (?, 1) \
         ON CONFLICT(grade_year) DO UPDATE SET last_seq = last_seq + 1 \
         RETURNING last_seq",
    )
    .bind(i64::from(grade_year.get()))
    .fetch_one(conn)
    .await
    .map_err(InfraError::Database)
    .map_err(|err| infra_error_to_problem(&err))?;

    let seq_raw: i64 = row
        .try_get("last_seq")
        .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?;
    u32::try_from(seq_raw)
        .map_err(|_| ProblemDetails::conflict("student_no_exhausted", "当前年级学号序号溢出"))
}
