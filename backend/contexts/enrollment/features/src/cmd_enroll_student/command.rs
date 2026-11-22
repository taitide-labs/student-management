use student_management_contracts::{EnrollStudentRequest, ProblemDetails};
use student_management_primitives::{IdempotencyKey, NonEmptyText};

#[derive(Debug, Clone)]
pub struct EnrollStudentCommand {
    pub full_name: NonEmptyText,
    pub homeroom_id: NonEmptyText,
    pub idempotency_key: IdempotencyKey,
}

impl TryFrom<EnrollStudentRequest> for EnrollStudentCommand {
    type Error = ProblemDetails;

    fn try_from(req: EnrollStudentRequest) -> Result<Self, Self::Error> {
        let full_name = NonEmptyText::try_new("学生姓名", &req.full_name, 64)
            .map_err(|err| ProblemDetails::bad_request("invalid_full_name", err.to_string()))?;
        let homeroom_id = NonEmptyText::try_new("行政班标识", &req.homeroom_id, 64)
            .map_err(|err| ProblemDetails::bad_request("invalid_homeroom_id", err.to_string()))?;
        let idempotency_key = IdempotencyKey::try_new(&req.idempotency_key).map_err(|err| {
            ProblemDetails::bad_request("invalid_idempotency_key", err.to_string())
        })?;
        Ok(Self {
            full_name,
            homeroom_id,
            idempotency_key,
        })
    }
}
