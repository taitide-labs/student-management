use sqlx::Row;
use student_management_contracts::{
    EnrollmentStatus, OperatorPrincipal, ProblemDetails, Role, StudentSummary,
};
use student_management_infra::{AppState, InfraError, infra_error_to_problem};

pub async fn execute(
    state: &AppState,
    operator: &OperatorPrincipal,
    query: &str,
    status_filter: Option<EnrollmentStatus>,
) -> Result<Vec<StudentSummary>, ProblemDetails> {
    let trimmed = query.trim().to_owned();
    let status_param = status_filter.map(status_to_db);
    let student_scope = match operator.role {
        Role::Student => match &operator.bound_student_no {
            Some(no) => Some(no.as_str()),
            None => return Ok(Vec::new()),
        },
        Role::Admin | Role::Teacher => None,
    };

    let like_pattern = format!("%{trimmed}%");
    let rows = sqlx::query(
        "SELECT s.student_no, s.full_name, s.homeroom_id, h.name AS homeroom_name, \
         s.status, s.enrolled_on, s.version \
         FROM students s \
         JOIN homerooms h ON h.id = s.homeroom_id \
         WHERE (? = '' OR s.student_no LIKE ? OR s.full_name LIKE ?) \
         AND (? IS NULL OR s.status = ?) \
         AND (? IS NULL OR s.student_no = ?) \
         ORDER BY s.student_no ASC",
    )
    .bind(&trimmed)
    .bind(&like_pattern)
    .bind(&like_pattern)
    .bind(status_param)
    .bind(status_param)
    .bind(student_scope)
    .bind(student_scope)
    .fetch_all(&state.pool)
    .await
    .map_err(InfraError::Database)
    .map_err(|err| infra_error_to_problem(&err))?;

    let mut items = Vec::with_capacity(rows.len());
    for row in rows {
        let status_raw: String = row
            .try_get("status")
            .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?;
        let version_raw: i64 = row
            .try_get("version")
            .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?;
        items.push(StudentSummary {
            student_no: row
                .try_get("student_no")
                .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?,
            full_name: row
                .try_get("full_name")
                .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?,
            homeroom_id: row
                .try_get("homeroom_id")
                .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?,
            homeroom_name: row
                .try_get("homeroom_name")
                .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?,
            status: parse_status(&status_raw)?,
            enrolled_on: row
                .try_get("enrolled_on")
                .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?,
            version: u64::try_from(version_raw).unwrap_or(1),
        });
    }

    Ok(items)
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
            format!("未知学籍状态：{raw}"),
        )),
    }
}
