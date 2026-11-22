use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::types::{AccountStatus, EnrollmentStatus, GradeStatus, Role};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AccountSession {
    pub account_id: String,
    pub username: String,
    pub display_name: String,
    pub role: Role,
    pub bound_student_no: Option<String>,
    pub access_token: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
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
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct HomeroomSummary {
    pub id: String,
    pub name: String,
    pub grade_year: u16,
    pub capacity: Option<u32>,
    pub active_count: u32,
    pub version: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
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
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
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
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct StudentDetailPayload {
    pub student: StudentSummary,
    pub changes: Vec<EnrollmentChangeRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CourseSummary {
    pub id: String,
    pub code: String,
    pub name: String,
    #[cfg_attr(feature = "openapi", schema(value_type = String))]
    pub max_score: Decimal,
    #[cfg_attr(feature = "openapi", schema(value_type = String))]
    pub pass_score: Decimal,
    pub version: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
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
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GradeEntrySummary {
    pub id: String,
    pub teaching_id: String,
    pub course_code: String,
    pub course_name: String,
    pub term: String,
    pub student_no: String,
    pub student_name: String,
    #[cfg_attr(feature = "openapi", schema(value_type = String))]
    pub score: Decimal,
    #[cfg_attr(feature = "openapi", schema(value_type = String))]
    pub max_score: Decimal,
    pub status: GradeStatus,
    pub version: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SignInRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct EnrollStudentRequest {
    pub full_name: String,
    pub homeroom_id: String,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct TransferHomeroomRequest {
    pub target_homeroom_id: String,
    pub reason: String,
    pub version: u64,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct TransitionStudentRequest {
    pub reason: String,
    pub version: u64,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ReverseEnrollmentRequest {
    pub target_status: EnrollmentStatus,
    pub reason: String,
    pub version: u64,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CreateHomeroomRequest {
    pub name: String,
    pub grade_year: u16,
    pub capacity: Option<u32>,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct UpdateHomeroomCapacityRequest {
    pub capacity: Option<u32>,
    pub version: u64,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CreateCourseRequest {
    pub code: String,
    pub name: String,
    #[cfg_attr(feature = "openapi", schema(value_type = String))]
    pub max_score: Decimal,
    #[cfg_attr(feature = "openapi", schema(value_type = String))]
    pub pass_score: Decimal,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AssignTeachingRequest {
    pub course_id: String,
    pub homeroom_id: String,
    pub term: String,
    pub teacher_account_id: String,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SaveDraftGradeRequest {
    pub teaching_id: String,
    pub student_no: String,
    #[cfg_attr(feature = "openapi", schema(value_type = String))]
    pub score: Decimal,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PublishGradesRequest {
    pub teaching_id: String,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AmendGradeRequest {
    #[cfg_attr(feature = "openapi", schema(value_type = String))]
    pub new_score: Decimal,
    pub reason: String,
    pub version: u64,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GradeAmendmentRecord {
    pub id: String,
    pub grade_id: String,
    pub course_name: String,
    pub student_no: String,
    pub student_name: String,
    #[cfg_attr(feature = "openapi", schema(value_type = String))]
    pub previous_score: Decimal,
    #[cfg_attr(feature = "openapi", schema(value_type = String))]
    pub new_score: Decimal,
    pub reason: String,
    pub operator_name: String,
    pub occurred_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ProvisionAccountRequest {
    pub username: String,
    pub display_name: String,
    pub role: Role,
    pub bound_student_no: Option<String>,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct BindPersonProfileRequest {
    pub student_no: Option<String>,
    pub version: u64,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ChangeAccountRoleRequest {
    pub role: Role,
    pub version: u64,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DisableAccountRequest {
    pub reason: String,
    pub version: u64,
    pub idempotency_key: String,
}
