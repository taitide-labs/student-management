use student_management_contracts::{
    AccountSummary, OperatorPrincipal, ProblemDetails, ProvisionAccountRequest, Role,
};
use student_management_infra::{
    AppState, IdempotencyOutcome, InfraError, check_idempotency, ensure_role,
    generate_random_secret, hash_password, hash_payload, infra_error_to_problem, save_idempotency,
};

use super::{command::ProvisionAccountCommand, workflow};

pub async fn execute(
    state: &AppState,
    operator: &OperatorPrincipal,
    req: ProvisionAccountRequest,
) -> Result<AccountSummary, ProblemDetails> {
    ensure_role(operator, &[Role::Admin])?;
    let payload_hash = hash_payload(&req).map_err(|err| infra_error_to_problem(&err))?;
    let cmd = ProvisionAccountCommand::try_from(req)?;

    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(InfraError::Database)
        .map_err(|err| infra_error_to_problem(&err))?;

    if let IdempotencyOutcome::Replay(cached) = check_idempotency::<AccountSummary>(
        &mut tx,
        "identity::cmd_provision_account",
        cmd.idempotency_key.as_str(),
        &payload_hash,
    )
    .await
    .map_err(|err| infra_error_to_problem(&err))?
    {
        return Ok(cached);
    }

    let idempotency_key = cmd.idempotency_key.as_str().to_owned();
    let account = workflow::prepare(state.ids.next_id("acc"), cmd)?;
    let initial_secret = generate_random_secret();
    let password_hash =
        hash_password(&initial_secret).map_err(|err| infra_error_to_problem(&err))?;
    let now = state.clock.now_rfc3339();
    let role_str = role_to_db(account.role);

    if let Err(err) = sqlx::query(
        "INSERT INTO accounts (account_id, username, display_name, password_hash, role, bound_student_no, status, version, created_at, updated_at) \
         VALUES (?, ?, ?, ?, ?, ?, 'active', ?, ?, ?)",
    )
    .bind(&account.account_id)
    .bind(account.username.as_str())
    .bind(account.display_name.as_str())
    .bind(password_hash)
    .bind(role_str)
    .bind(account.bound_student_no.as_ref().map(|no| no.as_str().to_owned()))
    .bind(account.version.get() as i64)
    .bind(&now)
    .bind(&now)
    .execute(&mut *tx)
    .await
    {
        return Err(map_insert_error(err));
    }

    let summary = AccountSummary {
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
    };

    save_idempotency(
        &mut tx,
        "identity::cmd_provision_account",
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

const fn role_to_db(role: Role) -> &'static str {
    match role {
        Role::Admin => "admin",
        Role::Teacher => "teacher",
        Role::Student => "student",
    }
}

fn map_insert_error(err: sqlx::Error) -> ProblemDetails {
    if let sqlx::Error::Database(db_err) = &err
        && db_err.is_unique_violation()
    {
        return ProblemDetails::conflict("account_duplicate", "登录账号或绑定学号已存在");
    }
    infra_error_to_problem(&InfraError::Database(err))
}
