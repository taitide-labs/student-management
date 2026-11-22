use nsh_core::prelude::SharedString;
use student_management_gateway::contract::{AccountSession, AccountSummary, Role};

use crate::gateway;

use super::super::store::{next_key, refresh_accounts};
use super::command;

pub async fn execute(
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
        .provision_account(session, &req)
        .await
        .map_err(|err| SharedString::from(err.to_string()))?;
    refresh_accounts(session).await;
    Ok(created)
}
