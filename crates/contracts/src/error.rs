use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Error)]
pub enum PortError {
    #[error("权限不足：{0}")]
    Forbidden(String),
    #[error("目标不存在：{0}")]
    NotFound(String),
    #[error("业务前置校验失败：{0}")]
    Conflict(String),
}
