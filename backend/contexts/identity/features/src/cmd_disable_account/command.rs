use student_management_contracts::{DisableAccountRequest, ProblemDetails};
use student_management_primitives::{IdempotencyKey, NonEmptyText, PositiveVersion, ReasonText};

#[derive(Debug, Clone)]
pub struct DisableAccountCommand {
    pub account_id: NonEmptyText,
    pub reason: ReasonText,
    pub expected_version: PositiveVersion,
    pub idempotency_key: IdempotencyKey,
}

impl DisableAccountCommand {
    pub fn parse(account_id: &str, req: &DisableAccountRequest) -> Result<Self, ProblemDetails> {
        let account_id = NonEmptyText::try_new("账号标识", account_id, 64)
            .map_err(|err| ProblemDetails::bad_request("invalid_account_id", err.to_string()))?;
        let reason = ReasonText::try_new(&req.reason)
            .map_err(|err| ProblemDetails::bad_request("invalid_reason", err.to_string()))?;
        let expected_version = PositiveVersion::try_new(req.version)
            .map_err(|err| ProblemDetails::bad_request("invalid_version", err.to_string()))?;
        let idempotency_key = IdempotencyKey::try_new(&req.idempotency_key).map_err(|err| {
            ProblemDetails::bad_request("invalid_idempotency_key", err.to_string())
        })?;
        Ok(Self {
            account_id,
            reason,
            expected_version,
            idempotency_key,
        })
    }
}
