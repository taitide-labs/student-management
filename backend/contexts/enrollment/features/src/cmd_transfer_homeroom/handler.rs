use std::num::NonZeroU32;

use sqlx::{Row, SqliteConnection};
use student_management_contracts::{
    EnrollmentStatus, OperatorPrincipal, ProblemDetails, Role, StudentSummary,
    TransferHomeroomRequest,
};
use student_management_enrollment_domain::{Homeroom, Student};
use student_management_infra::{
    AppState, IdempotencyOutcome, InfraError, check_idempotency, ensure_role, hash_payload,
    infra_error_to_problem, save_idempotency,
};
use student_management_primitives::{
    GradeYear, HomeroomName, NonEmptyText, PositiveVersion, StudentNo,
};

use super::{command::TransferHomeroomCommand, workflow};

pub async fn execute(
    state: &AppState,
    operator: &OperatorPrincipal,
    student_no: &str,
    req: TransferHomeroomRequest,
) -> Result<StudentSummary, ProblemDetails> {
    ensure_role(operator, &[Role::Admin])?;
    let payload_hash =
        hash_payload(&(student_no, &req)).map_err(|err| infra_error_to_problem(&err))?;
    let cmd = TransferHomeroomCommand::parse(student_no, &req)?;

    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(InfraError::Database)
        .map_err(|err| infra_error_to_problem(&err))?;

    if let IdempotencyOutcome::Replay(cached) = check_idempotency::<StudentSummary>(
        &mut tx,
        "enrollment::cmd_transfer_homeroom",
        cmd.idempotency_key.as_str(),
        &payload_hash,
    )
    .await
    .map_err(|err| infra_error_to_problem(&err))?
    {
        return Ok(cached);
    }

    let expected_ver = cmd.expected_version.get() as i64;
    let (mut student, from_homeroom_name) = load_student(&mut tx, cmd.student_no.as_str()).await?;
    let (target_homeroom, target_active) =
        load_homeroom(&mut tx, cmd.target_homeroom_id.as_str()).await?;
    let change = workflow::apply(&mut student, &target_homeroom, target_active, &cmd)?;

    let now = state.clock.now_rfc3339();
    let res = sqlx::query(
        "UPDATE students SET homeroom_id = ?, version = ?, updated_at = ? \
         WHERE student_no = ? AND version = ?",
    )
    .bind(&student.homeroom_id)
    .bind(student.version.get() as i64)
    .bind(&now)
    .bind(student.student_no.as_str())
    .bind(expected_ver)
    .execute(&mut *tx)
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
    .bind(state.ids.next_id("chg"))
    .bind(student.student_no.as_str())
    .bind(change.action)
    .bind(from_homeroom_name)
    .bind(target_homeroom.name.as_str())
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
        homeroom_id: target_homeroom.id,
        homeroom_name: target_homeroom.name.as_str().to_owned(),
        status: student.status,
        enrolled_on: student.enrolled_on,
        version: student.version.get(),
    };

    save_idempotency(
        &mut tx,
        "enrollment::cmd_transfer_homeroom",
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

async fn load_student(
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

    let status = match status_raw.as_str() {
        "active" => EnrollmentStatus::Active,
        "suspended" => EnrollmentStatus::Suspended,
        "withdrawn" => EnrollmentStatus::Withdrawn,
        "graduated" => EnrollmentStatus::Graduated,
        _ => {
            return Err(ProblemDetails::bad_request(
                "invalid_status",
                "学籍状态异常",
            ));
        }
    };
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

async fn load_homeroom(
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
    .ok_or_else(|| {
        ProblemDetails::not_found("homeroom_not_found", format!("行政班 {homeroom_id} 不存在"))
    })?;

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
        .ok()
        .and_then(|val| GradeYear::try_new(val).ok())
        .ok_or_else(|| ProblemDetails::bad_request("invalid_grade_year", "行政班年级异常"))?;
    let name = HomeroomName::try_new(&name_raw)
        .map_err(|err| ProblemDetails::bad_request("invalid_homeroom_name", err.to_string()))?;
    let capacity = capacity_raw
        .map(|cap| {
            u32::try_from(cap)
                .ok()
                .and_then(NonZeroU32::new)
                .ok_or_else(|| ProblemDetails::bad_request("invalid_capacity", "容量数据异常"))
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
