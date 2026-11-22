use student_management_contracts::{ChangeAccountRoleRequest, ProblemDetails, Role};
use student_management_primitives::{IdempotencyKey, NonEmptyText, PositiveVersion};

#[derive(Debug, Clone)]
pub struct ChangeAccountRoleCommand {
    pub account_id: NonEmptyText,
    pub new_role: Role,
    pub expected_version: PositiveVersion,
    pub idempotency_key: IdempotencyKey,
}

impl ChangeAccountRoleCommand {
    pub fn parse(account_id: &str, req: &ChangeAccountRoleRequest) -> Result<Self, ProblemDetails> {
        let account_id = NonEmptyText::try_new("账号标识", account_id, 64)
            .map_err(|err| ProblemDetails::bad_request("invalid_account_id", err.to_string()))?;
        let expected_version = PositiveVersion::try_new(req.version)
            .map_err(|err| ProblemDetails::bad_request("invalid_version", err.to_string()))?;
        let idempotency_key = IdempotencyKey::try_new(&req.idempotency_key).map_err(|err| {
            ProblemDetails::bad_request("invalid_idempotency_key", err.to_string())
        })?;
        Ok(Self {
            account_id,
            new_role: req.role,
            expected_version,
            idempotency_key,
        })
    }
}
