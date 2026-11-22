use student_management_contracts::ProblemDetails;
use student_management_identity_domain::{Account, IdentityDomainError};

use super::command::DisableAccountCommand;

pub fn apply(
    account: &mut Account,
    operator_account_id: &str,
    cmd: DisableAccountCommand,
) -> Result<(), ProblemDetails> {
    account
        .disable(operator_account_id, cmd.expected_version, cmd.reason)
        .map_err(|err| match err {
            IdentityDomainError::StudentBindingRequiresStudentRole => {
                ProblemDetails::bad_request("student_binding_role_mismatch", err.to_string())
            }
            IdentityDomainError::SelfDisableForbidden => {
                ProblemDetails::conflict("self_disable_forbidden", err.to_string())
            }
            IdentityDomainError::AlreadyDisabled => {
                ProblemDetails::conflict("already_disabled", err.to_string())
            }
            IdentityDomainError::DisabledAccountImmutable
            | IdentityDomainError::VersionConflict { .. } => {
                ProblemDetails::conflict("account_conflict", err.to_string())
            }
        })
}
