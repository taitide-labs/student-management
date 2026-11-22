use nsh_core::prelude::*;
use student_management_gateway::contract::{
    AccountSession, GradeAmendmentRecord, GradeEntrySummary, GradeStatus, TeachingSummary,
};

use crate::gateway;

#[derive(Clone, Default)]
struct GradingState {
    teachings: Vec<TeachingSummary>,
    grades: Vec<GradeEntrySummary>,
    amendments: Vec<GradeAmendmentRecord>,
    op_seq: u64,
}

fn state() -> Signal<GradingState> {
    global::<GradingState>()
}

pub fn next_key(prefix: &str) -> String {
    let mut seq = 0_u64;
    state().update(|s| {
        s.op_seq += 1;
        seq = s.op_seq;
    });
    format!("{prefix}-{seq}")
}

pub async fn refresh_grading(session: &AccountSession) {
    let gw = gateway::client();
    let teachings = gw.list_teachings(session).await.unwrap_or_default();
    let grades = gw.list_grades(session, None).await.unwrap_or_default();
    let amendments = gw
        .list_grade_amendments(session, None)
        .await
        .unwrap_or_default();
    state().update(|s| {
        s.teachings = teachings;
        s.grades = grades;
        s.amendments = amendments;
    });
}

pub fn load_grading(session: &AccountSession) {
    let active = session.clone();
    spawn(async move {
        refresh_grading(&active).await;
    });
}

#[must_use]
pub fn teachings() -> Vec<TeachingSummary> {
    state().with(|s| s.teachings.clone())
}

#[must_use]
pub fn grades() -> Vec<GradeEntrySummary> {
    state().with(|s| s.grades.clone())
}

#[must_use]
pub fn amendments() -> Vec<GradeAmendmentRecord> {
    state().with(|s| s.amendments.clone())
}

#[must_use]
pub fn default_draft_teaching_id() -> Option<String> {
    state().with(|s| {
        s.grades
            .iter()
            .find(|g| g.status == GradeStatus::Draft)
            .map(|g| g.teaching_id.clone())
            .or_else(|| s.teachings.first().map(|t| t.id.clone()))
    })
}
