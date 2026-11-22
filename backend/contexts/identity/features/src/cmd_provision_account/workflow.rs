use student_management_contracts::ProblemDetails;
use student_management_identity_domain::{Account, AccountProvisionInput, IdentityDomainError};

use super::command::ProvisionAccountCommand;

pub fn prepare(
    account_id: String,
    cmd: ProvisionAccountCommand,
) -> Result<Account, ProblemDetails> {
    Account::provision(AccountProvisionInput {
        account_id,
        username: cmd.username,
        display_name: cmd.display_name,
        role: cmd.role,
        bound_student_no: cmd.bound_student_no,
    })
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
