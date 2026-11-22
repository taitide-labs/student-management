use student_management_contracts::{BindPersonProfileRequest, ProblemDetails};
use student_management_primitives::{IdempotencyKey, NonEmptyText, PositiveVersion, StudentNo};

#[derive(Debug, Clone)]
pub struct BindPersonProfileCommand {
    pub account_id: NonEmptyText,
    pub student_no: Option<StudentNo>,
    pub expected_version: PositiveVersion,
    pub idempotency_key: IdempotencyKey,
}

impl BindPersonProfileCommand {
    pub fn parse(account_id: &str, req: &BindPersonProfileRequest) -> Result<Self, ProblemDetails> {
        let account_id = NonEmptyText::try_new("账号标识", account_id, 64)
            .map_err(|err| ProblemDetails::bad_request("invalid_account_id", err.to_string()))?;
        let student_no = req
            .student_no
            .as_deref()
            .filter(|raw| !raw.trim().is_empty())
            .map(StudentNo::try_new)
            .transpose()
            .map_err(|err| ProblemDetails::bad_request("invalid_student_no", err.to_string()))?;
        let expected_version = PositiveVersion::try_new(req.version)
            .map_err(|err| ProblemDetails::bad_request("invalid_version", err.to_string()))?;
        let idempotency_key = IdempotencyKey::try_new(&req.idempotency_key).map_err(|err| {
            ProblemDetails::bad_request("invalid_idempotency_key", err.to_string())
        })?;
        Ok(Self {
            account_id,
            student_no,
            expected_version,
            idempotency_key,
        })
    }
}
