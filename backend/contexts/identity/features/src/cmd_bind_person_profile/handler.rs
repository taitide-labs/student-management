use sqlx::{Row, SqliteConnection};
use student_management_contracts::{
    AccountStatus, AccountSummary, BindPersonProfileRequest, OperatorPrincipal, ProblemDetails,
    Role,
};
use student_management_identity_domain::Account;
use student_management_infra::{
    AppState, IdempotencyOutcome, InfraError, check_idempotency, ensure_role, hash_payload,
    infra_error_to_problem, parse_role, save_idempotency,
};
use student_management_primitives::{NonEmptyText, PositiveVersion, StudentNo, Username};

use super::{command::BindPersonProfileCommand, workflow};

pub async fn execute(
    state: &AppState,
    operator: &OperatorPrincipal,
    account_id: &str,
    req: BindPersonProfileRequest,
) -> Result<AccountSummary, ProblemDetails> {
    ensure_role(operator, &[Role::Admin])?;
    let payload_hash =
        hash_payload(&(account_id, &req)).map_err(|err| infra_error_to_problem(&err))?;
    let cmd = BindPersonProfileCommand::parse(account_id, &req)?;

    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(InfraError::Database)
        .map_err(|err| infra_error_to_problem(&err))?;

    if let IdempotencyOutcome::Replay(cached) = check_idempotency::<AccountSummary>(
        &mut tx,
        "identity::cmd_bind_person_profile",
        cmd.idempotency_key.as_str(),
        &payload_hash,
    )
    .await
    .map_err(|err| infra_error_to_problem(&err))?
    {
        return Ok(cached);
    }

    let idempotency_key = cmd.idempotency_key.as_str().to_owned();
    let expected_ver = cmd.expected_version.get() as i64;
    let mut account = load_account(&mut tx, cmd.account_id.as_str()).await?;
    workflow::apply(&mut account, cmd)?;

    let now = state.clock.now_rfc3339();
    let res = sqlx::query(
        "UPDATE accounts SET bound_student_no = ?, version = ?, updated_at = ? \
         WHERE account_id = ? AND version = ?",
    )
    .bind(account.bound_student_no.as_ref().map(StudentNo::as_str))
    .bind(account.version.get() as i64)
    .bind(&now)
    .bind(&account.account_id)
    .bind(expected_ver)
    .execute(&mut *tx)
    .await;

    match res {
        Ok(done) if done.rows_affected() == 0 => {
            return Err(ProblemDetails::conflict(
                "version_conflict",
                "账号版本号已变更",
            ));
        }
        Ok(_) => {}
        Err(err) => return Err(map_update_error(err)),
    }

    let summary = to_summary(account);
    save_idempotency(
        &mut tx,
        "identity::cmd_bind_person_profile",
        &idempotency_key,
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

async fn load_account(
    conn: &mut SqliteConnection,
    account_id: &str,
) -> Result<Account, ProblemDetails> {
    let row = sqlx::query(
        "SELECT account_id, username, display_name, role, bound_student_no, status, version \
         FROM accounts WHERE account_id = ?",
    )
    .bind(account_id)
    .fetch_optional(conn)
    .await
    .map_err(InfraError::Database)
    .map_err(|err| infra_error_to_problem(&err))?
    .ok_or_else(|| {
        ProblemDetails::not_found("account_not_found", format!("账号 {account_id} 不存在"))
    })?;

    let role_raw: String = row
        .try_get("role")
        .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?;
    let status_raw: String = row
        .try_get("status")
        .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?;
    let username_raw: String = row
        .try_get("username")
        .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?;
    let display_raw: String = row
        .try_get("display_name")
        .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?;
    let bound_raw: Option<String> = row
        .try_get("bound_student_no")
        .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?;
    let version_raw: i64 = row
        .try_get("version")
        .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?;

    let status = match status_raw.as_str() {
        "active" => AccountStatus::Active,
        "disabled" => AccountStatus::Disabled,
        _ => {
            return Err(ProblemDetails::bad_request(
                "invalid_status",
                "账号状态异常",
            ));
        }
    };
    let bound_student_no = bound_raw
        .as_deref()
        .map(StudentNo::try_new)
        .transpose()
        .map_err(|err| ProblemDetails::bad_request("invalid_student_no", err.to_string()))?;
    let version = u64::try_from(version_raw)
        .ok()
        .and_then(|val| PositiveVersion::try_new(val).ok())
        .ok_or_else(|| ProblemDetails::bad_request("invalid_version", "账号版本号异常"))?;

    Ok(Account {
        account_id: row
            .try_get("account_id")
            .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?,
        username: Username::try_new(&username_raw)
            .map_err(|err| ProblemDetails::bad_request("invalid_username", err.to_string()))?,
        display_name: NonEmptyText::try_new("显示名称", &display_raw, 64)
            .map_err(|err| ProblemDetails::bad_request("invalid_display_name", err.to_string()))?,
        role: parse_role(&role_raw)?,
        bound_student_no,
        status,
        disable_reason: None,
        version,
    })
}

fn to_summary(account: Account) -> AccountSummary {
    AccountSummary {
        account_id: account.account_id,
        username: account.username.as_str().to_owned(),
        display_name: account.display_name.as_str().to_owned(),
        role: account.role,
        bound_student_no: account
            .bound_student_no
            .as_ref()
            .map(|no| no.as_str().to_owned()),
        status: account.status,
        version: account.version.get(),
    }
}

fn map_update_error(err: sqlx::Error) -> ProblemDetails {
    if let sqlx::Error::Database(db_err) = &err
        && db_err.is_unique_violation()
    {
        return ProblemDetails::conflict("student_already_bound", "该学号已绑定至其他账号");
    }
    infra_error_to_problem(&InfraError::Database(err))
}
