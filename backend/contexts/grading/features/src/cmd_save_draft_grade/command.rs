use student_management_contracts::{ProblemDetails, SaveDraftGradeRequest};
use student_management_primitives::{IdempotencyKey, NonEmptyText, Score, StudentNo};

#[derive(Debug, Clone)]
pub struct SaveDraftGradeCommand {
    pub teaching_id: NonEmptyText,
    pub student_no: StudentNo,
    pub score: Score,
    pub idempotency_key: IdempotencyKey,
}

impl SaveDraftGradeCommand {
    pub fn parse(req: &SaveDraftGradeRequest) -> Result<Self, ProblemDetails> {
        let teaching_id = NonEmptyText::try_new("教学任务标识", &req.teaching_id, 64)
            .map_err(|err| ProblemDetails::bad_request("invalid_teaching_id", err.to_string()))?;
        let student_no = StudentNo::try_new(&req.student_no)
            .map_err(|err| ProblemDetails::bad_request("invalid_student_no", err.to_string()))?;
        let score = Score::try_new(req.score)
            .map_err(|err| ProblemDetails::bad_request("invalid_score", err.to_string()))?;
        let idempotency_key = IdempotencyKey::try_new(&req.idempotency_key).map_err(|err| {
            ProblemDetails::bad_request("invalid_idempotency_key", err.to_string())
        })?;
        Ok(Self {
            teaching_id,
            student_no,
            score,
            idempotency_key,
        })
    }
}
