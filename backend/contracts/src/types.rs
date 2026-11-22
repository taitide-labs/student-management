use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Admin,
    Teacher,
    Student,
}

impl Role {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Admin => "教务管理员",
            Self::Teacher => "教师",
            Self::Student => "学生",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum AccountStatus {
    Active,
    Disabled,
}

impl AccountStatus {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Active => "启用",
            Self::Disabled => "已停用",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum EnrollmentStatus {
    Active,
    Suspended,
    Withdrawn,
    Graduated,
}

impl EnrollmentStatus {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Active => "在籍",
            Self::Suspended => "休学",
            Self::Withdrawn => "退学",
            Self::Graduated => "毕业",
        }
    }

    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Withdrawn | Self::Graduated)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum GradeStatus {
    Draft,
    Published,
    Voided,
}

impl GradeStatus {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Draft => "草稿",
            Self::Published => "已锁定",
            Self::Voided => "已作废",
        }
    }
}
