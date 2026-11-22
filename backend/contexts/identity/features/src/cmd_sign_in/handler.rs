use sqlx::Row;
use student_management_contracts::{AccountSession, ProblemDetails, SignInRequest};
use student_management_infra::{
    AppState, InfraError, infra_error_to_problem, parse_role, verify_password,
};

use super::command::SignInCommand;

pub async fn execute(
    state: &AppState,
    req: SignInRequest,
) -> Result<AccountSession, ProblemDetails> {
    let cmd = SignInCommand::try_from(req)?;

    let row = sqlx::query(
        "SELECT account_id, username, display_name, password_hash, role, bound_student_no, status \
         FROM accounts WHERE username = ?",
    )
    .bind(cmd.username.as_str())
    .fetch_optional(&state.pool)
    .await
    .map_err(InfraError::Database)
    .map_err(|err| infra_error_to_problem(&err))?;

    let Some(row) = row else {
        return Err(ProblemDetails::unauthorized(
            "invalid_credentials",
            "账号或密码错误",
        ));
    };

    let status: String = row
        .try_get("status")
        .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?;
    if status != "active" {
        return Err(ProblemDetails::forbidden(
            "account_disabled",
            "该账号已被停用，请联系管理员",
        ));
    }

    let password_hash: String = row
        .try_get("password_hash")
        .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?;
    if !verify_password(cmd.password.as_str(), &password_hash) {
        return Err(ProblemDetails::unauthorized(
            "invalid_credentials",
            "账号或密码错误",
        ));
    }

    let account_id: String = row
        .try_get("account_id")
        .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?;
    let role_raw: String = row
        .try_get("role")
        .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?;
    let role = parse_role(&role_raw)?;
    let token = state.ids.next_id("tok");
    let now = state.clock.now_rfc3339();

    sqlx::query("INSERT INTO sessions (token, account_id, created_at) VALUES (?, ?, ?)")
        .bind(&token)
        .bind(&account_id)
        .bind(now)
        .execute(&state.pool)
        .await
        .map_err(InfraError::Database)
        .map_err(|err| infra_error_to_problem(&err))?;

    Ok(AccountSession {
        account_id,
        username: row
            .try_get("username")
            .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?,
        display_name: row
            .try_get("display_name")
            .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?,
        role,
        bound_student_no: row
            .try_get("bound_student_no")
            .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?,
        access_token: token,
    })
}
