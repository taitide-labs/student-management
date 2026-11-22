use nsh_core::prelude::SharedString;
use student_management_gateway::contract::{AccountSession, EnrollmentStatus, StudentSummary};

use crate::gateway;

use super::super::store::{next_idempotency_key, refresh_roster, refresh_student_detail};
use super::command::{self, ActionKind};

pub async fn execute(
    session: &AccountSession,
    student_no: &str,
    version: u64,
    kind: ActionKind,
    target_homeroom_id: Option<&str>,
    reverse_target: EnrollmentStatus,
    reason: &str,
) -> Result<StudentSummary, SharedString> {
    let key = next_idempotency_key("transition");
    let gw = gateway::client();
    let res = match kind {
        ActionKind::Transfer => {
            let req = command::build_transfer(target_homeroom_id, reason, version, key)
                .map_err(SharedString::from)?;
            gw.transfer_homeroom(session, student_no, &req).await
        }
        ActionKind::Suspend | ActionKind::Resume | ActionKind::Withdraw | ActionKind::Graduate => {
            let req =
                command::build_transition(reason, version, key).map_err(SharedString::from)?;
            match kind {
                ActionKind::Suspend => gw.suspend_student(session, student_no, &req).await,
                ActionKind::Resume => gw.resume_student(session, student_no, &req).await,
                ActionKind::Withdraw => gw.withdraw_student(session, student_no, &req).await,
                ActionKind::Graduate => gw.graduate_student(session, student_no, &req).await,
                ActionKind::Transfer | ActionKind::Reverse => {
                    unreachable!("outer match guarantees non-transfer/reverse action")
                }
            }
        }
        ActionKind::Reverse => {
            let req = command::build_reverse(reverse_target, reason, version, key)
                .map_err(SharedString::from)?;
            gw.reverse_enrollment(session, student_no, &req).await
        }
    };
    let updated = res.map_err(|err| SharedString::from(err.to_string()))?;
    refresh_roster(session).await;
    refresh_student_detail(session, student_no).await;
    Ok(updated)
}
