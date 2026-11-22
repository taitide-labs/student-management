use nsh_core::prelude::SharedString;
use student_management_gateway::contract::{AccountSession, StudentSummary};

use crate::gateway;

use super::super::store::{next_idempotency_key, refresh_roster};
use super::command;

pub async fn execute(
    session: &AccountSession,
    full_name: &str,
    homeroom_id: Option<&str>,
) -> Result<StudentSummary, SharedString> {
    let req = command::build_request(full_name, homeroom_id, next_idempotency_key("enroll"))
        .map_err(SharedString::from)?;
    let created = gateway::client()
        .enroll_student(session, &req)
        .await
        .map_err(|err| SharedString::from(err.to_string()))?;
    refresh_roster(session).await;
    Ok(created)
}
