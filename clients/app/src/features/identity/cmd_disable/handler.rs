use nsh_core::prelude::SharedString;
use student_management_gateway::contract::{AccountSession, AccountSummary};

use crate::gateway;

use super::super::store::{next_key, refresh_accounts};
use super::command;

pub async fn execute(
    session: &AccountSession,
    account_id: &str,
    reason: &str,
    version: u64,
) -> Result<AccountSummary, SharedString> {
    let req =
        command::build_request(reason, version, next_key("acc-dis")).map_err(SharedString::from)?;
    let updated = gateway::client()
        .disable_account(session, account_id, &req)
        .await
        .map_err(|err| SharedString::from(err.to_string()))?;
    refresh_accounts(session).await;
    Ok(updated)
}
