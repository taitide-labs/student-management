use nsh_core::prelude::SharedString;
use student_management_gateway::contract::{AccountSession, GradeEntrySummary};

use crate::gateway;

use super::super::store::{next_key, refresh_grading};
use super::command;

pub async fn execute(
    session: &AccountSession,
    grade_id: &str,
    version: u64,
    score_raw: &str,
    reason: &str,
) -> Result<GradeEntrySummary, SharedString> {
    let req = command::build_request(score_raw, reason, version, next_key("grd-amd"))
        .map_err(SharedString::from)?;
    let updated = gateway::client()
        .amend_grade(session, grade_id, &req)
        .await
        .map_err(|err| SharedString::from(err.to_string()))?;
    refresh_grading(session).await;
    Ok(updated)
}
