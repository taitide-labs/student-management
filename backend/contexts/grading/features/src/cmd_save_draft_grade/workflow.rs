use student_management_contracts::ProblemDetails;
use student_management_grading_domain::{GradeEntry, GradingDomainError};
use student_management_primitives::{MaxScore, Score};

use super::command::SaveDraftGradeCommand;

pub fn create_draft(
    id: String,
    cmd: &SaveDraftGradeCommand,
    max_score: MaxScore,
) -> Result<GradeEntry, ProblemDetails> {
    GradeEntry::new_draft(
        id,
        cmd.teaching_id.as_str().to_owned(),
        cmd.student_no.clone(),
        cmd.score,
        max_score,
    )
    .map_err(|err| map_domain_error(&err))
}

pub fn update_draft(
    entry: &mut GradeEntry,
    score: Score,
    max_score: MaxScore,
) -> Result<(), ProblemDetails> {
    entry
        .save_draft(score, max_score)
        .map_err(|err| map_domain_error(&err))
}

fn map_domain_error(err: &GradingDomainError) -> ProblemDetails {
    match err {
        GradingDomainError::ScoreExceedsMax { .. } => {
            ProblemDetails::bad_request("score_exceeds_max", err.to_string())
        }
        GradingDomainError::PublishedGradeLocked
        | GradingDomainError::VoidedGradeLocked
        | GradingDomainError::VersionConflict { .. } => {
            ProblemDetails::conflict("grade_locked_or_conflict", err.to_string())
        }
        GradingDomainError::PassScoreExceedsMax { .. }
        | GradingDomainError::AmendmentRequiresPublishedGrade
        | GradingDomainError::AmendmentRequiresAdmin
        | GradingDomainError::AmendmentScoreUnchanged => {
            ProblemDetails::bad_request("invalid_operation", err.to_string())
        }
    }
}
