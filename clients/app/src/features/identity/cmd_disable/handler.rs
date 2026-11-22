use nsh_core::prelude::SharedString;
use student_management_gateway::contract::{AccountSession, AccountSummary};

use crate::gateway;

use super::super::store::{load_accounts, next_key};
use super::command;

pub fn execute(
    session: &AccountSession,
    account_id: &str,
    reason: &str,
    version: u64,
) -> Result<AccountSummary, SharedString> {
    let req =
        command::build_request(reason, version, next_key("acc-dis")).map_err(SharedString::from)?;
    let updated = gateway::client()
        .with_mut(|gw| gw.disable_account(session, account_id, &req))
        .map_err(|err| SharedString::from(err.to_string()))?;
    load_accounts(session);
    Ok(updated)
}
