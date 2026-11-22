use std::sync::Arc;

use student_management_contracts::{
    AccountSession, AccountSummary, AmendGradeRequest, AssignTeachingRequest,
    BindPersonProfileRequest, ChangeAccountRoleRequest, CourseSummary, CreateCourseRequest,
    CreateHomeroomRequest, DisableAccountRequest, EnrollStudentRequest, EnrollmentStatus,
    GradeAmendmentRecord, GradeEntrySummary, HomeroomSummary, OperatorPrincipal, ProblemDetails,
    ProvisionAccountRequest, PublishGradesRequest, ReverseEnrollmentRequest, Role,
    SaveDraftGradeRequest, SignInRequest, StudentDetailPayload, StudentSummary, TeachingSummary,
    TransferHomeroomRequest, TransitionStudentRequest, UpdateHomeroomCapacityRequest,
};
use student_management_runtime::LocalBackend;
use tokio::runtime::Runtime;

use crate::contract::{Gateway, GatewayError};

pub struct LocalGatewayHandle {
    rt: Arc<Runtime>,
    backend: LocalBackend,
}

impl LocalGatewayHandle {
    pub fn open(database_url: &str) -> Result<Self, GatewayError> {
        let rt = Runtime::new().map_err(|err| GatewayError::Transport(err.to_string()))?;
        let backend = rt
            .block_on(LocalBackend::open(database_url))
            .map_err(GatewayError::Problem)?;
        Ok(Self {
            rt: Arc::new(rt),
            backend,
        })
    }

    #[must_use]
    pub const fn backend(&self) -> &LocalBackend {
        &self.backend
    }

    pub fn ensure_bootstrap_admin(
        &self,
        username: &str,
        display_name: &str,
        password: &str,
    ) -> Result<(), GatewayError> {
        self.rt
            .block_on(
                self.backend
                    .ensure_bootstrap_admin(username, display_name, password),
            )
            .map_err(GatewayError::Problem)
    }
}

fn principal_from_session(session: &AccountSession) -> OperatorPrincipal {
    OperatorPrincipal {
        account_id: session.account_id.clone(),
        display_name: session.display_name.clone(),
        role: session.role,
        bound_student_no: session.bound_student_no.clone(),
    }
}

fn planned_err(slice: &str) -> GatewayError {
    GatewayError::Problem(ProblemDetails::not_implemented(slice))
}

impl Gateway for LocalGatewayHandle {
    fn sign_in(&self, req: &SignInRequest) -> Result<AccountSession, GatewayError> {
        self.rt
            .block_on(self.backend.sign_in(req.clone()))
            .map_err(GatewayError::Problem)
    }

    fn switch_role(&self, _role: Role) -> Result<AccountSession, GatewayError> {
        Err(planned_err("identity::switch_role"))
    }

    fn list_homerooms(&self) -> Result<Vec<HomeroomSummary>, GatewayError> {
        self.rt
            .block_on(self.backend.list_homerooms())
            .map_err(GatewayError::Problem)
    }

    fn create_homeroom(
        &mut self,
        session: &AccountSession,
        req: &CreateHomeroomRequest,
    ) -> Result<HomeroomSummary, GatewayError> {
        let operator = principal_from_session(session);
        self.rt
            .block_on(self.backend.create_homeroom(&operator, req.clone()))
            .map_err(GatewayError::Problem)
    }

    fn update_homeroom_capacity(
        &mut self,
        session: &AccountSession,
        homeroom_id: &str,
        req: &UpdateHomeroomCapacityRequest,
    ) -> Result<HomeroomSummary, GatewayError> {
        let operator = principal_from_session(session);
        self.rt
            .block_on(
                self.backend
                    .update_homeroom_capacity(&operator, homeroom_id, req.clone()),
            )
            .map_err(GatewayError::Problem)
    }

    fn list_students(
        &self,
        session: &AccountSession,
        query: &str,
        status_filter: Option<EnrollmentStatus>,
    ) -> Result<Vec<StudentSummary>, GatewayError> {
        let operator = principal_from_session(session);
        self.rt
            .block_on(self.backend.list_students(&operator, query, status_filter))
            .map_err(GatewayError::Problem)
    }

    fn student_detail(
        &self,
        session: &AccountSession,
        student_no: &str,
    ) -> Result<StudentDetailPayload, GatewayError> {
        let operator = principal_from_session(session);
        self.rt
            .block_on(self.backend.student_detail(&operator, student_no))
            .map_err(GatewayError::Problem)
    }

    fn enroll_student(
        &mut self,
        session: &AccountSession,
        req: &EnrollStudentRequest,
    ) -> Result<StudentSummary, GatewayError> {
        let operator = principal_from_session(session);
        self.rt
            .block_on(self.backend.enroll_student(&operator, req.clone()))
            .map_err(GatewayError::Problem)
    }

    fn transfer_homeroom(
        &mut self,
        session: &AccountSession,
        student_no: &str,
        req: &TransferHomeroomRequest,
    ) -> Result<StudentSummary, GatewayError> {
        let operator = principal_from_session(session);
        self.rt
            .block_on(
                self.backend
                    .transfer_homeroom(&operator, student_no, req.clone()),
            )
            .map_err(GatewayError::Problem)
    }

    fn suspend_student(
        &mut self,
        session: &AccountSession,
        student_no: &str,
        req: &TransitionStudentRequest,
    ) -> Result<StudentSummary, GatewayError> {
        let operator = principal_from_session(session);
        self.rt
            .block_on(
                self.backend
                    .suspend_student(&operator, student_no, req.clone()),
            )
            .map_err(GatewayError::Problem)
    }

    fn resume_student(
        &mut self,
        session: &AccountSession,
        student_no: &str,
        req: &TransitionStudentRequest,
    ) -> Result<StudentSummary, GatewayError> {
        let operator = principal_from_session(session);
        self.rt
            .block_on(
                self.backend
                    .resume_student(&operator, student_no, req.clone()),
            )
            .map_err(GatewayError::Problem)
    }

    fn withdraw_student(
        &mut self,
        session: &AccountSession,
        student_no: &str,
        req: &TransitionStudentRequest,
    ) -> Result<StudentSummary, GatewayError> {
        let operator = principal_from_session(session);
        self.rt
            .block_on(
                self.backend
                    .withdraw_student(&operator, student_no, req.clone()),
            )
            .map_err(GatewayError::Problem)
    }

    fn graduate_student(
        &mut self,
        session: &AccountSession,
        student_no: &str,
        req: &TransitionStudentRequest,
    ) -> Result<StudentSummary, GatewayError> {
        let operator = principal_from_session(session);
        self.rt
            .block_on(
                self.backend
                    .graduate_student(&operator, student_no, req.clone()),
            )
            .map_err(GatewayError::Problem)
    }

    fn reverse_enrollment(
        &mut self,
        _session: &AccountSession,
        _student_no: &str,
        _req: &ReverseEnrollmentRequest,
    ) -> Result<StudentSummary, GatewayError> {
        Err(planned_err("enrollment::cmd_reverse_enrollment"))
    }

    fn list_courses(&self, _session: &AccountSession) -> Result<Vec<CourseSummary>, GatewayError> {
        Err(planned_err("grading::qry_courses"))
    }

    fn create_course(
        &mut self,
        _session: &AccountSession,
        _req: &CreateCourseRequest,
    ) -> Result<CourseSummary, GatewayError> {
        Err(planned_err("grading::cmd_create_course"))
    }

    fn list_teachings(
        &self,
        _session: &AccountSession,
    ) -> Result<Vec<TeachingSummary>, GatewayError> {
        Err(planned_err("grading::qry_teachings"))
    }

    fn assign_teaching(
        &mut self,
        _session: &AccountSession,
        _req: &AssignTeachingRequest,
    ) -> Result<TeachingSummary, GatewayError> {
        Err(planned_err("grading::cmd_assign_teaching"))
    }

    fn list_grades(
        &self,
        _session: &AccountSession,
        _student_no: Option<&str>,
    ) -> Result<Vec<GradeEntrySummary>, GatewayError> {
        Err(planned_err("grading::qry_grades"))
    }

    fn save_draft_grade(
        &mut self,
        _session: &AccountSession,
        _req: &SaveDraftGradeRequest,
    ) -> Result<GradeEntrySummary, GatewayError> {
        Err(planned_err("grading::cmd_save_draft_grade"))
    }

    fn publish_grades(
        &mut self,
        _session: &AccountSession,
        _req: &PublishGradesRequest,
    ) -> Result<Vec<GradeEntrySummary>, GatewayError> {
        Err(planned_err("grading::cmd_publish_grades"))
    }

    fn amend_grade(
        &mut self,
        _session: &AccountSession,
        _grade_id: &str,
        _req: &AmendGradeRequest,
    ) -> Result<GradeEntrySummary, GatewayError> {
        Err(planned_err("grading::cmd_amend_grade"))
    }

    fn list_grade_amendments(
        &self,
        _session: &AccountSession,
        _grade_id: Option<&str>,
    ) -> Result<Vec<GradeAmendmentRecord>, GatewayError> {
        Err(planned_err("grading::qry_grade_amendments"))
    }

    fn list_accounts(&self, session: &AccountSession) -> Result<Vec<AccountSummary>, GatewayError> {
        let operator = principal_from_session(session);
        self.rt
            .block_on(self.backend.list_accounts(&operator))
            .map_err(GatewayError::Problem)
    }

    fn provision_account(
        &mut self,
        session: &AccountSession,
        req: &ProvisionAccountRequest,
    ) -> Result<AccountSummary, GatewayError> {
        let operator = principal_from_session(session);
        self.rt
            .block_on(self.backend.provision_account(&operator, req.clone()))
            .map_err(GatewayError::Problem)
    }

    fn bind_person_profile(
        &mut self,
        session: &AccountSession,
        account_id: &str,
        req: &BindPersonProfileRequest,
    ) -> Result<AccountSummary, GatewayError> {
        let operator = principal_from_session(session);
        self.rt
            .block_on(
                self.backend
                    .bind_person_profile(&operator, account_id, req.clone()),
            )
            .map_err(GatewayError::Problem)
    }

    fn change_account_role(
        &mut self,
        session: &AccountSession,
        account_id: &str,
        req: &ChangeAccountRoleRequest,
    ) -> Result<AccountSummary, GatewayError> {
        let operator = principal_from_session(session);
        self.rt
            .block_on(
                self.backend
                    .change_account_role(&operator, account_id, req.clone()),
            )
            .map_err(GatewayError::Problem)
    }

    fn disable_account(
        &mut self,
        session: &AccountSession,
        account_id: &str,
        req: &DisableAccountRequest,
    ) -> Result<AccountSummary, GatewayError> {
        let operator = principal_from_session(session);
        self.rt
            .block_on(
                self.backend
                    .disable_account(&operator, account_id, req.clone()),
            )
            .map_err(GatewayError::Problem)
    }
}
