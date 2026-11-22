use std::fmt;

use serde::{Deserialize, Serialize};

use crate::{common::NonEmptyText, error::PrimitiveError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GradeYear(u16);

impl GradeYear {
    pub const MIN: u16 = 1900;
    pub const MAX: u16 = 2999;

    pub fn try_new(year: u16) -> Result<Self, PrimitiveError> {
        if !(Self::MIN..=Self::MAX).contains(&year) {
            return Err(PrimitiveError::OutOfRange {
                field: "年级",
                reason: "必须在 1900 至 2999 之间",
            });
        }
        Ok(Self(year))
    }

    #[must_use]
    pub const fn get(self) -> u16 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct HomeroomName(String);

impl HomeroomName {
    pub const MAX_CHARS: usize = 48;

    pub fn try_new(raw: &str) -> Result<Self, PrimitiveError> {
        let text = NonEmptyText::try_new("班级名称", raw, Self::MAX_CHARS)?;
        Ok(Self(text.as_str().to_owned()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct StudentNo(String);

impl StudentNo {
    pub fn try_new(raw: &str) -> Result<Self, PrimitiveError> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(PrimitiveError::Empty { field: "学号" });
        }
        if trimmed.len() != 8 || !trimmed.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(PrimitiveError::InvalidFormat {
                field: "学号",
                reason: "必须为 8 位数字",
            });
        }
        Ok(Self(trimmed.to_owned()))
    }

    pub fn from_sequence(grade_year: GradeYear, seq: u32) -> Result<Self, PrimitiveError> {
        if !(1..=9999).contains(&seq) {
            return Err(PrimitiveError::OutOfRange {
                field: "学号序号",
                reason: "必须在 1 至 9999 之间",
            });
        }
        let code = format!("{:04}{:04}", grade_year.get(), seq);
        Ok(Self(code))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for StudentNo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
