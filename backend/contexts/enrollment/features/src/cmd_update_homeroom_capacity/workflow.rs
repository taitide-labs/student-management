use student_management_contracts::ProblemDetails;
use student_management_enrollment_domain::{EnrollmentDomainError, Homeroom};

use super::command::UpdateHomeroomCapacityCommand;

pub fn apply(
    homeroom: &mut Homeroom,
    active_count: u32,
    cmd: &UpdateHomeroomCapacityCommand,
) -> Result<(), ProblemDetails> {
    homeroom
        .update_capacity(cmd.expected_version, cmd.capacity, active_count)
        .map_err(|err| match err {
            EnrollmentDomainError::CapacityBelowActiveCount { .. }
            | EnrollmentDomainError::VersionConflict { .. } => {
                ProblemDetails::conflict("homeroom_conflict", err.to_string())
            }
            EnrollmentDomainError::HomeroomCapacityExceeded { .. }
            | EnrollmentDomainError::TerminalStatusLocked(_)
            | EnrollmentDomainError::TransferRequiresActiveStatus
            | EnrollmentDomainError::InvalidTransition { .. }
            | EnrollmentDomainError::ReversalRequiresTerminalStatus
            | EnrollmentDomainError::ReversalTargetMustBeNonTerminal
            | EnrollmentDomainError::SameHomeroomTransfer => {
                ProblemDetails::bad_request("invalid_operation", err.to_string())
            }
        })
}
