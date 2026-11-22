use student_management_contracts::ProblemDetails;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum GatewayError {
    #[error("{0}")]
    Problem(#[from] ProblemDetails),
    #[error("网络不可达：{0}")]
    Transport(String),
}
