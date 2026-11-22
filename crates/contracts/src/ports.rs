use crate::{error::PortError, types::EnrollmentStatus};

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

pub trait IdentityLookupPort {
    fn verify_teacher(&self, account_id: &str) -> Result<TeacherIdentity, PortError>;
}

pub trait EnrollmentQueryPort {
    fn verify_homeroom_exists(&self, homeroom_id: &str) -> Result<HomeroomSnapshot, PortError>;

    fn check_student_active(
        &self,
        student_no: &str,
    ) -> Result<StudentEnrollmentSnapshot, PortError>;
}
