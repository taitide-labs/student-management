use nsh_core::prelude::SharedString;
use student_management_gateway::contract::{AccountSession, HomeroomSummary};

use crate::gateway;

use super::super::store::{load_roster, next_idempotency_key};
use super::command;

pub fn create_homeroom(
    session: &AccountSession,
    name: &str,
    grade_year_raw: &str,
    capacity_raw: &str,
) -> Result<HomeroomSummary, SharedString> {
    let req = command::build_create(
        name,
        grade_year_raw,
        capacity_raw,
        next_idempotency_key("hr-create"),
    )
    .map_err(SharedString::from)?;
    let created = gateway::client()
        .with_mut(|gw| gw.create_homeroom(session, &req))
        .map_err(|err| SharedString::from(err.to_string()))?;
    load_roster(session);
    Ok(created)
}

pub fn update_capacity(
    session: &AccountSession,
    homeroom_id: &str,
    capacity_raw: &str,
    version: u64,
) -> Result<HomeroomSummary, SharedString> {
    let req = command::build_update_capacity(capacity_raw, version, next_idempotency_key("hr-cap"))
        .map_err(SharedString::from)?;
    let updated = gateway::client()
        .with_mut(|gw| gw.update_homeroom_capacity(session, homeroom_id, &req))
        .map_err(|err| SharedString::from(err.to_string()))?;
    load_roster(session);
    Ok(updated)
}
