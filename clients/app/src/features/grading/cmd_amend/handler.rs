use nsh_core::prelude::SharedString;
use student_management_gateway::contract::{AccountSession, GradeEntrySummary};

use crate::gateway;

use super::super::store::{load_grading, next_key};
use super::command;

pub fn execute(
    session: &AccountSession,
    grade_id: &str,
    version: u64,
    score_raw: &str,
    reason: &str,
) -> Result<GradeEntrySummary, SharedString> {
    let req = command::build_request(score_raw, reason, version, next_key("grd-amd"))
        .map_err(SharedString::from)?;
    let updated = gateway::client()
        .with_mut(|gw| gw.amend_grade(session, grade_id, &req))
        .map_err(|err| SharedString::from(err.to_string()))?;
    load_grading(session);
    Ok(updated)
}
