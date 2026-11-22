use student_management_contracts::{ProblemDetails, ProvisionAccountRequest, Role};
use student_management_primitives::{IdempotencyKey, NonEmptyText, StudentNo, Username};

#[derive(Debug, Clone)]
pub struct ProvisionAccountCommand {
    pub username: Username,
    pub display_name: NonEmptyText,
    pub role: Role,
    pub bound_student_no: Option<StudentNo>,
    pub idempotency_key: IdempotencyKey,
}

impl TryFrom<ProvisionAccountRequest> for ProvisionAccountCommand {
    type Error = ProblemDetails;

    fn try_from(req: ProvisionAccountRequest) -> Result<Self, Self::Error> {
        let username = Username::try_new(&req.username)
            .map_err(|err| ProblemDetails::bad_request("invalid_username", err.to_string()))?;
        let display_name = NonEmptyText::try_new("显示名称", &req.display_name, 64)
            .map_err(|err| ProblemDetails::bad_request("invalid_display_name", err.to_string()))?;
        let bound_student_no = req
            .bound_student_no
            .as_deref()
            .filter(|raw| !raw.trim().is_empty())
            .map(StudentNo::try_new)
            .transpose()
            .map_err(|err| ProblemDetails::bad_request("invalid_student_no", err.to_string()))?;
        let idempotency_key = IdempotencyKey::try_new(&req.idempotency_key).map_err(|err| {
            ProblemDetails::bad_request("invalid_idempotency_key", err.to_string())
        })?;
        Ok(Self {
            username,
            display_name,
            role: req.role,
            bound_student_no,
            idempotency_key,
        })
    }
}
