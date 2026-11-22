use student_management_contracts::{AccountStatus, Role};
use student_management_primitives::{
    NonEmptyText, PositiveVersion, ReasonText, StudentNo, Username,
};

use crate::error::IdentityDomainError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountProvisionInput {
    pub account_id: String,
    pub username: Username,
    pub display_name: NonEmptyText,
    pub role: Role,
    pub bound_student_no: Option<StudentNo>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Account {
    pub account_id: String,
    pub username: Username,
    pub display_name: NonEmptyText,
    pub role: Role,
    pub bound_student_no: Option<StudentNo>,
    pub status: AccountStatus,
    pub disable_reason: Option<ReasonText>,
    pub version: PositiveVersion,
}

impl Account {
    pub fn provision(input: AccountProvisionInput) -> Result<Self, IdentityDomainError> {
        if input.bound_student_no.is_some() && input.role != Role::Student {
            return Err(IdentityDomainError::StudentBindingRequiresStudentRole);
        }
        Ok(Self {
            account_id: input.account_id,
            username: input.username,
            display_name: input.display_name,
            role: input.role,
            bound_student_no: input.bound_student_no,
            status: AccountStatus::Active,
            disable_reason: None,
            version: PositiveVersion::try_new(1).unwrap_or_else(|_| unreachable!()),
        })
    }

    pub fn bind_person_profile(
        &mut self,
        expected_version: PositiveVersion,
        student_no: Option<StudentNo>,
    ) -> Result<(), IdentityDomainError> {
        self.ensure_active_and_version(expected_version)?;
        if student_no.is_some() && self.role != Role::Student {
            return Err(IdentityDomainError::StudentBindingRequiresStudentRole);
        }
        self.bound_student_no = student_no;
        self.version = self.version.next();
        Ok(())
    }

    pub fn change_role(
        &mut self,
        expected_version: PositiveVersion,
        new_role: Role,
    ) -> Result<(), IdentityDomainError> {
        self.ensure_active_and_version(expected_version)?;
        self.role = new_role;
        if new_role != Role::Student {
            self.bound_student_no = None;
        }
        self.version = self.version.next();
        Ok(())
    }

    pub fn disable(
        &mut self,
        operator_account_id: &str,
        expected_version: PositiveVersion,
        reason: ReasonText,
    ) -> Result<(), IdentityDomainError> {
        if self.status == AccountStatus::Disabled {
            return Err(IdentityDomainError::AlreadyDisabled);
        }
        if self.version != expected_version {
            return Err(IdentityDomainError::VersionConflict {
                expected: expected_version.get(),
                actual: self.version.get(),
            });
        }
        if operator_account_id == self.account_id {
            return Err(IdentityDomainError::SelfDisableForbidden);
        }
        self.status = AccountStatus::Disabled;
        self.disable_reason = Some(reason);
        self.version = self.version.next();
        Ok(())
    }

    fn ensure_active_and_version(
        &self,
        expected_version: PositiveVersion,
    ) -> Result<(), IdentityDomainError> {
        if self.status == AccountStatus::Disabled {
            return Err(IdentityDomainError::DisabledAccountImmutable);
        }
        if self.version != expected_version {
            return Err(IdentityDomainError::VersionConflict {
                expected: expected_version.get(),
                actual: self.version.get(),
            });
        }
        Ok(())
    }
}
