use rust_decimal::Decimal;
use sqlx::Row;
use student_management_contracts::{CourseSummary, OperatorPrincipal, ProblemDetails, Role};
use student_management_infra::{AppState, InfraError, ensure_role, infra_error_to_problem};

pub async fn execute(
    state: &AppState,
    operator: &OperatorPrincipal,
) -> Result<Vec<CourseSummary>, ProblemDetails> {
    ensure_role(operator, &[Role::Admin, Role::Teacher, Role::Student])?;

    let rows = sqlx::query(
        "SELECT id, code, name, max_score, pass_score, version FROM courses ORDER BY code ASC",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(InfraError::Database)
    .map_err(|err| infra_error_to_problem(&err))?;

    let mut courses = Vec::with_capacity(rows.len());
    for row in rows {
        let max_raw: String = row
            .try_get("max_score")
            .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?;
        let pass_raw: String = row
            .try_get("pass_score")
            .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?;
        let version_raw: i64 = row
            .try_get("version")
            .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?;

        let max_score = max_raw
            .parse::<Decimal>()
            .map_err(|_| ProblemDetails::bad_request("invalid_score", "课程满分数据异常"))?;
        let pass_score = pass_raw
            .parse::<Decimal>()
            .map_err(|_| ProblemDetails::bad_request("invalid_score", "课程及格线数据异常"))?;

        courses.push(CourseSummary {
            id: row
                .try_get("id")
                .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?,
            code: row
                .try_get("code")
                .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?,
            name: row
                .try_get("name")
                .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?,
            max_score,
            pass_score,
            version: u64::try_from(version_raw).unwrap_or(1),
        });
    }

    Ok(courses)
}
