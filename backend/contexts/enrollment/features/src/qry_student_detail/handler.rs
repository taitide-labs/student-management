use sqlx::Row;
use student_management_contracts::{
    EnrollmentChangeRecord, EnrollmentStatus, OperatorPrincipal, ProblemDetails, Role,
    StudentDetailPayload, StudentSummary,
};
use student_management_infra::{AppState, InfraError, infra_error_to_problem};

pub async fn execute(
    state: &AppState,
    operator: &OperatorPrincipal,
    student_no: &str,
) -> Result<StudentDetailPayload, ProblemDetails> {
    if operator.role == Role::Student && operator.bound_student_no.as_deref() != Some(student_no) {
        return Err(ProblemDetails::forbidden(
            "student_scope_forbidden",
            "仅允许查看本人绑定的学籍档案",
        ));
    }

    let stu_row = sqlx::query(
        "SELECT s.student_no, s.full_name, s.homeroom_id, h.name AS homeroom_name, \
         s.status, s.enrolled_on, s.version \
         FROM students s \
         JOIN homerooms h ON h.id = s.homeroom_id \
         WHERE s.student_no = ?",
    )
    .bind(student_no)
    .fetch_optional(&state.pool)
    .await
    .map_err(InfraError::Database)
    .map_err(|err| infra_error_to_problem(&err))?
    .ok_or_else(|| {
        ProblemDetails::not_found("student_not_found", format!("学生 {student_no} 不存在"))
    })?;

    let status_raw: String = stu_row
        .try_get("status")
        .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?;
    let version_raw: i64 = stu_row
        .try_get("version")
        .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?;

    let student = StudentSummary {
        student_no: stu_row
            .try_get("student_no")
            .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?,
        full_name: stu_row
            .try_get("full_name")
            .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?,
        homeroom_id: stu_row
            .try_get("homeroom_id")
            .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?,
        homeroom_name: stu_row
            .try_get("homeroom_name")
            .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?,
        status: parse_status(&status_raw)?,
        enrolled_on: stu_row
            .try_get("enrolled_on")
            .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?,
        version: u64::try_from(version_raw).unwrap_or(1),
    };

    let change_rows = sqlx::query(
        "SELECT id, student_no, action, from_summary, to_summary, reason, operator_name, occurred_at \
         FROM enrollment_change_records \
         WHERE student_no = ? ORDER BY occurred_at DESC, rowid DESC",
    )
    .bind(student_no)
    .fetch_all(&state.pool)
    .await
    .map_err(InfraError::Database)
    .map_err(|err| infra_error_to_problem(&err))?;

    let mut changes = Vec::with_capacity(change_rows.len());
    for row in change_rows {
        changes.push(EnrollmentChangeRecord {
            id: row
                .try_get("id")
                .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?,
            student_no: row
                .try_get("student_no")
                .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?,
            action: row
                .try_get("action")
                .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?,
            from_summary: row
                .try_get("from_summary")
                .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?,
            to_summary: row
                .try_get("to_summary")
                .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?,
            reason: row
                .try_get("reason")
                .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?,
            operator_name: row
                .try_get("operator_name")
                .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?,
            occurred_at: row
                .try_get("occurred_at")
                .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?,
        });
    }

    Ok(StudentDetailPayload { student, changes })
}

fn parse_status(raw: &str) -> Result<EnrollmentStatus, ProblemDetails> {
    match raw {
        "active" => Ok(EnrollmentStatus::Active),
        "suspended" => Ok(EnrollmentStatus::Suspended),
        "withdrawn" => Ok(EnrollmentStatus::Withdrawn),
        "graduated" => Ok(EnrollmentStatus::Graduated),
        _ => Err(ProblemDetails::bad_request(
            "invalid_status",
            format!("未知学籍状态：{raw}"),
        )),
    }
}
