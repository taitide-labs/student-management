use rust_decimal::Decimal;
use student_management_contracts::GradeStatus;
use student_management_primitives::{MaxScore, PositiveVersion, ReasonText, Score, StudentNo};

use crate::error::GradingDomainError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GradeAmendmentDraft {
    pub previous_score: Decimal,
    pub new_score: Decimal,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GradeEntry {
    pub id: String,
    pub teaching_id: String,
    pub student_no: StudentNo,
    pub score: Score,
    pub status: GradeStatus,
    pub version: PositiveVersion,
}

impl GradeEntry {
    pub fn new_draft(
        id: String,
        teaching_id: String,
        student_no: StudentNo,
        score: Score,
        max_score: MaxScore,
    ) -> Result<Self, GradingDomainError> {
        ensure_within_max(score, max_score)?;
        Ok(Self {
            id,
            teaching_id,
            student_no,
            score,
            status: GradeStatus::Draft,
            version: PositiveVersion::try_new(1).unwrap_or_else(|_| unreachable!()),
        })
    }

    pub fn save_draft(
        &mut self,
        score: Score,
        max_score: MaxScore,
    ) -> Result<(), GradingDomainError> {
        match self.status {
            GradeStatus::Published => return Err(GradingDomainError::PublishedGradeLocked),
            GradeStatus::Voided => return Err(GradingDomainError::VoidedGradeLocked),
            GradeStatus::Draft => {}
        }
        ensure_within_max(score, max_score)?;
        self.score = score;
        self.version = self.version.next();
        Ok(())
    }

    pub fn publish(&mut self) -> Result<(), GradingDomainError> {
        match self.status {
            GradeStatus::Voided => return Err(GradingDomainError::VoidedGradeLocked),
            GradeStatus::Published => return Ok(()),
            GradeStatus::Draft => {}
        }
        self.status = GradeStatus::Published;
        self.version = self.version.next();
        Ok(())
    }

    pub fn amend(
        &mut self,
        is_admin: bool,
        expected_version: PositiveVersion,
        new_score: Score,
        max_score: MaxScore,
        reason: &ReasonText,
    ) -> Result<GradeAmendmentDraft, GradingDomainError> {
        if !is_admin {
            return Err(GradingDomainError::AmendmentRequiresAdmin);
        }
        if self.status != GradeStatus::Published {
            return Err(GradingDomainError::AmendmentRequiresPublishedGrade);
        }
        if self.version != expected_version {
            return Err(GradingDomainError::VersionConflict {
                expected: expected_version.get(),
                actual: self.version.get(),
            });
        }
        ensure_within_max(new_score, max_score)?;
        if new_score == self.score {
            return Err(GradingDomainError::AmendmentScoreUnchanged);
        }
        let previous = self.score.get();
        self.score = new_score;
        self.version = self.version.next();
        Ok(GradeAmendmentDraft {
            previous_score: previous,
            new_score: new_score.get(),
            reason: reason.as_str().to_owned(),
        })
    }

    pub fn void_if_draft(&mut self) -> bool {
        if self.status != GradeStatus::Draft {
            return false;
        }
        self.status = GradeStatus::Voided;
        self.version = self.version.next();
        true
    }
}

fn ensure_within_max(score: Score, max_score: MaxScore) -> Result<(), GradingDomainError> {
    if score.get() > max_score.get() {
        return Err(GradingDomainError::ScoreExceedsMax {
            score: score.get(),
            max_score: max_score.get(),
        });
    }
    Ok(())
}
