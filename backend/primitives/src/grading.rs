use std::fmt;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::{common::NonEmptyText, error::PrimitiveError};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CourseCode(String);

impl CourseCode {
    pub const MIN_CHARS: usize = 2;
    pub const MAX_CHARS: usize = 24;

    pub fn try_new(raw: &str) -> Result<Self, PrimitiveError> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(PrimitiveError::Empty {
                field: "课程编码"
            });
        }
        if trimmed.len() < Self::MIN_CHARS || trimmed.len() > Self::MAX_CHARS {
            return Err(PrimitiveError::OutOfRange {
                field: "课程编码",
                reason: "长度必须在 2 至 24 个字符之间",
            });
        }
        if !trimmed
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
        {
            return Err(PrimitiveError::InvalidFormat {
                field: "课程编码",
                reason: "仅允许字母、数字、连字符或下划线",
            });
        }
        Ok(Self(trimmed.to_ascii_uppercase()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TermCode(String);

impl TermCode {
    pub const MAX_CHARS: usize = 32;

    pub fn try_new(raw: &str) -> Result<Self, PrimitiveError> {
        let text = NonEmptyText::try_new("学期标识", raw, Self::MAX_CHARS)?;
        Ok(Self(text.as_str().to_owned()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Score(Decimal);

impl Score {
    pub fn try_new(value: Decimal) -> Result<Self, PrimitiveError> {
        if value.is_sign_negative() {
            return Err(PrimitiveError::OutOfRange {
                field: "成绩分值",
                reason: "不能为负数",
            });
        }
        if value.scale() > 2 {
            return Err(PrimitiveError::InvalidFormat {
                field: "成绩分值",
                reason: "最多保留两位小数",
            });
        }
        Ok(Self(value.normalize()))
    }

    #[must_use]
    pub const fn get(self) -> Decimal {
        self.0
    }
}

impl fmt::Display for Score {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MaxScore(Decimal);

impl MaxScore {
    pub fn try_new(value: Decimal) -> Result<Self, PrimitiveError> {
        if value <= Decimal::ZERO {
            return Err(PrimitiveError::OutOfRange {
                field: "满分",
                reason: "必须大于 0",
            });
        }
        if value.scale() > 2 {
            return Err(PrimitiveError::InvalidFormat {
                field: "满分",
                reason: "最多保留两位小数",
            });
        }
        Ok(Self(value.normalize()))
    }

    #[must_use]
    pub const fn get(self) -> Decimal {
        self.0
    }
}
