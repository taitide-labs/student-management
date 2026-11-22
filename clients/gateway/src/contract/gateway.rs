use std::{cell::RefCell, rc::Rc};

use student_management_contracts::{
    AccountSession, AccountSummary, AmendGradeRequest, AssignTeachingRequest,
    BindPersonProfileRequest, ChangeAccountRoleRequest, CourseSummary, CreateCourseRequest,
    CreateHomeroomRequest, DisableAccountRequest, EnrollStudentRequest, EnrollmentStatus,
    GradeAmendmentRecord, GradeEntrySummary, HomeroomSummary, ProvisionAccountRequest,
    PublishGradesRequest, ReverseEnrollmentRequest, Role, SaveDraftGradeRequest, SignInRequest,
    StudentDetailPayload, StudentSummary, TeachingSummary, TransferHomeroomRequest,
    TransitionStudentRequest, UpdateHomeroomCapacityRequest,
};

use crate::contract::GatewayError;

pub trait Gateway {
    fn sign_in(&self, req: &SignInRequest) -> Result<AccountSession, GatewayError>;
    fn switch_role(&self, role: Role) -> Result<AccountSession, GatewayError>;
    fn list_homerooms(&self) -> Result<Vec<HomeroomSummary>, GatewayError>;
    fn create_homeroom(
        &mut self,
        session: &AccountSession,
        req: &CreateHomeroomRequest,
    ) -> Result<HomeroomSummary, GatewayError>;
    fn update_homeroom_capacity(
        &mut self,
        session: &AccountSession,
        homeroom_id: &str,
        req: &UpdateHomeroomCapacityRequest,
    ) -> Result<HomeroomSummary, GatewayError>;
    fn list_students(
        &self,
        session: &AccountSession,
        query: &str,
        status_filter: Option<EnrollmentStatus>,
    ) -> Result<Vec<StudentSummary>, GatewayError>;
    fn student_detail(
        &self,
        session: &AccountSession,
        student_no: &str,
    ) -> Result<StudentDetailPayload, GatewayError>;
    fn enroll_student(
        &mut self,
        session: &AccountSession,
        req: &EnrollStudentRequest,
    ) -> Result<StudentSummary, GatewayError>;
    fn transfer_homeroom(
        &mut self,
        session: &AccountSession,
        student_no: &str,
        req: &TransferHomeroomRequest,
    ) -> Result<StudentSummary, GatewayError>;
    fn suspend_student(
        &mut self,
        session: &AccountSession,
        student_no: &str,
        req: &TransitionStudentRequest,
    ) -> Result<StudentSummary, GatewayError>;
    fn resume_student(
        &mut self,
        session: &AccountSession,
        student_no: &str,
        req: &TransitionStudentRequest,
    ) -> Result<StudentSummary, GatewayError>;
    fn withdraw_student(
        &mut self,
        session: &AccountSession,
        student_no: &str,
        req: &TransitionStudentRequest,
    ) -> Result<StudentSummary, GatewayError>;
    fn graduate_student(
        &mut self,
        session: &AccountSession,
        student_no: &str,
        req: &TransitionStudentRequest,
    ) -> Result<StudentSummary, GatewayError>;
    fn reverse_enrollment(
        &mut self,
        session: &AccountSession,
        student_no: &str,
        req: &ReverseEnrollmentRequest,
    ) -> Result<StudentSummary, GatewayError>;
    fn list_courses(&self, session: &AccountSession) -> Result<Vec<CourseSummary>, GatewayError>;
    fn create_course(
        &mut self,
        session: &AccountSession,
        req: &CreateCourseRequest,
    ) -> Result<CourseSummary, GatewayError>;
    fn list_teachings(
        &self,
        session: &AccountSession,
    ) -> Result<Vec<TeachingSummary>, GatewayError>;
    fn assign_teaching(
        &mut self,
        session: &AccountSession,
        req: &AssignTeachingRequest,
    ) -> Result<TeachingSummary, GatewayError>;
    fn list_grades(
        &self,
        session: &AccountSession,
        student_no: Option<&str>,
    ) -> Result<Vec<GradeEntrySummary>, GatewayError>;
    fn save_draft_grade(
        &mut self,
        session: &AccountSession,
        req: &SaveDraftGradeRequest,
    ) -> Result<GradeEntrySummary, GatewayError>;
    fn publish_grades(
        &mut self,
        session: &AccountSession,
        req: &PublishGradesRequest,
    ) -> Result<Vec<GradeEntrySummary>, GatewayError>;
    fn amend_grade(
        &mut self,
        session: &AccountSession,
        grade_id: &str,
        req: &AmendGradeRequest,
    ) -> Result<GradeEntrySummary, GatewayError>;
    fn list_grade_amendments(
        &self,
        session: &AccountSession,
        grade_id: Option<&str>,
    ) -> Result<Vec<GradeAmendmentRecord>, GatewayError>;
    fn list_accounts(&self, session: &AccountSession) -> Result<Vec<AccountSummary>, GatewayError>;
    fn provision_account(
        &mut self,
        session: &AccountSession,
        req: &ProvisionAccountRequest,
    ) -> Result<AccountSummary, GatewayError>;
    fn bind_person_profile(
        &mut self,
        session: &AccountSession,
        account_id: &str,
        req: &BindPersonProfileRequest,
    ) -> Result<AccountSummary, GatewayError>;
    fn change_account_role(
        &mut self,
        session: &AccountSession,
        account_id: &str,
        req: &ChangeAccountRoleRequest,
    ) -> Result<AccountSummary, GatewayError>;
    fn disable_account(
        &mut self,
        session: &AccountSession,
        account_id: &str,
        req: &DisableAccountRequest,
    ) -> Result<AccountSummary, GatewayError>;
    fn set_offline(&mut self, _offline: bool) {}
}

#[derive(Clone)]
pub struct GatewayClient {
    inner: Rc<RefCell<dyn Gateway>>,
}

impl GatewayClient {
    #[must_use]
    pub fn new(inner: Rc<RefCell<dyn Gateway>>) -> Self {
        Self { inner }
    }

    pub fn with<R>(&self, f: impl FnOnce(&dyn Gateway) -> R) -> R {
        let guard = self.inner.borrow();
        f(&*guard)
    }

    pub fn with_mut<R>(&self, f: impl FnOnce(&mut dyn Gateway) -> R) -> R {
        let mut guard = self.inner.borrow_mut();
        f(&mut *guard)
    }
}
