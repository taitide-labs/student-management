use std::{future::Future, ops::Deref, pin::Pin, rc::Rc};

use student_management_contracts::{
    AccountSession, AccountSummary, AmendGradeRequest, AssignTeachingRequest,
    BindPersonProfileRequest, ChangeAccountRoleRequest, CourseSummary, CreateCourseRequest,
    CreateHomeroomRequest, DisableAccountRequest, EnrollStudentRequest, EnrollmentStatus,
    GradeAmendmentRecord, GradeEntrySummary, HomeroomSummary, ProvisionAccountRequest,
    PublishGradesRequest, ReverseEnrollmentRequest, SaveDraftGradeRequest, SignInRequest,
    StudentDetailPayload, StudentSummary, TeachingSummary, TransferHomeroomRequest,
    TransitionStudentRequest, UpdateHomeroomCapacityRequest,
};

use crate::contract::GatewayError;

pub type GatewayFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

pub trait Gateway {
    fn sign_in<'a>(
        &'a self,
        req: &'a SignInRequest,
    ) -> GatewayFuture<'a, Result<AccountSession, GatewayError>>;
    fn list_homerooms(&self) -> GatewayFuture<'_, Result<Vec<HomeroomSummary>, GatewayError>>;
    fn create_homeroom<'a>(
        &'a self,
        session: &'a AccountSession,
        req: &'a CreateHomeroomRequest,
    ) -> GatewayFuture<'a, Result<HomeroomSummary, GatewayError>>;
    fn update_homeroom_capacity<'a>(
        &'a self,
        session: &'a AccountSession,
        homeroom_id: &'a str,
        req: &'a UpdateHomeroomCapacityRequest,
    ) -> GatewayFuture<'a, Result<HomeroomSummary, GatewayError>>;
    fn list_students<'a>(
        &'a self,
        session: &'a AccountSession,
        query: &'a str,
        status_filter: Option<EnrollmentStatus>,
    ) -> GatewayFuture<'a, Result<Vec<StudentSummary>, GatewayError>>;
    fn student_detail<'a>(
        &'a self,
        session: &'a AccountSession,
        student_no: &'a str,
    ) -> GatewayFuture<'a, Result<StudentDetailPayload, GatewayError>>;
    fn enroll_student<'a>(
        &'a self,
        session: &'a AccountSession,
        req: &'a EnrollStudentRequest,
    ) -> GatewayFuture<'a, Result<StudentSummary, GatewayError>>;
    fn transfer_homeroom<'a>(
        &'a self,
        session: &'a AccountSession,
        student_no: &'a str,
        req: &'a TransferHomeroomRequest,
    ) -> GatewayFuture<'a, Result<StudentSummary, GatewayError>>;
    fn suspend_student<'a>(
        &'a self,
        session: &'a AccountSession,
        student_no: &'a str,
        req: &'a TransitionStudentRequest,
    ) -> GatewayFuture<'a, Result<StudentSummary, GatewayError>>;
    fn resume_student<'a>(
        &'a self,
        session: &'a AccountSession,
        student_no: &'a str,
        req: &'a TransitionStudentRequest,
    ) -> GatewayFuture<'a, Result<StudentSummary, GatewayError>>;
    fn withdraw_student<'a>(
        &'a self,
        session: &'a AccountSession,
        student_no: &'a str,
        req: &'a TransitionStudentRequest,
    ) -> GatewayFuture<'a, Result<StudentSummary, GatewayError>>;
    fn graduate_student<'a>(
        &'a self,
        session: &'a AccountSession,
        student_no: &'a str,
        req: &'a TransitionStudentRequest,
    ) -> GatewayFuture<'a, Result<StudentSummary, GatewayError>>;
    fn reverse_enrollment<'a>(
        &'a self,
        session: &'a AccountSession,
        student_no: &'a str,
        req: &'a ReverseEnrollmentRequest,
    ) -> GatewayFuture<'a, Result<StudentSummary, GatewayError>>;
    fn list_courses<'a>(
        &'a self,
        session: &'a AccountSession,
    ) -> GatewayFuture<'a, Result<Vec<CourseSummary>, GatewayError>>;
    fn create_course<'a>(
        &'a self,
        session: &'a AccountSession,
        req: &'a CreateCourseRequest,
    ) -> GatewayFuture<'a, Result<CourseSummary, GatewayError>>;
    fn list_teachings<'a>(
        &'a self,
        session: &'a AccountSession,
    ) -> GatewayFuture<'a, Result<Vec<TeachingSummary>, GatewayError>>;
    fn assign_teaching<'a>(
        &'a self,
        session: &'a AccountSession,
        req: &'a AssignTeachingRequest,
    ) -> GatewayFuture<'a, Result<TeachingSummary, GatewayError>>;
    fn list_grades<'a>(
        &'a self,
        session: &'a AccountSession,
        student_no: Option<&'a str>,
    ) -> GatewayFuture<'a, Result<Vec<GradeEntrySummary>, GatewayError>>;
    fn save_draft_grade<'a>(
        &'a self,
        session: &'a AccountSession,
        req: &'a SaveDraftGradeRequest,
    ) -> GatewayFuture<'a, Result<GradeEntrySummary, GatewayError>>;
    fn publish_grades<'a>(
        &'a self,
        session: &'a AccountSession,
        req: &'a PublishGradesRequest,
    ) -> GatewayFuture<'a, Result<Vec<GradeEntrySummary>, GatewayError>>;
    fn amend_grade<'a>(
        &'a self,
        session: &'a AccountSession,
        grade_id: &'a str,
        req: &'a AmendGradeRequest,
    ) -> GatewayFuture<'a, Result<GradeEntrySummary, GatewayError>>;
    fn list_grade_amendments<'a>(
        &'a self,
        session: &'a AccountSession,
        grade_id: Option<&'a str>,
    ) -> GatewayFuture<'a, Result<Vec<GradeAmendmentRecord>, GatewayError>>;
    fn list_accounts<'a>(
        &'a self,
        session: &'a AccountSession,
    ) -> GatewayFuture<'a, Result<Vec<AccountSummary>, GatewayError>>;
    fn provision_account<'a>(
        &'a self,
        session: &'a AccountSession,
        req: &'a ProvisionAccountRequest,
    ) -> GatewayFuture<'a, Result<AccountSummary, GatewayError>>;
    fn bind_person_profile<'a>(
        &'a self,
        session: &'a AccountSession,
        account_id: &'a str,
        req: &'a BindPersonProfileRequest,
    ) -> GatewayFuture<'a, Result<AccountSummary, GatewayError>>;
    fn change_account_role<'a>(
        &'a self,
        session: &'a AccountSession,
        account_id: &'a str,
        req: &'a ChangeAccountRoleRequest,
    ) -> GatewayFuture<'a, Result<AccountSummary, GatewayError>>;
    fn disable_account<'a>(
        &'a self,
        session: &'a AccountSession,
        account_id: &'a str,
        req: &'a DisableAccountRequest,
    ) -> GatewayFuture<'a, Result<AccountSummary, GatewayError>>;
}

#[derive(Clone)]
pub struct GatewayClient {
    inner: Rc<dyn Gateway>,
}

impl GatewayClient {
    #[must_use]
    pub fn new(inner: Rc<dyn Gateway>) -> Self {
        Self { inner }
    }
}

impl Deref for GatewayClient {
    type Target = dyn Gateway;

    fn deref(&self) -> &Self::Target {
        &*self.inner
    }
}
