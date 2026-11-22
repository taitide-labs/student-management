use student_management_contracts::ProblemDetails;
use student_management_grading_domain::{Course, GradingDomainError};

use super::command::CreateCourseCommand;

pub fn apply(id: String, cmd: &CreateCourseCommand) -> Result<Course, ProblemDetails> {
    Course::create(
        id,
        cmd.code.clone(),
        cmd.name.clone(),
        cmd.max_score,
        cmd.pass_score,
    )
    .map_err(|err| match err {
        GradingDomainError::PassScoreExceedsMax { .. } => {
            ProblemDetails::bad_request("invalid_pass_score", err.to_string())
        }
        GradingDomainError::ScoreExceedsMax { .. }
        | GradingDomainError::PublishedGradeLocked
        | GradingDomainError::VoidedGradeLocked
        | GradingDomainError::AmendmentRequiresPublishedGrade
        | GradingDomainError::AmendmentRequiresAdmin
        | GradingDomainError::AmendmentScoreUnchanged
        | GradingDomainError::VersionConflict { .. } => {
            ProblemDetails::bad_request("invalid_operation", err.to_string())
        }
    })
}
