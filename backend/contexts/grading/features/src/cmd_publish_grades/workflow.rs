use student_management_contracts::ProblemDetails;
use student_management_grading_domain::{GradeEntry, GradingDomainError};

pub fn apply(entry: &mut GradeEntry) -> Result<(), ProblemDetails> {
    entry.publish().map_err(|err| match err {
        GradingDomainError::VoidedGradeLocked
        | GradingDomainError::PublishedGradeLocked
        | GradingDomainError::VersionConflict { .. } => {
            ProblemDetails::conflict("grade_publish_conflict", err.to_string())
        }
        GradingDomainError::PassScoreExceedsMax { .. }
        | GradingDomainError::ScoreExceedsMax { .. }
        | GradingDomainError::AmendmentRequiresPublishedGrade
        | GradingDomainError::AmendmentRequiresAdmin
        | GradingDomainError::AmendmentScoreUnchanged => {
            ProblemDetails::bad_request("invalid_operation", err.to_string())
        }
    })
}
