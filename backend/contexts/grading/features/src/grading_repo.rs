use student_management_contracts::{PortError, ProblemDetails};

#[must_use]
pub fn port_error_to_problem(err: &PortError) -> ProblemDetails {
    match err {
        PortError::Forbidden(msg) => ProblemDetails::forbidden("port_forbidden", msg.clone()),
        PortError::NotFound(msg) => ProblemDetails::not_found("port_not_found", msg.clone()),
        PortError::Conflict(msg) => ProblemDetails::conflict("port_conflict", msg.clone()),
    }
}
