use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountSession {
    pub account_id: String,
    pub username: String,
    pub display_name: String,
    pub role: Role,
    pub bound_student_no: Option<String>,
    pub access_token: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountSummary {
    pub account_id: String,
    pub username: String,
    pub display_name: String,
    pub role: Role,
    pub bound_student_no: Option<String>,
    pub status: AccountStatus,
    pub version: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HomeroomSummary {
    pub id: String,
    pub name: String,
    pub grade_year: u16,
    pub capacity: Option<u32>,
    pub active_count: u32,
    pub version: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StudentSummary {
    pub student_no: String,
    pub full_name: String,
    pub homeroom_id: String,
    pub homeroom_name: String,
    pub status: EnrollmentStatus,
    pub enrolled_on: String,
    pub version: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnrollmentChangeRecord {
    pub id: String,
    pub student_no: String,
    pub action: String,
    pub from_summary: String,
    pub to_summary: String,
    pub reason: String,
    pub operator_name: String,
    pub occurred_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CourseSummary {
    pub id: String,
    pub code: String,
    pub name: String,
    pub max_score: Decimal,
    pub pass_score: Decimal,
    pub version: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TeachingSummary {
    pub id: String,
    pub course_id: String,
    pub course_code: String,
    pub course_name: String,
    pub homeroom_id: String,
    pub homeroom_name: String,
    pub term: String,
    pub teacher_account_id: String,
    pub teacher_name: String,
    pub version: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GradeEntrySummary {
    pub id: String,
    pub teaching_id: String,
    pub course_code: String,
    pub course_name: String,
    pub term: String,
    pub student_no: String,
    pub student_name: String,
    pub score: Decimal,
    pub max_score: Decimal,
    pub status: GradeStatus,
    pub version: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignInRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnrollStudentRequest {
    pub full_name: String,
    pub homeroom_id: String,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransferHomeroomRequest {
    pub target_homeroom_id: String,
    pub reason: String,
    pub version: u64,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransitionStudentRequest {
    pub reason: String,
    pub version: u64,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReverseEnrollmentRequest {
    pub target_status: EnrollmentStatus,
    pub reason: String,
    pub version: u64,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateHomeroomRequest {
    pub name: String,
    pub grade_year: u16,
    pub capacity: Option<u32>,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateHomeroomCapacityRequest {
    pub capacity: Option<u32>,
    pub version: u64,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateCourseRequest {
    pub code: String,
    pub name: String,
    pub max_score: Decimal,
    pub pass_score: Decimal,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssignTeachingRequest {
    pub course_id: String,
    pub homeroom_id: String,
    pub term: String,
    pub teacher_account_id: String,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaveDraftGradeRequest {
    pub teaching_id: String,
    pub student_no: String,
    pub score: Decimal,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublishGradesRequest {
    pub teaching_id: String,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AmendGradeRequest {
    pub new_score: Decimal,
    pub reason: String,
    pub version: u64,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GradeAmendmentRecord {
    pub id: String,
    pub grade_id: String,
    pub course_name: String,
    pub student_no: String,
    pub student_name: String,
    pub previous_score: Decimal,
    pub new_score: Decimal,
    pub reason: String,
    pub operator_name: String,
    pub occurred_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProvisionAccountRequest {
    pub username: String,
    pub display_name: String,
    pub role: Role,
    pub bound_student_no: Option<String>,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BindPersonProfileRequest {
    pub student_no: Option<String>,
    pub version: u64,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChangeAccountRoleRequest {
    pub role: Role,
    pub version: u64,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DisableAccountRequest {
    pub reason: String,
    pub version: u64,
    pub idempotency_key: String,
}
