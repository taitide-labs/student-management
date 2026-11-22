use nsh_core::prelude::*;
use student_management_gateway::contract::{
    AccountSession, EnrollmentStatus, GatewayError, HomeroomSummary, StudentDetailPayload,
    StudentSummary,
};

use crate::gateway;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StatusFilter {
    #[default]
    All,
    Active,
    Suspended,
    Withdrawn,
    Graduated,
}

impl StatusFilter {
    #[must_use]
    pub const fn as_status(self) -> Option<EnrollmentStatus> {
        match self {
            Self::All => None,
            Self::Active => Some(EnrollmentStatus::Active),
            Self::Suspended => Some(EnrollmentStatus::Suspended),
            Self::Withdrawn => Some(EnrollmentStatus::Withdrawn),
            Self::Graduated => Some(EnrollmentStatus::Graduated),
        }
    }
}

#[derive(Clone, Default)]
struct RosterState {
    students: Vec<StudentSummary>,
    homerooms: Vec<HomeroomSummary>,
    query: String,
    filter: StatusFilter,
    loading: bool,
    error: Option<SharedString>,
    op_seq: u64,
}

fn roster() -> Signal<RosterState> {
    global::<RosterState>()
}

pub fn next_idempotency_key(prefix: &str) -> String {
    let mut seq = 0_u64;
    roster().update(|s| {
        s.op_seq += 1;
        seq = s.op_seq;
    });
    format!("{prefix}-{seq}")
}

pub fn load_roster(session: &AccountSession) {
    roster().update(|s| {
        s.loading = true;
        s.error = None;
    });
    let query = roster().with_untracked(|s| s.query.clone());
    let filter = roster().with_untracked(|s| s.filter.as_status());
    let res = gateway::client().with(|gw| {
        let students = gw.list_students(session, &query, filter)?;
        let homerooms = gw.list_homerooms()?;
        Ok::<_, GatewayError>((students, homerooms))
    });
    match res {
        Ok((students, homerooms)) => roster().update(|s| {
            s.students = students;
            s.homerooms = homerooms;
            s.loading = false;
            s.error = None;
        }),
        Err(err) => roster().update(|s| {
            s.loading = false;
            s.error = Some(SharedString::from(err.to_string()));
        }),
    }
}

pub fn set_query(session: &AccountSession, query: &str) {
    roster().update(|s| s.query = query.to_owned());
    load_roster(session);
}

pub fn set_filter(session: &AccountSession, filter: StatusFilter) {
    roster().update(|s| s.filter = filter);
    load_roster(session);
}

#[must_use]
pub fn student_count() -> usize {
    roster().with(|s| s.students.len())
}

#[must_use]
pub fn student_ids() -> Vec<String> {
    roster().with(|s| s.students.iter().map(|st| st.student_no.clone()).collect())
}

#[must_use]
pub fn student_at(index: usize) -> Option<StudentSummary> {
    roster().with(|s| s.students.get(index).cloned())
}

#[must_use]
pub fn homerooms() -> Vec<HomeroomSummary> {
    roster().with(|s| s.homerooms.clone())
}

#[must_use]
pub fn is_loading() -> bool {
    roster().with(|s| s.loading)
}

#[must_use]
pub fn error_message() -> Option<SharedString> {
    roster().with(|s| s.error.clone())
}

#[must_use]
pub fn current_query() -> String {
    roster().with(|s| s.query.clone())
}

#[must_use]
pub fn current_filter() -> StatusFilter {
    roster().with(|s| s.filter)
}

pub fn student_detail(
    session: &AccountSession,
    student_no: &str,
) -> Result<StudentDetailPayload, SharedString> {
    gateway::client()
        .with(|gw| gw.student_detail(session, student_no))
        .map_err(|err| SharedString::from(err.to_string()))
}
