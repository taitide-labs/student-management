use student_management_contracts::{ProblemDetails, TransferHomeroomRequest};
use student_management_primitives::{
    IdempotencyKey, NonEmptyText, PositiveVersion, ReasonText, StudentNo,
};

#[derive(Debug, Clone)]
pub struct TransferHomeroomCommand {
    pub student_no: StudentNo,
    pub target_homeroom_id: NonEmptyText,
    pub reason: ReasonText,
    pub expected_version: PositiveVersion,
    pub idempotency_key: IdempotencyKey,
}

impl TransferHomeroomCommand {
    pub fn parse(student_no: &str, req: &TransferHomeroomRequest) -> Result<Self, ProblemDetails> {
        let student_no = StudentNo::try_new(student_no)
            .map_err(|err| ProblemDetails::bad_request("invalid_student_no", err.to_string()))?;
        let target_homeroom_id = NonEmptyText::try_new("目标行政班", &req.target_homeroom_id, 64)
            .map_err(|err| {
            ProblemDetails::bad_request("invalid_homeroom_id", err.to_string())
        })?;
        let reason = ReasonText::try_new(&req.reason)
            .map_err(|err| ProblemDetails::bad_request("invalid_reason", err.to_string()))?;
        let expected_version = PositiveVersion::try_new(req.version)
            .map_err(|err| ProblemDetails::bad_request("invalid_version", err.to_string()))?;
        let idempotency_key = IdempotencyKey::try_new(&req.idempotency_key).map_err(|err| {
            ProblemDetails::bad_request("invalid_idempotency_key", err.to_string())
        })?;
        Ok(Self {
            student_no,
            target_homeroom_id,
            reason,
            expected_version,
            idempotency_key,
        })
    }
}
