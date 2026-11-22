use crate::{
    error::PortError,
    types::{EnrollmentStatus, Role},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperatorPrincipal {
    pub account_id: String,
    pub display_name: String,
    pub role: Role,
    pub bound_student_no: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TeacherIdentity {
    pub account_id: String,
    pub display_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HomeroomSnapshot {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudentEnrollmentSnapshot {
    pub student_no: String,
    pub full_name: String,
    pub homeroom_id: String,
    pub status: EnrollmentStatus,
}

pub trait IdentityLookupPort: Send + Sync {
    fn verify_teacher(
        &self,
        account_id: &str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<TeacherIdentity, PortError>> + Send + '_>,
    >;
}

pub trait EnrollmentQueryPort: Send + Sync {
    fn verify_homeroom_exists(
        &self,
        homeroom_id: &str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<HomeroomSnapshot, PortError>> + Send + '_>,
    >;

    fn check_student_active(
        &self,
        student_no: &str,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<StudentEnrollmentSnapshot, PortError>>
                + Send
                + '_,
        >,
    >;
}
