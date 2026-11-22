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
    detail: Option<(String, StudentDetailPayload)>,
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

pub async fn refresh_roster(session: &AccountSession) {
    roster().update(|s| {
        s.loading = true;
        s.error = None;
    });
    let query = roster().with_untracked(|s| s.query.clone());
    let filter = roster().with_untracked(|s| s.filter.as_status());
    let gw = gateway::client();
    let res = async {
        let students = gw.list_students(session, &query, filter).await?;
        let homerooms = gw.list_homerooms().await?;
        Ok::<_, GatewayError>((students, homerooms))
    }
    .await;
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

pub fn load_roster(session: &AccountSession) {
    roster().update(|s| {
        s.loading = true;
        s.error = None;
    });
    let active = session.clone();
    spawn(async move {
        refresh_roster(&active).await;
    });
}

pub async fn refresh_student_detail(session: &AccountSession, student_no: &str) {
    let result = gateway::client().student_detail(session, student_no).await;
    roster().update(|s| {
        s.detail = result.ok().map(|payload| (student_no.to_owned(), payload));
    });
}

pub fn load_student_detail(session: &AccountSession, student_no: &str) {
    roster().update(|s| {
        if s.detail.as_ref().is_some_and(|(no, _)| no != student_no) {
            s.detail = None;
        }
    });
    let active = session.clone();
    let target = student_no.to_owned();
    spawn(async move {
        refresh_student_detail(&active, &target).await;
    });
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

#[must_use]
pub fn current_student_detail(student_no: &str) -> Option<StudentDetailPayload> {
    roster().with(|s| {
        s.detail
            .as_ref()
            .filter(|(no, _)| no == student_no)
            .map(|(_, payload)| payload.clone())
    })
}
