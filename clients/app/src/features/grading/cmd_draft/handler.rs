use nsh_core::prelude::SharedString;
use student_management_gateway::contract::{AccountSession, GradeEntrySummary};

use crate::gateway;

use super::super::store::{next_key, refresh_grading};
use super::command;

pub async fn save_draft(
    session: &AccountSession,
    teaching_id: &str,
    student_no: &str,
    score_raw: &str,
) -> Result<GradeEntrySummary, SharedString> {
    let req = command::build_save_draft(teaching_id, student_no, score_raw, next_key("grd-save"))
        .map_err(SharedString::from)?;
    let saved = gateway::client()
        .save_draft_grade(session, &req)
        .await
        .map_err(|err| SharedString::from(err.to_string()))?;
    refresh_grading(session).await;
    Ok(saved)
}

pub async fn publish_teaching(
    session: &AccountSession,
    teaching_id: &str,
) -> Result<usize, SharedString> {
    let req =
        command::build_publish(teaching_id, next_key("grd-pub")).map_err(SharedString::from)?;
    let published = gateway::client()
        .publish_grades(session, &req)
        .await
        .map_err(|err| SharedString::from(err.to_string()))?;
    refresh_grading(session).await;
    Ok(published.len())
}
