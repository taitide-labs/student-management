use nsh_core::prelude::SharedString;
use student_management_gateway::contract::{AccountSession, EnrollmentStatus, StudentSummary};

use crate::gateway;

use super::super::store::{load_roster, next_idempotency_key};
use super::command::{self, ActionKind};

pub fn execute(
    session: &AccountSession,
    student_no: &str,
    version: u64,
    kind: ActionKind,
    target_homeroom_id: Option<&str>,
    reverse_target: EnrollmentStatus,
    reason: &str,
) -> Result<StudentSummary, SharedString> {
    let key = next_idempotency_key("transition");
    let res = match kind {
        ActionKind::Transfer => {
            let req = command::build_transfer(target_homeroom_id, reason, version, key)
                .map_err(SharedString::from)?;
            gateway::client().with_mut(|gw| gw.transfer_homeroom(session, student_no, &req))
        }
        ActionKind::Suspend | ActionKind::Resume | ActionKind::Withdraw | ActionKind::Graduate => {
            let req =
                command::build_transition(reason, version, key).map_err(SharedString::from)?;
            gateway::client().with_mut(|gw| match kind {
                ActionKind::Suspend => gw.suspend_student(session, student_no, &req),
                ActionKind::Resume => gw.resume_student(session, student_no, &req),
                ActionKind::Withdraw => gw.withdraw_student(session, student_no, &req),
                ActionKind::Graduate => gw.graduate_student(session, student_no, &req),
                ActionKind::Transfer | ActionKind::Reverse => {
                    unreachable!("outer match guarantees non-transfer/reverse action")
                }
            })
        }
        ActionKind::Reverse => {
            let req = command::build_reverse(reverse_target, reason, version, key)
                .map_err(SharedString::from)?;
            gateway::client().with_mut(|gw| gw.reverse_enrollment(session, student_no, &req))
        }
    };
    let updated = res.map_err(|err| SharedString::from(err.to_string()))?;
    load_roster(session);
    Ok(updated)
}
