use student_management_contracts::ProblemDetails;
use student_management_identity_domain::{Account, IdentityDomainError};

use super::command::BindPersonProfileCommand;

pub fn apply(account: &mut Account, cmd: BindPersonProfileCommand) -> Result<(), ProblemDetails> {
    account
        .bind_person_profile(cmd.expected_version, cmd.student_no)
        .map_err(|err| match err {
            IdentityDomainError::StudentBindingRequiresStudentRole => {
                ProblemDetails::bad_request("student_binding_role_mismatch", err.to_string())
            }
            IdentityDomainError::DisabledAccountImmutable
            | IdentityDomainError::SelfDisableForbidden
            | IdentityDomainError::AlreadyDisabled
            | IdentityDomainError::VersionConflict { .. } => {
                ProblemDetails::conflict("account_conflict", err.to_string())
            }
        })
}
