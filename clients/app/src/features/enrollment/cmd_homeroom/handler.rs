use nsh_core::prelude::SharedString;
use student_management_gateway::contract::{AccountSession, HomeroomSummary};

use crate::gateway;

use super::super::store::{next_idempotency_key, refresh_roster};
use super::command;

pub async fn create_homeroom(
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
        .create_homeroom(session, &req)
        .await
        .map_err(|err| SharedString::from(err.to_string()))?;
    refresh_roster(session).await;
    Ok(created)
}

pub async fn update_capacity(
    session: &AccountSession,
    homeroom_id: &str,
    capacity_raw: &str,
    version: u64,
) -> Result<HomeroomSummary, SharedString> {
    let req = command::build_update_capacity(capacity_raw, version, next_idempotency_key("hr-cap"))
        .map_err(SharedString::from)?;
    let updated = gateway::client()
        .update_homeroom_capacity(session, homeroom_id, &req)
        .await
        .map_err(|err| SharedString::from(err.to_string()))?;
    refresh_roster(session).await;
    Ok(updated)
}
