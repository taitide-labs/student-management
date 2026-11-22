use std::num::NonZeroU32;

use student_management_contracts::{CreateHomeroomRequest, ProblemDetails};
use student_management_primitives::{GradeYear, HomeroomName, IdempotencyKey};

#[derive(Debug, Clone)]
pub struct CreateHomeroomCommand {
    pub name: HomeroomName,
    pub grade_year: GradeYear,
    pub capacity: Option<NonZeroU32>,
    pub idempotency_key: IdempotencyKey,
}

impl TryFrom<CreateHomeroomRequest> for CreateHomeroomCommand {
    type Error = ProblemDetails;

    fn try_from(req: CreateHomeroomRequest) -> Result<Self, Self::Error> {
        let name = HomeroomName::try_new(&req.name)
            .map_err(|err| ProblemDetails::bad_request("invalid_homeroom_name", err.to_string()))?;
        let grade_year = GradeYear::try_new(req.grade_year)
            .map_err(|err| ProblemDetails::bad_request("invalid_grade_year", err.to_string()))?;
        let capacity = match req.capacity {
            Some(cap) => Some(NonZeroU32::new(cap).ok_or_else(|| {
                ProblemDetails::bad_request("invalid_capacity", "行政班容量上限必须大于 0")
            })?),
            None => None,
        };
        let idempotency_key = IdempotencyKey::try_new(&req.idempotency_key).map_err(|err| {
            ProblemDetails::bad_request("invalid_idempotency_key", err.to_string())
        })?;
        Ok(Self {
            name,
            grade_year,
            capacity,
            idempotency_key,
        })
    }
}
