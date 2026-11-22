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
use student_management_runtime::{
    AppState, dispatch_outbox, enrollment, ensure_bootstrap_admin, grading, identity,
    open_app_state, resolve_session,
};
use tokio::runtime::Runtime;

use crate::contract::{Gateway, GatewayError};

pub struct LocalGatewayHandle {
    rt: Arc<Runtime>,
    state: AppState,
}

impl LocalGatewayHandle {
    pub fn open(database_url: &str) -> Result<Self, GatewayError> {
        let rt = Runtime::new().map_err(|err| GatewayError::Transport(err.to_string()))?;
        let state = rt
            .block_on(open_app_state(database_url))
            .map_err(GatewayError::Problem)?;
        Ok(Self {
            rt: Arc::new(rt),
            state,
        })
    }

    pub fn ensure_bootstrap_admin(
        &self,
        username: &str,
        display_name: &str,
        password: &str,
    ) -> Result<(), GatewayError> {
        self.rt
            .block_on(ensure_bootstrap_admin(
                &self.state,
                username,
                display_name,
                password,
            ))
            .map_err(GatewayError::Problem)
    }

    pub fn resolve_session(&self, token: &str) -> Result<AccountSession, GatewayError> {
        self.rt
            .block_on(resolve_session(&self.state, token))
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

impl Gateway for LocalGatewayHandle {
    fn sign_in(&self, req: &SignInRequest) -> Result<AccountSession, GatewayError> {
        self.rt
            .block_on(identity::cmd_sign_in::handler::execute(
                &self.state,
                req.clone(),
            ))
            .map_err(GatewayError::Problem)
    }

    fn switch_role(&self, _role: Role) -> Result<AccountSession, GatewayError> {
        Err(GatewayError::Problem(ProblemDetails::not_implemented(
            "identity::switch_role",
        )))
    }

    fn list_homerooms(&self) -> Result<Vec<HomeroomSummary>, GatewayError> {
        self.rt
            .block_on(enrollment::qry_homerooms::handler::execute(&self.state))
            .map_err(GatewayError::Problem)
    }

    fn create_homeroom(
        &mut self,
        session: &AccountSession,
        req: &CreateHomeroomRequest,
    ) -> Result<HomeroomSummary, GatewayError> {
        let operator = principal_from_session(session);
        self.rt
            .block_on(enrollment::cmd_create_homeroom::handler::execute(
                &self.state,
                &operator,
                req.clone(),
            ))
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
            .block_on(enrollment::cmd_update_homeroom_capacity::handler::execute(
                &self.state,
                &operator,
                homeroom_id,
                req.clone(),
            ))
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
            .block_on(enrollment::qry_students::handler::execute(
                &self.state,
                &operator,
                query,
                status_filter,
            ))
            .map_err(GatewayError::Problem)
    }

    fn student_detail(
        &self,
        session: &AccountSession,
        student_no: &str,
    ) -> Result<StudentDetailPayload, GatewayError> {
        let operator = principal_from_session(session);
        self.rt
            .block_on(enrollment::qry_student_detail::handler::execute(
                &self.state,
                &operator,
                student_no,
            ))
            .map_err(GatewayError::Problem)
    }

    fn enroll_student(
        &mut self,
        session: &AccountSession,
        req: &EnrollStudentRequest,
    ) -> Result<StudentSummary, GatewayError> {
        let operator = principal_from_session(session);
        self.rt
            .block_on(enrollment::cmd_enroll_student::handler::execute(
                &self.state,
                &operator,
                req.clone(),
            ))
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
            .block_on(enrollment::cmd_transfer_homeroom::handler::execute(
                &self.state,
                &operator,
                student_no,
                req.clone(),
            ))
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
            .block_on(enrollment::cmd_suspend_student::handler::execute(
                &self.state,
                &operator,
                student_no,
                req.clone(),
            ))
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
            .block_on(enrollment::cmd_resume_student::handler::execute(
                &self.state,
                &operator,
                student_no,
                req.clone(),
            ))
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
            .block_on(async {
                let summary = enrollment::cmd_withdraw_student::handler::execute(
                    &self.state,
                    &operator,
                    student_no,
                    req.clone(),
                )
                .await?;
                let _ = dispatch_outbox(&self.state).await?;
                Ok(summary)
            })
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
            .block_on(enrollment::cmd_graduate_student::handler::execute(
                &self.state,
                &operator,
                student_no,
                req.clone(),
            ))
            .map_err(GatewayError::Problem)
    }

    fn reverse_enrollment(
        &mut self,
        session: &AccountSession,
        student_no: &str,
        req: &ReverseEnrollmentRequest,
    ) -> Result<StudentSummary, GatewayError> {
        let operator = principal_from_session(session);
        self.rt
            .block_on(enrollment::cmd_reverse_enrollment::handler::execute(
                &self.state,
                &operator,
                student_no,
                req.clone(),
            ))
            .map_err(GatewayError::Problem)
    }

    fn list_courses(&self, session: &AccountSession) -> Result<Vec<CourseSummary>, GatewayError> {
        let operator = principal_from_session(session);
        self.rt
            .block_on(grading::qry_courses::handler::execute(
                &self.state,
                &operator,
            ))
            .map_err(GatewayError::Problem)
    }

    fn create_course(
        &mut self,
        session: &AccountSession,
        req: &CreateCourseRequest,
    ) -> Result<CourseSummary, GatewayError> {
        let operator = principal_from_session(session);
        self.rt
            .block_on(grading::cmd_create_course::handler::execute(
                &self.state,
                &operator,
                req.clone(),
            ))
            .map_err(GatewayError::Problem)
    }

    fn list_teachings(
        &self,
        session: &AccountSession,
    ) -> Result<Vec<TeachingSummary>, GatewayError> {
        let operator = principal_from_session(session);
        self.rt
            .block_on(grading::qry_teachings::handler::execute(
                &self.state,
                &operator,
            ))
            .map_err(GatewayError::Problem)
    }

    fn assign_teaching(
        &mut self,
        session: &AccountSession,
        req: &AssignTeachingRequest,
    ) -> Result<TeachingSummary, GatewayError> {
        let operator = principal_from_session(session);
        self.rt
            .block_on(grading::cmd_assign_teaching::handler::execute(
                &self.state,
                &operator,
                req.clone(),
            ))
            .map_err(GatewayError::Problem)
    }

    fn list_grades(
        &self,
        session: &AccountSession,
        student_no: Option<&str>,
    ) -> Result<Vec<GradeEntrySummary>, GatewayError> {
        let operator = principal_from_session(session);
        self.rt
            .block_on(grading::qry_grades::handler::execute(
                &self.state,
                &operator,
                student_no,
            ))
            .map_err(GatewayError::Problem)
    }

    fn save_draft_grade(
        &mut self,
        session: &AccountSession,
        req: &SaveDraftGradeRequest,
    ) -> Result<GradeEntrySummary, GatewayError> {
        let operator = principal_from_session(session);
        self.rt
            .block_on(grading::cmd_save_draft_grade::handler::execute(
                &self.state,
                &operator,
                req.clone(),
            ))
            .map_err(GatewayError::Problem)
    }

    fn publish_grades(
        &mut self,
        session: &AccountSession,
        req: &PublishGradesRequest,
    ) -> Result<Vec<GradeEntrySummary>, GatewayError> {
        let operator = principal_from_session(session);
        self.rt
            .block_on(grading::cmd_publish_grades::handler::execute(
                &self.state,
                &operator,
                req.clone(),
            ))
            .map_err(GatewayError::Problem)
    }

    fn amend_grade(
        &mut self,
        session: &AccountSession,
        grade_id: &str,
        req: &AmendGradeRequest,
    ) -> Result<GradeEntrySummary, GatewayError> {
        let operator = principal_from_session(session);
        self.rt
            .block_on(grading::cmd_amend_grade::handler::execute(
                &self.state,
                &operator,
                grade_id,
                req.clone(),
            ))
            .map_err(GatewayError::Problem)
    }

    fn list_grade_amendments(
        &self,
        session: &AccountSession,
        grade_id: Option<&str>,
    ) -> Result<Vec<GradeAmendmentRecord>, GatewayError> {
        let operator = principal_from_session(session);
        self.rt
            .block_on(grading::qry_grade_amendments::handler::execute(
                &self.state,
                &operator,
                grade_id,
            ))
            .map_err(GatewayError::Problem)
    }

    fn list_accounts(&self, session: &AccountSession) -> Result<Vec<AccountSummary>, GatewayError> {
        let operator = principal_from_session(session);
        self.rt
            .block_on(identity::qry_accounts::handler::execute(
                &self.state,
                &operator,
            ))
            .map_err(GatewayError::Problem)
    }

    fn provision_account(
        &mut self,
        session: &AccountSession,
        req: &ProvisionAccountRequest,
    ) -> Result<AccountSummary, GatewayError> {
        let operator = principal_from_session(session);
        self.rt
            .block_on(identity::cmd_provision_account::handler::execute(
                &self.state,
                &operator,
                req.clone(),
            ))
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
            .block_on(identity::cmd_bind_person_profile::handler::execute(
                &self.state,
                &operator,
                account_id,
                req.clone(),
            ))
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
            .block_on(identity::cmd_change_account_role::handler::execute(
                &self.state,
                &operator,
                account_id,
                req.clone(),
            ))
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
            .block_on(identity::cmd_disable_account::handler::execute(
                &self.state,
                &operator,
                account_id,
                req.clone(),
            ))
            .map_err(GatewayError::Problem)
    }
}
