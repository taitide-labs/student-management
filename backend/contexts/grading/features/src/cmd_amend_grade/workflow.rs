use student_management_contracts::ProblemDetails;
use student_management_grading_domain::{GradeAmendmentDraft, GradeEntry, GradingDomainError};
use student_management_primitives::MaxScore;

use super::command::AmendGradeCommand;

pub fn apply(
    entry: &mut GradeEntry,
    is_admin: bool,
    cmd: &AmendGradeCommand,
    max_score: MaxScore,
) -> Result<GradeAmendmentDraft, ProblemDetails> {
    entry
        .amend(
            is_admin,
            cmd.expected_version,
            cmd.new_score,
            max_score,
            &cmd.reason,
        )
        .map_err(|err| map_domain_error(&err))
}

fn map_domain_error(err: &GradingDomainError) -> ProblemDetails {
    match err {
        GradingDomainError::AmendmentRequiresAdmin => {
            ProblemDetails::forbidden("amendment_requires_admin", err.to_string())
        }
        GradingDomainError::ScoreExceedsMax { .. } => {
            ProblemDetails::bad_request("score_exceeds_max", err.to_string())
        }
        GradingDomainError::AmendmentRequiresPublishedGrade
        | GradingDomainError::AmendmentScoreUnchanged
        | GradingDomainError::PublishedGradeLocked
        | GradingDomainError::VoidedGradeLocked
        | GradingDomainError::VersionConflict { .. } => {
            ProblemDetails::conflict("grade_amendment_conflict", err.to_string())
        }
        GradingDomainError::PassScoreExceedsMax { .. } => {
            ProblemDetails::bad_request("invalid_operation", err.to_string())
        }
    }
}
