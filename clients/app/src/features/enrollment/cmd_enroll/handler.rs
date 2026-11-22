use nsh_core::prelude::SharedString;
use student_management_gateway::contract::{AccountSession, StudentSummary};

use crate::gateway;

use super::super::store::{load_roster, next_idempotency_key};
use super::command;

pub fn execute(
    session: &AccountSession,
    full_name: &str,
    homeroom_id: Option<&str>,
) -> Result<StudentSummary, SharedString> {
    let req = command::build_request(full_name, homeroom_id, next_idempotency_key("enroll"))
        .map_err(SharedString::from)?;
    let created = gateway::client()
        .with_mut(|gw| gw.enroll_student(session, &req))
        .map_err(|err| SharedString::from(err.to_string()))?;
    load_roster(session);
    Ok(created)
}
