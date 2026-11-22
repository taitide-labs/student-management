use std::fmt;

use serde::{Deserialize, Serialize};

use crate::error::PrimitiveError;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Username(String);

impl Username {
    pub const MIN_CHARS: usize = 2;
    pub const MAX_CHARS: usize = 32;

    pub fn try_new(raw: &str) -> Result<Self, PrimitiveError> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(PrimitiveError::Empty {
                field: "登录账号"
            });
        }
        if trimmed.len() < Self::MIN_CHARS || trimmed.len() > Self::MAX_CHARS {
            return Err(PrimitiveError::OutOfRange {
                field: "登录账号",
                reason: "长度必须在 2 至 32 个字符之间",
            });
        }
        let lower = trimmed.to_ascii_lowercase();
        if !lower.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        }) {
            return Err(PrimitiveError::InvalidFormat {
                field: "登录账号",
                reason: "仅允许小写字母、数字、点、下划线或连字符",
            });
        }
        Ok(Self(lower))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Username {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
