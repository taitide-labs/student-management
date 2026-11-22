use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum PrimitiveError {
    #[error("{field}不能为空")]
    Empty { field: &'static str },
    #[error("{field}长度超出上限（最多 {max} 个字符）")]
    TooLong { field: &'static str, max: usize },
    #[error("{field}格式非法：{reason}")]
    InvalidFormat {
        field: &'static str,
        reason: &'static str,
    },
    #[error("{field}超出允许范围：{reason}")]
    OutOfRange {
        field: &'static str,
        reason: &'static str,
    },
}
