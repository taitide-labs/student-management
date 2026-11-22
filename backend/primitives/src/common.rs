use std::fmt;

use serde::{Deserialize, Serialize};

use crate::error::PrimitiveError;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NonEmptyText(String);

impl NonEmptyText {
    pub fn try_new(
        field: &'static str,
        raw: &str,
        max_chars: usize,
    ) -> Result<Self, PrimitiveError> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(PrimitiveError::Empty { field });
        }
        if trimmed.chars().count() > max_chars {
            return Err(PrimitiveError::TooLong {
                field,
                max: max_chars,
            });
        }
        Ok(Self(trimmed.to_owned()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for NonEmptyText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ReasonText(String);

impl ReasonText {
    pub const MAX_CHARS: usize = 240;

    pub fn try_new(raw: &str) -> Result<Self, PrimitiveError> {
        let text = NonEmptyText::try_new("原因说明", raw, Self::MAX_CHARS)?;
        Ok(Self(text.0))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ReasonText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct IdempotencyKey(String);

impl IdempotencyKey {
    pub const MAX_CHARS: usize = 128;

    pub fn try_new(raw: &str) -> Result<Self, PrimitiveError> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(PrimitiveError::Empty { field: "幂等键" });
        }
        if trimmed.chars().count() > Self::MAX_CHARS {
            return Err(PrimitiveError::TooLong {
                field: "幂等键",
                max: Self::MAX_CHARS,
            });
        }
        if trimmed.chars().any(char::is_whitespace) {
            return Err(PrimitiveError::InvalidFormat {
                field: "幂等键",
                reason: "不允许包含空白字符",
            });
        }
        Ok(Self(trimmed.to_owned()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PositiveVersion(u64);

impl PositiveVersion {
    pub fn try_new(value: u64) -> Result<Self, PrimitiveError> {
        if value == 0 {
            return Err(PrimitiveError::OutOfRange {
                field: "版本号",
                reason: "必须大于等于 1",
            });
        }
        Ok(Self(value))
    }

    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }

    #[must_use]
    pub const fn next(self) -> Self {
        Self(self.0.saturating_add(1))
    }
}
