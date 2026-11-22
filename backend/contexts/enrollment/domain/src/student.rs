use student_management_contracts::EnrollmentStatus;
use student_management_primitives::{NonEmptyText, PositiveVersion, ReasonText, StudentNo};

use crate::error::EnrollmentDomainError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnrollmentTransition {
    Suspend,
    Resume,
    Withdraw,
    Graduate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnrollmentChangeDraft {
    pub action: &'static str,
    pub from_summary: String,
    pub to_summary: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Student {
    pub student_no: StudentNo,
    pub full_name: NonEmptyText,
    pub homeroom_id: String,
    pub status: EnrollmentStatus,
    pub enrolled_on: String,
    pub version: PositiveVersion,
}

impl Student {
    #[must_use]
    pub fn enroll(
        student_no: StudentNo,
        full_name: NonEmptyText,
        homeroom_id: &str,
        enrolled_on: String,
    ) -> (Self, EnrollmentChangeDraft) {
        let student = Self {
            student_no,
            full_name,
            homeroom_id: homeroom_id.to_owned(),
            status: EnrollmentStatus::Active,
            enrolled_on,
            version: PositiveVersion::try_new(1).unwrap_or_else(|_| unreachable!()),
        };
        let change = EnrollmentChangeDraft {
            action: "入学建档",
            from_summary: "无".to_owned(),
            to_summary: format!("在籍 · {homeroom_id}"),
            reason: "新生入学建档".to_owned(),
        };
        (student, change)
    }

    pub fn transfer_homeroom(
        &mut self,
        expected_version: PositiveVersion,
        target_homeroom_id: String,
        reason: &ReasonText,
    ) -> Result<EnrollmentChangeDraft, EnrollmentDomainError> {
        self.ensure_version(expected_version)?;
        if self.status.is_terminal() {
            return Err(EnrollmentDomainError::TerminalStatusLocked(self.status));
        }
        if self.status != EnrollmentStatus::Active {
            return Err(EnrollmentDomainError::TransferRequiresActiveStatus);
        }
        if self.homeroom_id == target_homeroom_id {
            return Err(EnrollmentDomainError::SameHomeroomTransfer);
        }
        let previous = self.homeroom_id.clone();
        self.homeroom_id = target_homeroom_id.clone();
        self.version = self.version.next();
        Ok(EnrollmentChangeDraft {
            action: "转班",
            from_summary: previous,
            to_summary: target_homeroom_id,
            reason: reason.as_str().to_owned(),
        })
    }

    pub fn transition(
        &mut self,
        expected_version: PositiveVersion,
        kind: EnrollmentTransition,
        reason: &ReasonText,
    ) -> Result<EnrollmentChangeDraft, EnrollmentDomainError> {
        self.ensure_version(expected_version)?;
        if self.status.is_terminal() {
            return Err(EnrollmentDomainError::TerminalStatusLocked(self.status));
        }

        let next = match (self.status, kind) {
            (EnrollmentStatus::Active, EnrollmentTransition::Suspend) => {
                EnrollmentStatus::Suspended
            }
            (EnrollmentStatus::Suspended, EnrollmentTransition::Resume) => EnrollmentStatus::Active,
            (
                EnrollmentStatus::Active | EnrollmentStatus::Suspended,
                EnrollmentTransition::Withdraw,
            ) => EnrollmentStatus::Withdrawn,
            (
                EnrollmentStatus::Active | EnrollmentStatus::Suspended,
                EnrollmentTransition::Graduate,
            ) => EnrollmentStatus::Graduated,
            _ => return Err(EnrollmentDomainError::InvalidTransition { from: self.status }),
        };

        let from = self.status;
        self.status = next;
        self.version = self.version.next();
        Ok(EnrollmentChangeDraft {
            action: transition_action_label(kind),
            from_summary: from.label().to_owned(),
            to_summary: next.label().to_owned(),
            reason: reason.as_str().to_owned(),
        })
    }

    pub fn reverse_terminal_status(
        &mut self,
        expected_version: PositiveVersion,
        target_status: EnrollmentStatus,
        reason: &ReasonText,
    ) -> Result<EnrollmentChangeDraft, EnrollmentDomainError> {
        self.ensure_version(expected_version)?;
        if !self.status.is_terminal() {
            return Err(EnrollmentDomainError::ReversalRequiresTerminalStatus);
        }
        if target_status.is_terminal() {
            return Err(EnrollmentDomainError::ReversalTargetMustBeNonTerminal);
        }
        let from = self.status;
        self.status = target_status;
        self.version = self.version.next();
        Ok(EnrollmentChangeDraft {
            action: "终态冲正",
            from_summary: from.label().to_owned(),
            to_summary: target_status.label().to_owned(),
            reason: reason.as_str().to_owned(),
        })
    }

    fn ensure_version(
        &self,
        expected_version: PositiveVersion,
    ) -> Result<(), EnrollmentDomainError> {
        if self.version != expected_version {
            return Err(EnrollmentDomainError::VersionConflict {
                expected: expected_version.get(),
                actual: self.version.get(),
            });
        }
        Ok(())
    }
}

const fn transition_action_label(kind: EnrollmentTransition) -> &'static str {
    match kind {
        EnrollmentTransition::Suspend => "休学",
        EnrollmentTransition::Resume => "复学",
        EnrollmentTransition::Withdraw => "退学",
        EnrollmentTransition::Graduate => "毕业",
    }
}
