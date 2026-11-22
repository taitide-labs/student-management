use student_management_contracts::{CreateCourseRequest, ProblemDetails};
use student_management_primitives::{CourseCode, IdempotencyKey, MaxScore, NonEmptyText, Score};

#[derive(Debug, Clone)]
pub struct CreateCourseCommand {
    pub code: CourseCode,
    pub name: NonEmptyText,
    pub max_score: MaxScore,
    pub pass_score: Score,
    pub idempotency_key: IdempotencyKey,
}

impl CreateCourseCommand {
    pub fn parse(req: &CreateCourseRequest) -> Result<Self, ProblemDetails> {
        let code = CourseCode::try_new(&req.code)
            .map_err(|err| ProblemDetails::bad_request("invalid_course_code", err.to_string()))?;
        let name = NonEmptyText::try_new("课程名称", &req.name, 64)
            .map_err(|err| ProblemDetails::bad_request("invalid_course_name", err.to_string()))?;
        let max_score = MaxScore::try_new(req.max_score)
            .map_err(|err| ProblemDetails::bad_request("invalid_max_score", err.to_string()))?;
        let pass_score = Score::try_new(req.pass_score)
            .map_err(|err| ProblemDetails::bad_request("invalid_pass_score", err.to_string()))?;
        let idempotency_key = IdempotencyKey::try_new(&req.idempotency_key).map_err(|err| {
            ProblemDetails::bad_request("invalid_idempotency_key", err.to_string())
        })?;
        Ok(Self {
            code,
            name,
            max_score,
            pass_score,
            idempotency_key,
        })
    }
}
