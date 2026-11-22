use nsh_core::prelude::SharedString;
use student_management_gateway::contract::{AccountSession, GradeEntrySummary};

use crate::gateway;

use super::super::store::{load_grading, next_key};
use super::command;

pub fn save_draft(
    session: &AccountSession,
    teaching_id: &str,
    student_no: &str,
    score_raw: &str,
) -> Result<GradeEntrySummary, SharedString> {
    let req = command::build_save_draft(teaching_id, student_no, score_raw, next_key("grd-save"))
        .map_err(SharedString::from)?;
    let saved = gateway::client()
        .with_mut(|gw| gw.save_draft_grade(session, &req))
        .map_err(|err| SharedString::from(err.to_string()))?;
    load_grading(session);
    Ok(saved)
}

pub fn publish_teaching(
    session: &AccountSession,
    teaching_id: &str,
) -> Result<usize, SharedString> {
    let req =
        command::build_publish(teaching_id, next_key("grd-pub")).map_err(SharedString::from)?;
    let published = gateway::client()
        .with_mut(|gw| gw.publish_grades(session, &req))
        .map_err(|err| SharedString::from(err.to_string()))?;
    load_grading(session);
    Ok(published.len())
}
