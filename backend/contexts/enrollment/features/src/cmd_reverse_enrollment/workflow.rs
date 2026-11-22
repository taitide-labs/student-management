use student_management_contracts::ProblemDetails;
use student_management_enrollment_domain::{EnrollmentChangeDraft, EnrollmentDomainError, Student};

use super::command::ReverseEnrollmentCommand;

pub fn apply(
    student: &mut Student,
    cmd: &ReverseEnrollmentCommand,
) -> Result<EnrollmentChangeDraft, ProblemDetails> {
    student
        .reverse_terminal_status(cmd.expected_version, cmd.target_status, &cmd.reason)
        .map_err(|err| match err {
            EnrollmentDomainError::VersionConflict { .. } => {
                ProblemDetails::conflict("enrollment_transition_conflict", err.to_string())
            }
            EnrollmentDomainError::ReversalRequiresTerminalStatus
            | EnrollmentDomainError::ReversalTargetMustBeNonTerminal => {
                ProblemDetails::bad_request("invalid_reversal", err.to_string())
            }
            EnrollmentDomainError::HomeroomCapacityExceeded { .. }
            | EnrollmentDomainError::CapacityBelowActiveCount { .. }
            | EnrollmentDomainError::TransferRequiresActiveStatus
            | EnrollmentDomainError::SameHomeroomTransfer
            | EnrollmentDomainError::TerminalStatusLocked(_)
            | EnrollmentDomainError::InvalidTransition { .. } => {
                ProblemDetails::bad_request("invalid_operation", err.to_string())
            }
        })
}
