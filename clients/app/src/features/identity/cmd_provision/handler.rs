use nsh_core::prelude::SharedString;
use student_management_gateway::contract::{AccountSession, AccountSummary, Role};

use crate::gateway;

use super::super::store::{load_accounts, next_key};
use super::command;

pub fn execute(
    session: &AccountSession,
    username: &str,
    display_name: &str,
    role: Role,
    student_no_raw: &str,
) -> Result<AccountSummary, SharedString> {
    let req = command::build_request(
        username,
        display_name,
        role,
        student_no_raw,
        next_key("acc-prov"),
    )
    .map_err(SharedString::from)?;
    let created = gateway::client()
        .with_mut(|gw| gw.provision_account(session, &req))
        .map_err(|err| SharedString::from(err.to_string()))?;
    load_accounts(session);
    Ok(created)
}
