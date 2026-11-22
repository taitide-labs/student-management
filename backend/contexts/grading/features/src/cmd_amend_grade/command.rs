use student_management_contracts::{AmendGradeRequest, ProblemDetails};
use student_management_primitives::{
    IdempotencyKey, NonEmptyText, PositiveVersion, ReasonText, Score,
};

#[derive(Debug, Clone)]
pub struct AmendGradeCommand {
    pub grade_id: NonEmptyText,
    pub new_score: Score,
    pub reason: ReasonText,
    pub expected_version: PositiveVersion,
    pub idempotency_key: IdempotencyKey,
}

impl AmendGradeCommand {
    pub fn parse(grade_id: &str, req: &AmendGradeRequest) -> Result<Self, ProblemDetails> {
        let grade_id = NonEmptyText::try_new("成绩记录标识", grade_id, 64)
            .map_err(|err| ProblemDetails::bad_request("invalid_grade_id", err.to_string()))?;
        let new_score = Score::try_new(req.new_score)
            .map_err(|err| ProblemDetails::bad_request("invalid_score", err.to_string()))?;
        let reason = ReasonText::try_new(&req.reason)
            .map_err(|err| ProblemDetails::bad_request("invalid_reason", err.to_string()))?;
        let expected_version = PositiveVersion::try_new(req.version)
            .map_err(|err| ProblemDetails::bad_request("invalid_version", err.to_string()))?;
        let idempotency_key = IdempotencyKey::try_new(&req.idempotency_key).map_err(|err| {
            ProblemDetails::bad_request("invalid_idempotency_key", err.to_string())
        })?;
        Ok(Self {
            grade_id,
            new_score,
            reason,
            expected_version,
            idempotency_key,
        })
    }
}
