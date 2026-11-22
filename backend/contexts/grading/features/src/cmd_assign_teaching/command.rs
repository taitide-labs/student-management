use student_management_contracts::{AssignTeachingRequest, ProblemDetails};
use student_management_primitives::{IdempotencyKey, NonEmptyText, TermCode};

#[derive(Debug, Clone)]
pub struct AssignTeachingCommand {
    pub course_id: NonEmptyText,
    pub homeroom_id: NonEmptyText,
    pub term: TermCode,
    pub teacher_account_id: NonEmptyText,
    pub idempotency_key: IdempotencyKey,
}

impl AssignTeachingCommand {
    pub fn parse(req: &AssignTeachingRequest) -> Result<Self, ProblemDetails> {
        let course_id = NonEmptyText::try_new("课程标识", &req.course_id, 64)
            .map_err(|err| ProblemDetails::bad_request("invalid_course_id", err.to_string()))?;
        let homeroom_id = NonEmptyText::try_new("行政班标识", &req.homeroom_id, 64)
            .map_err(|err| ProblemDetails::bad_request("invalid_homeroom_id", err.to_string()))?;
        let term = TermCode::try_new(&req.term)
            .map_err(|err| ProblemDetails::bad_request("invalid_term", err.to_string()))?;
        let teacher_account_id = NonEmptyText::try_new("教师账号标识", &req.teacher_account_id, 64)
            .map_err(|err| {
                ProblemDetails::bad_request("invalid_teacher_account_id", err.to_string())
            })?;
        let idempotency_key = IdempotencyKey::try_new(&req.idempotency_key).map_err(|err| {
            ProblemDetails::bad_request("invalid_idempotency_key", err.to_string())
        })?;
        Ok(Self {
            course_id,
            homeroom_id,
            term,
            teacher_account_id,
            idempotency_key,
        })
    }
}
