use student_management_contracts::{
    AccountSession, AccountSummary, AmendGradeRequest, AssignTeachingRequest,
    BindPersonProfileRequest, ChangeAccountRoleRequest, CourseSummary, CreateCourseRequest,
    CreateHomeroomRequest, DisableAccountRequest, EnrollStudentRequest, EnrollmentStatus,
    GradeAmendmentRecord, GradeEntrySummary, HomeroomSummary, OperatorPrincipal,
    ProvisionAccountRequest, PublishGradesRequest, ReverseEnrollmentRequest, SaveDraftGradeRequest,
    SignInRequest, StudentDetailPayload, StudentSummary, TeachingSummary, TransferHomeroomRequest,
    TransitionStudentRequest, UpdateHomeroomCapacityRequest,
};
use student_management_runtime::{
    AppState, dispatch_outbox, enrollment, ensure_bootstrap_admin, grading, identity,
    open_app_state, open_local_admin_session, resolve_session,
};

use crate::contract::{Gateway, GatewayError, GatewayFuture};

pub struct LocalGatewayHandle {
    state: AppState,
}

impl LocalGatewayHandle {
    pub async fn open(database_url: &str) -> Result<Self, GatewayError> {
        let state = open_app_state(database_url)
            .await
            .map_err(GatewayError::Problem)?;
        Ok(Self { state })
    }

    pub async fn ensure_bootstrap_admin(
        &self,
        username: &str,
        display_name: &str,
        password: &str,
    ) -> Result<(), GatewayError> {
        ensure_bootstrap_admin(&self.state, username, display_name, password)
            .await
            .map_err(GatewayError::Problem)
    }

    pub async fn resolve_session(&self, token: &str) -> Result<AccountSession, GatewayError> {
        resolve_session(&self.state, token)
            .await
            .map_err(GatewayError::Problem)
    }

    pub async fn open_local_admin_session(&self) -> Result<AccountSession, GatewayError> {
        open_local_admin_session(&self.state)
            .await
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
    fn sign_in<'a>(
        &'a self,
        req: &'a SignInRequest,
    ) -> GatewayFuture<'a, Result<AccountSession, GatewayError>> {
        Box::pin(async move {
            identity::cmd_sign_in::handler::execute(&self.state, req.clone())
                .await
                .map_err(GatewayError::Problem)
        })
    }

    fn list_homerooms(&self) -> GatewayFuture<'_, Result<Vec<HomeroomSummary>, GatewayError>> {
        Box::pin(async move {
            enrollment::qry_homerooms::handler::execute(&self.state)
                .await
                .map_err(GatewayError::Problem)
        })
    }

    fn create_homeroom<'a>(
        &'a self,
        session: &'a AccountSession,
        req: &'a CreateHomeroomRequest,
    ) -> GatewayFuture<'a, Result<HomeroomSummary, GatewayError>> {
        Box::pin(async move {
            let operator = principal_from_session(session);
            enrollment::cmd_create_homeroom::handler::execute(&self.state, &operator, req.clone())
                .await
                .map_err(GatewayError::Problem)
        })
    }

    fn update_homeroom_capacity<'a>(
        &'a self,
        session: &'a AccountSession,
        homeroom_id: &'a str,
        req: &'a UpdateHomeroomCapacityRequest,
    ) -> GatewayFuture<'a, Result<HomeroomSummary, GatewayError>> {
        Box::pin(async move {
            let operator = principal_from_session(session);
            enrollment::cmd_update_homeroom_capacity::handler::execute(
                &self.state,
                &operator,
                homeroom_id,
                req.clone(),
            )
            .await
            .map_err(GatewayError::Problem)
        })
    }

    fn list_students<'a>(
        &'a self,
        session: &'a AccountSession,
        query: &'a str,
        status_filter: Option<EnrollmentStatus>,
    ) -> GatewayFuture<'a, Result<Vec<StudentSummary>, GatewayError>> {
        Box::pin(async move {
            let operator = principal_from_session(session);
            enrollment::qry_students::handler::execute(&self.state, &operator, query, status_filter)
                .await
                .map_err(GatewayError::Problem)
        })
    }

    fn student_detail<'a>(
        &'a self,
        session: &'a AccountSession,
        student_no: &'a str,
    ) -> GatewayFuture<'a, Result<StudentDetailPayload, GatewayError>> {
        Box::pin(async move {
            let operator = principal_from_session(session);
            enrollment::qry_student_detail::handler::execute(&self.state, &operator, student_no)
                .await
                .map_err(GatewayError::Problem)
        })
    }

    fn enroll_student<'a>(
        &'a self,
        session: &'a AccountSession,
        req: &'a EnrollStudentRequest,
    ) -> GatewayFuture<'a, Result<StudentSummary, GatewayError>> {
        Box::pin(async move {
            let operator = principal_from_session(session);
            enrollment::cmd_enroll_student::handler::execute(&self.state, &operator, req.clone())
                .await
                .map_err(GatewayError::Problem)
        })
    }

    fn transfer_homeroom<'a>(
        &'a self,
        session: &'a AccountSession,
        student_no: &'a str,
        req: &'a TransferHomeroomRequest,
    ) -> GatewayFuture<'a, Result<StudentSummary, GatewayError>> {
        Box::pin(async move {
            let operator = principal_from_session(session);
            enrollment::cmd_transfer_homeroom::handler::execute(
                &self.state,
                &operator,
                student_no,
                req.clone(),
            )
            .await
            .map_err(GatewayError::Problem)
        })
    }

    fn suspend_student<'a>(
        &'a self,
        session: &'a AccountSession,
        student_no: &'a str,
        req: &'a TransitionStudentRequest,
    ) -> GatewayFuture<'a, Result<StudentSummary, GatewayError>> {
        Box::pin(async move {
            let operator = principal_from_session(session);
            enrollment::cmd_suspend_student::handler::execute(
                &self.state,
                &operator,
                student_no,
                req.clone(),
            )
            .await
            .map_err(GatewayError::Problem)
        })
    }

    fn resume_student<'a>(
        &'a self,
        session: &'a AccountSession,
        student_no: &'a str,
        req: &'a TransitionStudentRequest,
    ) -> GatewayFuture<'a, Result<StudentSummary, GatewayError>> {
        Box::pin(async move {
            let operator = principal_from_session(session);
            enrollment::cmd_resume_student::handler::execute(
                &self.state,
                &operator,
                student_no,
                req.clone(),
            )
            .await
            .map_err(GatewayError::Problem)
        })
    }

    fn withdraw_student<'a>(
        &'a self,
        session: &'a AccountSession,
        student_no: &'a str,
        req: &'a TransitionStudentRequest,
    ) -> GatewayFuture<'a, Result<StudentSummary, GatewayError>> {
        Box::pin(async move {
            let operator = principal_from_session(session);
            let summary = enrollment::cmd_withdraw_student::handler::execute(
                &self.state,
                &operator,
                student_no,
                req.clone(),
            )
            .await
            .map_err(GatewayError::Problem)?;
            let _ = dispatch_outbox(&self.state)
                .await
                .map_err(GatewayError::Problem)?;
            Ok(summary)
        })
    }

    fn graduate_student<'a>(
        &'a self,
        session: &'a AccountSession,
        student_no: &'a str,
        req: &'a TransitionStudentRequest,
    ) -> GatewayFuture<'a, Result<StudentSummary, GatewayError>> {
        Box::pin(async move {
            let operator = principal_from_session(session);
            enrollment::cmd_graduate_student::handler::execute(
                &self.state,
                &operator,
                student_no,
                req.clone(),
            )
            .await
            .map_err(GatewayError::Problem)
        })
    }

    fn reverse_enrollment<'a>(
        &'a self,
        session: &'a AccountSession,
        student_no: &'a str,
        req: &'a ReverseEnrollmentRequest,
    ) -> GatewayFuture<'a, Result<StudentSummary, GatewayError>> {
        Box::pin(async move {
            let operator = principal_from_session(session);
            enrollment::cmd_reverse_enrollment::handler::execute(
                &self.state,
                &operator,
                student_no,
                req.clone(),
            )
            .await
            .map_err(GatewayError::Problem)
        })
    }

    fn list_courses<'a>(
        &'a self,
        session: &'a AccountSession,
    ) -> GatewayFuture<'a, Result<Vec<CourseSummary>, GatewayError>> {
        Box::pin(async move {
            let operator = principal_from_session(session);
            grading::qry_courses::handler::execute(&self.state, &operator)
                .await
                .map_err(GatewayError::Problem)
        })
    }

    fn create_course<'a>(
        &'a self,
        session: &'a AccountSession,
        req: &'a CreateCourseRequest,
    ) -> GatewayFuture<'a, Result<CourseSummary, GatewayError>> {
        Box::pin(async move {
            let operator = principal_from_session(session);
            grading::cmd_create_course::handler::execute(&self.state, &operator, req.clone())
                .await
                .map_err(GatewayError::Problem)
        })
    }

    fn list_teachings<'a>(
        &'a self,
        session: &'a AccountSession,
    ) -> GatewayFuture<'a, Result<Vec<TeachingSummary>, GatewayError>> {
        Box::pin(async move {
            let operator = principal_from_session(session);
            grading::qry_teachings::handler::execute(&self.state, &operator)
                .await
                .map_err(GatewayError::Problem)
        })
    }

    fn assign_teaching<'a>(
        &'a self,
        session: &'a AccountSession,
        req: &'a AssignTeachingRequest,
    ) -> GatewayFuture<'a, Result<TeachingSummary, GatewayError>> {
        Box::pin(async move {
            let operator = principal_from_session(session);
            grading::cmd_assign_teaching::handler::execute(&self.state, &operator, req.clone())
                .await
                .map_err(GatewayError::Problem)
        })
    }

    fn list_grades<'a>(
        &'a self,
        session: &'a AccountSession,
        student_no: Option<&'a str>,
    ) -> GatewayFuture<'a, Result<Vec<GradeEntrySummary>, GatewayError>> {
        Box::pin(async move {
            let operator = principal_from_session(session);
            grading::qry_grades::handler::execute(&self.state, &operator, student_no)
                .await
                .map_err(GatewayError::Problem)
        })
    }

    fn save_draft_grade<'a>(
        &'a self,
        session: &'a AccountSession,
        req: &'a SaveDraftGradeRequest,
    ) -> GatewayFuture<'a, Result<GradeEntrySummary, GatewayError>> {
        Box::pin(async move {
            let operator = principal_from_session(session);
            grading::cmd_save_draft_grade::handler::execute(&self.state, &operator, req.clone())
                .await
                .map_err(GatewayError::Problem)
        })
    }

    fn publish_grades<'a>(
        &'a self,
        session: &'a AccountSession,
        req: &'a PublishGradesRequest,
    ) -> GatewayFuture<'a, Result<Vec<GradeEntrySummary>, GatewayError>> {
        Box::pin(async move {
            let operator = principal_from_session(session);
            grading::cmd_publish_grades::handler::execute(&self.state, &operator, req.clone())
                .await
                .map_err(GatewayError::Problem)
        })
    }

    fn amend_grade<'a>(
        &'a self,
        session: &'a AccountSession,
        grade_id: &'a str,
        req: &'a AmendGradeRequest,
    ) -> GatewayFuture<'a, Result<GradeEntrySummary, GatewayError>> {
        Box::pin(async move {
            let operator = principal_from_session(session);
            grading::cmd_amend_grade::handler::execute(
                &self.state,
                &operator,
                grade_id,
                req.clone(),
            )
            .await
            .map_err(GatewayError::Problem)
        })
    }

    fn list_grade_amendments<'a>(
        &'a self,
        session: &'a AccountSession,
        grade_id: Option<&'a str>,
    ) -> GatewayFuture<'a, Result<Vec<GradeAmendmentRecord>, GatewayError>> {
        Box::pin(async move {
            let operator = principal_from_session(session);
            grading::qry_grade_amendments::handler::execute(&self.state, &operator, grade_id)
                .await
                .map_err(GatewayError::Problem)
        })
    }

    fn list_accounts<'a>(
        &'a self,
        session: &'a AccountSession,
    ) -> GatewayFuture<'a, Result<Vec<AccountSummary>, GatewayError>> {
        Box::pin(async move {
            let operator = principal_from_session(session);
            identity::qry_accounts::handler::execute(&self.state, &operator)
                .await
                .map_err(GatewayError::Problem)
        })
    }

    fn provision_account<'a>(
        &'a self,
        session: &'a AccountSession,
        req: &'a ProvisionAccountRequest,
    ) -> GatewayFuture<'a, Result<AccountSummary, GatewayError>> {
        Box::pin(async move {
            let operator = principal_from_session(session);
            identity::cmd_provision_account::handler::execute(&self.state, &operator, req.clone())
                .await
                .map_err(GatewayError::Problem)
        })
    }

    fn bind_person_profile<'a>(
        &'a self,
        session: &'a AccountSession,
        account_id: &'a str,
        req: &'a BindPersonProfileRequest,
    ) -> GatewayFuture<'a, Result<AccountSummary, GatewayError>> {
        Box::pin(async move {
            let operator = principal_from_session(session);
            identity::cmd_bind_person_profile::handler::execute(
                &self.state,
                &operator,
                account_id,
                req.clone(),
            )
            .await
            .map_err(GatewayError::Problem)
        })
    }

    fn change_account_role<'a>(
        &'a self,
        session: &'a AccountSession,
        account_id: &'a str,
        req: &'a ChangeAccountRoleRequest,
    ) -> GatewayFuture<'a, Result<AccountSummary, GatewayError>> {
        Box::pin(async move {
            let operator = principal_from_session(session);
            identity::cmd_change_account_role::handler::execute(
                &self.state,
                &operator,
                account_id,
                req.clone(),
            )
            .await
            .map_err(GatewayError::Problem)
        })
    }

    fn disable_account<'a>(
        &'a self,
        session: &'a AccountSession,
        account_id: &'a str,
        req: &'a DisableAccountRequest,
    ) -> GatewayFuture<'a, Result<AccountSummary, GatewayError>> {
        Box::pin(async move {
            let operator = principal_from_session(session);
            identity::cmd_disable_account::handler::execute(
                &self.state,
                &operator,
                account_id,
                req.clone(),
            )
            .await
            .map_err(GatewayError::Problem)
        })
    }
}
