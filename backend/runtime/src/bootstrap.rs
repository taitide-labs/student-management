use std::sync::Arc;

use sqlx::Row;
use student_management_contracts::{AccountSession, ProblemDetails};
use student_management_enrollment_features::port::SqliteEnrollmentQueryPort;
use student_management_identity_features::port::SqliteIdentityLookupPort;
use student_management_infra::{
    AppState, InfraError, SystemClock, UuidGenerator, authenticate_bearer, dispatch_pending,
    hash_password, infra_error_to_problem, open_sqlite_pool,
};

pub fn link_all_contexts() {
    student_management_identity_features::link();
    student_management_enrollment_features::link();
    student_management_grading_features::link();
}

pub async fn build_app_state(database_url: &str) -> Result<AppState, InfraError> {
    link_all_contexts();
    let pool = open_sqlite_pool(database_url).await?;
    let identity_port = Arc::new(SqliteIdentityLookupPort::new(pool.clone()));
    let enrollment_port = Arc::new(SqliteEnrollmentQueryPort::new(pool.clone()));
    Ok(
        AppState::new(pool, Arc::new(SystemClock), Arc::new(UuidGenerator))
            .with_ports(identity_port, enrollment_port),
    )
}

pub async fn open_app_state(database_url: &str) -> Result<AppState, ProblemDetails> {
    build_app_state(database_url)
        .await
        .map_err(|err| infra_error_to_problem(&err))
}

pub async fn ensure_bootstrap_admin(
    state: &AppState,
    username: &str,
    display_name: &str,
    password: &str,
) -> Result<(), ProblemDetails> {
    let count_row = sqlx::query("SELECT COUNT(*) AS cnt FROM accounts")
        .fetch_one(&state.pool)
        .await
        .map_err(InfraError::Database)
        .map_err(|err| infra_error_to_problem(&err))?;
    let count: i64 = count_row
        .try_get("cnt")
        .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?;
    if count > 0 {
        return Ok(());
    }

    let hash = hash_password(password).map_err(|err| infra_error_to_problem(&err))?;
    let now = state.clock.now_rfc3339();
    let account_id = state.ids.next_id("acc");
    sqlx::query(
        "INSERT INTO accounts (account_id, username, display_name, password_hash, role, status, version, created_at, updated_at) \
         VALUES (?, ?, ?, ?, 'admin', 'active', 1, ?, ?)",
    )
    .bind(account_id)
    .bind(username)
    .bind(display_name)
    .bind(hash)
    .bind(&now)
    .bind(&now)
    .execute(&state.pool)
    .await
    .map_err(InfraError::Database)
    .map_err(|err| infra_error_to_problem(&err))?;
    Ok(())
}

pub async fn resolve_session(
    state: &AppState,
    token: &str,
) -> Result<AccountSession, ProblemDetails> {
    let header = format!("Bearer {token}");
    let principal = authenticate_bearer(&state.pool, Some(&header)).await?;
    let username: String = sqlx::query("SELECT username FROM accounts WHERE account_id = ?")
        .bind(&principal.account_id)
        .fetch_one(&state.pool)
        .await
        .map_err(InfraError::Database)
        .map_err(|err| infra_error_to_problem(&err))?
        .try_get("username")
        .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?;
    Ok(AccountSession {
        account_id: principal.account_id,
        username,
        display_name: principal.display_name,
        role: principal.role,
        bound_student_no: principal.bound_student_no,
        access_token: token.to_owned(),
    })
}

pub async fn dispatch_outbox(state: &AppState) -> Result<usize, ProblemDetails> {
    dispatch_pending(&state.pool, &state.clock.now_rfc3339())
        .await
        .map_err(|err| infra_error_to_problem(&err))
}
