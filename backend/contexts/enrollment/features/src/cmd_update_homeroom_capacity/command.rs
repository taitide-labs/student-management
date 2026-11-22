use std::num::NonZeroU32;

use student_management_contracts::{ProblemDetails, UpdateHomeroomCapacityRequest};
use student_management_primitives::{IdempotencyKey, NonEmptyText, PositiveVersion};

#[derive(Debug, Clone)]
pub struct UpdateHomeroomCapacityCommand {
    pub homeroom_id: NonEmptyText,
    pub capacity: Option<NonZeroU32>,
    pub expected_version: PositiveVersion,
    pub idempotency_key: IdempotencyKey,
}

impl UpdateHomeroomCapacityCommand {
    pub fn parse(
        homeroom_id: &str,
        req: &UpdateHomeroomCapacityRequest,
    ) -> Result<Self, ProblemDetails> {
        let homeroom_id = NonEmptyText::try_new("行政班标识", homeroom_id, 64)
            .map_err(|err| ProblemDetails::bad_request("invalid_homeroom_id", err.to_string()))?;
        let capacity = match req.capacity {
            Some(cap) => Some(NonZeroU32::new(cap).ok_or_else(|| {
                ProblemDetails::bad_request("invalid_capacity", "行政班容量上限必须大于 0")
            })?),
            None => None,
        };
        let expected_version = PositiveVersion::try_new(req.version)
            .map_err(|err| ProblemDetails::bad_request("invalid_version", err.to_string()))?;
        let idempotency_key = IdempotencyKey::try_new(&req.idempotency_key).map_err(|err| {
            ProblemDetails::bad_request("invalid_idempotency_key", err.to_string())
        })?;
        Ok(Self {
            homeroom_id,
            capacity,
            expected_version,
            idempotency_key,
        })
    }
}
