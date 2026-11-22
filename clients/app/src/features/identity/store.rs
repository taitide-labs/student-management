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

pub fn load_accounts(session: &AccountSession) {
    if session.role != Role::Admin {
        accounts_state().update(|s| s.accounts.clear());
        return;
    }
    if let Ok(rows) = gateway::client().with(|gw| gw.list_accounts(session)) {
        accounts_state().update(|s| s.accounts = rows);
    }
}

pub fn change_account_role(
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
        .with_mut(|gw| gw.change_account_role(session, account_id, &req))
        .map_err(|err| SharedString::from(err.to_string()))?;
    load_accounts(session);
    Ok(updated)
}

#[must_use]
pub fn auth_error() -> Option<SharedString> {
    state().with(|s| s.error.clone())
}

pub fn sign_in(username: &str, password: &str) -> Result<AccountSession, SharedString> {
    let req = SignInRequest {
        username: username.to_owned(),
        password: password.to_owned(),
    };
    let result = gateway::client().with(|gw| gw.sign_in(&req));
    apply_result(result)
}

pub fn quick_sign_in(role: Role) -> Result<AccountSession, SharedString> {
    let result = gateway::client().with(|gw| gw.switch_role(role));
    apply_result(result)
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
