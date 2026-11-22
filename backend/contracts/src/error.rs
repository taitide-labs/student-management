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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Error)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[error("{title} ({status}): {detail}")]
pub struct ProblemDetails {
    #[serde(rename = "type")]
    pub problem_type: String,
    pub title: String,
    pub status: u16,
    pub detail: String,
    pub code: String,
}

impl ProblemDetails {
    #[must_use]
    pub fn bad_request(code: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            problem_type: "https://student-management.local/problems/bad-request".to_owned(),
            title: "请求参数无效".to_owned(),
            status: 400,
            detail: detail.into(),
            code: code.into(),
        }
    }

    #[must_use]
    pub fn unauthorized(code: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            problem_type: "https://student-management.local/problems/unauthorized".to_owned(),
            title: "未认证或会话已失效".to_owned(),
            status: 401,
            detail: detail.into(),
            code: code.into(),
        }
    }

    #[must_use]
    pub fn forbidden(code: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            problem_type: "https://student-management.local/problems/forbidden".to_owned(),
            title: "无权执行该操作".to_owned(),
            status: 403,
            detail: detail.into(),
            code: code.into(),
        }
    }

    #[must_use]
    pub fn not_found(code: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            problem_type: "https://student-management.local/problems/not-found".to_owned(),
            title: "目标资源不存在".to_owned(),
            status: 404,
            detail: detail.into(),
            code: code.into(),
        }
    }

    #[must_use]
    pub fn conflict(code: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            problem_type: "https://student-management.local/problems/conflict".to_owned(),
            title: "业务状态或版本冲突".to_owned(),
            status: 409,
            detail: detail.into(),
            code: code.into(),
        }
    }

    #[must_use]
    pub fn not_implemented(slice: impl Into<String>) -> Self {
        let name = slice.into();
        Self {
            problem_type: "https://student-management.local/problems/not-implemented".to_owned(),
            title: "切片尚未激活".to_owned(),
            status: 501,
            detail: format!("切片 {name} 处于 Planned 状态，尚未实现"),
            code: "slice_planned".to_owned(),
        }
    }
}
