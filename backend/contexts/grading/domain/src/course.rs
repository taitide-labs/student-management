use student_management_primitives::{CourseCode, MaxScore, NonEmptyText, PositiveVersion, Score};

use crate::error::GradingDomainError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Course {
    pub id: String,
    pub code: CourseCode,
    pub name: NonEmptyText,
    pub max_score: MaxScore,
    pub pass_score: Score,
    pub version: PositiveVersion,
}

impl Course {
    pub fn create(
        id: String,
        code: CourseCode,
        name: NonEmptyText,
        max_score: MaxScore,
        pass_score: Score,
    ) -> Result<Self, GradingDomainError> {
        if pass_score.get() > max_score.get() {
            return Err(GradingDomainError::PassScoreExceedsMax {
                pass_score: pass_score.get(),
                max_score: max_score.get(),
            });
        }
        Ok(Self {
            id,
            code,
            name,
            max_score,
            pass_score,
            version: PositiveVersion::try_new(1).unwrap_or_else(|_| unreachable!()),
        })
    }
}
