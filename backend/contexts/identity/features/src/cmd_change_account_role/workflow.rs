use student_management_contracts::ProblemDetails;
use student_management_identity_domain::{Account, IdentityDomainError};

use super::command::ChangeAccountRoleCommand;

pub fn apply(account: &mut Account, cmd: &ChangeAccountRoleCommand) -> Result<(), ProblemDetails> {
    account
        .change_role(cmd.expected_version, cmd.new_role)
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
