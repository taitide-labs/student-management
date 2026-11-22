use student_management_contracts::ProblemDetails;
use student_management_enrollment_domain::{
    EnrollmentChangeDraft, EnrollmentDomainError, Homeroom, Student,
};

use super::command::TransferHomeroomCommand;

pub fn apply(
    student: &mut Student,
    target_homeroom: &Homeroom,
    target_active_count: u32,
    cmd: &TransferHomeroomCommand,
) -> Result<EnrollmentChangeDraft, ProblemDetails> {
    target_homeroom
        .ensure_seat_available(target_active_count)
        .map_err(|err| ProblemDetails::conflict("homeroom_capacity_exceeded", err.to_string()))?;
    student
        .transfer_homeroom(
            cmd.expected_version,
            target_homeroom.id.clone(),
            &cmd.reason,
        )
        .map_err(|err| match err {
            EnrollmentDomainError::HomeroomCapacityExceeded { .. }
            | EnrollmentDomainError::TerminalStatusLocked(_)
            | EnrollmentDomainError::TransferRequiresActiveStatus
            | EnrollmentDomainError::SameHomeroomTransfer
            | EnrollmentDomainError::VersionConflict { .. } => {
                ProblemDetails::conflict("transfer_conflict", err.to_string())
            }
            EnrollmentDomainError::CapacityBelowActiveCount { .. }
            | EnrollmentDomainError::InvalidTransition { .. }
            | EnrollmentDomainError::ReversalRequiresTerminalStatus
            | EnrollmentDomainError::ReversalTargetMustBeNonTerminal => {
                ProblemDetails::bad_request("invalid_transfer", err.to_string())
            }
        })
}
