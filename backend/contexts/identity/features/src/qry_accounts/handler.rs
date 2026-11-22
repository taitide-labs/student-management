use sqlx::Row;
use student_management_contracts::{
    AccountStatus, AccountSummary, OperatorPrincipal, ProblemDetails, Role,
};
use student_management_infra::{
    AppState, InfraError, ensure_role, infra_error_to_problem, parse_role,
};

pub async fn execute(
    state: &AppState,
    operator: &OperatorPrincipal,
) -> Result<Vec<AccountSummary>, ProblemDetails> {
    ensure_role(operator, &[Role::Admin])?;

    let rows = sqlx::query(
        "SELECT account_id, username, display_name, role, bound_student_no, status, version \
         FROM accounts ORDER BY username ASC",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(InfraError::Database)
    .map_err(|err| infra_error_to_problem(&err))?;

    let mut items = Vec::with_capacity(rows.len());
    for row in rows {
        let role_raw: String = row
            .try_get("role")
            .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?;
        let status_raw: String = row
            .try_get("status")
            .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?;
        let version_raw: i64 = row
            .try_get("version")
            .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?;

        items.push(AccountSummary {
            account_id: row
                .try_get("account_id")
                .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?,
            username: row
                .try_get("username")
                .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?,
            display_name: row
                .try_get("display_name")
                .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?,
            role: parse_role(&role_raw)?,
            bound_student_no: row
                .try_get("bound_student_no")
                .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?,
            status: parse_account_status(&status_raw)?,
            version: u64::try_from(version_raw).unwrap_or(1),
        });
    }

    Ok(items)
}

fn parse_account_status(raw: &str) -> Result<AccountStatus, ProblemDetails> {
    match raw {
        "active" => Ok(AccountStatus::Active),
        "disabled" => Ok(AccountStatus::Disabled),
        _ => Err(ProblemDetails::bad_request(
            "invalid_account_status",
            format!("未知账号状态：{raw}"),
        )),
    }
}
