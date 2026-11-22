use student_management_contracts::{ProblemDetails, PublishGradesRequest};
use student_management_primitives::{IdempotencyKey, NonEmptyText};

#[derive(Debug, Clone)]
pub struct PublishGradesCommand {
    pub teaching_id: NonEmptyText,
    pub idempotency_key: IdempotencyKey,
}

impl PublishGradesCommand {
    pub fn parse(req: &PublishGradesRequest) -> Result<Self, ProblemDetails> {
        let teaching_id = NonEmptyText::try_new("教学任务标识", &req.teaching_id, 64)
            .map_err(|err| ProblemDetails::bad_request("invalid_teaching_id", err.to_string()))?;
        let idempotency_key = IdempotencyKey::try_new(&req.idempotency_key).map_err(|err| {
            ProblemDetails::bad_request("invalid_idempotency_key", err.to_string())
        })?;
        Ok(Self {
            teaching_id,
            idempotency_key,
        })
    }
}
