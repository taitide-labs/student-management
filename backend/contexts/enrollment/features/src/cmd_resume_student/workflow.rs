use student_management_contracts::ProblemDetails;
use student_management_enrollment_domain::{
    EnrollmentChangeDraft, EnrollmentDomainError, EnrollmentTransition, Student,
};

use super::command::ResumeStudentCommand;

pub fn apply(
    student: &mut Student,
    cmd: &ResumeStudentCommand,
) -> Result<EnrollmentChangeDraft, ProblemDetails> {
    student
        .transition(
            cmd.expected_version,
            EnrollmentTransition::Resume,
            &cmd.reason,
        )
        .map_err(|err| match err {
            EnrollmentDomainError::TerminalStatusLocked(_)
            | EnrollmentDomainError::InvalidTransition { .. }
            | EnrollmentDomainError::VersionConflict { .. } => {
                ProblemDetails::conflict("enrollment_transition_conflict", err.to_string())
            }
            EnrollmentDomainError::HomeroomCapacityExceeded { .. }
            | EnrollmentDomainError::CapacityBelowActiveCount { .. }
            | EnrollmentDomainError::TransferRequiresActiveStatus
            | EnrollmentDomainError::ReversalRequiresTerminalStatus
            | EnrollmentDomainError::ReversalTargetMustBeNonTerminal
            | EnrollmentDomainError::SameHomeroomTransfer => {
                ProblemDetails::bad_request("invalid_operation", err.to_string())
            }
        })
}
