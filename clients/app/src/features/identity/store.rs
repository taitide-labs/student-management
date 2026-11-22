use nsh_core::prelude::*;
use student_management_gateway::contract::{
    AccountSession, AccountSummary, ChangeAccountRoleRequest, Role, SignInRequest,
};

use crate::gateway;

#[derive(Clone, Default)]
struct AuthState {
    session: Option<AccountSession>,
    error: Option<SharedString>,
}

#[derive(Clone, Default)]
struct AccountsState {
    accounts: Vec<AccountSummary>,
    op_seq: u64,
}

pub fn next_key(prefix: &str) -> String {
    let mut seq = 0_u64;
    accounts_state().update(|s| {
        s.op_seq += 1;
        seq = s.op_seq;
    });
    format!("{prefix}-{seq}")
}

fn state() -> Signal<AuthState> {
    global::<AuthState>()
}

fn accounts_state() -> Signal<AccountsState> {
    global::<AccountsState>()
}

#[must_use]
pub fn current_session() -> Option<AccountSession> {
    state().with(|s| s.session.clone())
}

#[must_use]
pub fn accounts() -> Vec<AccountSummary> {
    accounts_state().with(|s| s.accounts.clone())
}

pub async fn refresh_accounts(session: &AccountSession) {
    if session.role != Role::Admin {
        accounts_state().update(|s| s.accounts.clear());
        return;
    }
    if let Ok(rows) = gateway::client().list_accounts(session).await {
        accounts_state().update(|s| s.accounts = rows);
    }
}

pub fn load_accounts(session: &AccountSession) {
    let active = session.clone();
    spawn(async move {
        refresh_accounts(&active).await;
    });
}

pub async fn change_account_role(
    session: &AccountSession,
    account_id: &str,
    role: Role,
    version: u64,
) -> Result<AccountSummary, SharedString> {
    let req = ChangeAccountRoleRequest {
        role,
        version,
        idempotency_key: next_key("acc-role"),
    };
    let updated = gateway::client()
        .change_account_role(session, account_id, &req)
        .await
        .map_err(|err| SharedString::from(err.to_string()))?;
    refresh_accounts(session).await;
    Ok(updated)
}

#[must_use]
pub fn auth_error() -> Option<SharedString> {
    state().with(|s| s.error.clone())
}

pub async fn sign_in(username: &str, password: &str) {
    let req = SignInRequest {
        username: username.to_owned(),
        password: password.to_owned(),
    };
    let result = gateway::client().sign_in(&req).await;
    drop(apply_result(result));
}

fn apply_result(
    result: Result<AccountSession, student_management_gateway::contract::GatewayError>,
) -> Result<AccountSession, SharedString> {
    match result {
        Ok(session) => {
            state().update(|s| {
                s.session = Some(session.clone());
                s.error = None;
            });
            Ok(session)
        }
        Err(err) => {
            let msg = SharedString::from(err.to_string());
            state().update(|s| s.error = Some(msg.clone()));
            Err(msg)
        }
    }
}

pub fn sign_out() {
    state().update(|s| {
        s.session = None;
        s.error = None;
    });
    accounts_state().update(|s| s.accounts.clear());
}
