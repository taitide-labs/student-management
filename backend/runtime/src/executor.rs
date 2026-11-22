use sqlx::Row;
use student_management_contracts::{
    AccountSession, AccountSummary, AssignTeachingRequest, BindPersonProfileRequest,
    ChangeAccountRoleRequest, CourseSummary, CreateCourseRequest, CreateHomeroomRequest,
    DisableAccountRequest, EnrollStudentRequest, EnrollmentStatus, GradeEntrySummary,
    HomeroomSummary, OperatorPrincipal, ProblemDetails, ProvisionAccountRequest,
    ReverseEnrollmentRequest, SignInRequest, StudentDetailPayload, StudentSummary, TeachingSummary,
    TransferHomeroomRequest, TransitionStudentRequest, UpdateHomeroomCapacityRequest,
};
use student_management_infra::{AppState, InfraError, hash_password, infra_error_to_problem};

use crate::bootstrap::build_app_state;

#[derive(Clone)]
pub struct LocalBackend {
    state: AppState,
}

impl LocalBackend {
    pub async fn open(database_url: &str) -> Result<Self, ProblemDetails> {
        let state = build_app_state(database_url)
            .await
            .map_err(|err| infra_error_to_problem(&err))?;
        Ok(Self { state })
    }

    #[must_use]
    pub const fn new(state: AppState) -> Self {
        Self { state }
    }

    #[must_use]
    pub const fn state(&self) -> &AppState {
        &self.state
    }

    pub async fn ensure_bootstrap_admin(
        &self,
        username: &str,
        display_name: &str,
        password: &str,
    ) -> Result<(), ProblemDetails> {
        let count_row = sqlx::query("SELECT COUNT(*) AS cnt FROM accounts")
            .fetch_one(&self.state.pool)
            .await
            .map_err(InfraError::Database)
            .map_err(|err| infra_error_to_problem(&err))?;
        let count: i64 = count_row
            .try_get("cnt")
            .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?;
        if count > 0 {
            return Ok(());
        }

        let hash = hash_password(password).map_err(|err| infra_error_to_problem(&err))?;
        let now = self.state.clock.now_rfc3339();
        let account_id = self.state.ids.next_id("acc");
        sqlx::query(
            "INSERT INTO accounts (account_id, username, display_name, password_hash, role, status, version, created_at, updated_at) VALUES (?, ?, ?, ?, 'admin', 'active', 1, ?, ?)",
        )
        .bind(account_id)
        .bind(username)
        .bind(display_name)
        .bind(hash)
        .bind(&now)
        .bind(&now)
        .execute(&self.state.pool)
        .await
        .map_err(InfraError::Database)
        .map_err(|err| infra_error_to_problem(&err))?;
        Ok(())
    }

    pub async fn sign_in(&self, req: SignInRequest) -> Result<AccountSession, ProblemDetails> {
        student_management_identity_features::cmd_sign_in::handler::execute(&self.state, req).await
    }

    pub async fn list_accounts(
        &self,
        operator: &OperatorPrincipal,
    ) -> Result<Vec<AccountSummary>, ProblemDetails> {
        student_management_identity_features::qry_accounts::handler::execute(&self.state, operator)
            .await
    }

    pub async fn provision_account(
        &self,
        operator: &OperatorPrincipal,
        req: ProvisionAccountRequest,
    ) -> Result<AccountSummary, ProblemDetails> {
        student_management_identity_features::cmd_provision_account::handler::execute(
            &self.state,
            operator,
            req,
        )
        .await
    }

    pub async fn bind_person_profile(
        &self,
        operator: &OperatorPrincipal,
        account_id: &str,
        req: BindPersonProfileRequest,
    ) -> Result<AccountSummary, ProblemDetails> {
        student_management_identity_features::cmd_bind_person_profile::handler::execute(
            &self.state,
            operator,
            account_id,
            req,
        )
        .await
    }

    pub async fn change_account_role(
        &self,
        operator: &OperatorPrincipal,
        account_id: &str,
        req: ChangeAccountRoleRequest,
    ) -> Result<AccountSummary, ProblemDetails> {
        student_management_identity_features::cmd_change_account_role::handler::execute(
            &self.state,
            operator,
            account_id,
            req,
        )
        .await
    }

    pub async fn disable_account(
        &self,
        operator: &OperatorPrincipal,
        account_id: &str,
        req: DisableAccountRequest,
    ) -> Result<AccountSummary, ProblemDetails> {
        student_management_identity_features::cmd_disable_account::handler::execute(
            &self.state,
            operator,
            account_id,
            req,
        )
        .await
    }

    pub async fn list_homerooms(&self) -> Result<Vec<HomeroomSummary>, ProblemDetails> {
        student_management_enrollment_features::qry_homerooms::handler::execute(&self.state).await
    }

    pub async fn create_homeroom(
        &self,
        operator: &OperatorPrincipal,
        req: CreateHomeroomRequest,
    ) -> Result<HomeroomSummary, ProblemDetails> {
        student_management_enrollment_features::cmd_create_homeroom::handler::execute(
            &self.state,
            operator,
            req,
        )
        .await
    }

    pub async fn update_homeroom_capacity(
        &self,
        operator: &OperatorPrincipal,
        homeroom_id: &str,
        req: UpdateHomeroomCapacityRequest,
    ) -> Result<HomeroomSummary, ProblemDetails> {
        student_management_enrollment_features::cmd_update_homeroom_capacity::handler::execute(
            &self.state,
            operator,
            homeroom_id,
            req,
        )
        .await
    }

    pub async fn list_students(
        &self,
        operator: &OperatorPrincipal,
        query: &str,
        status_filter: Option<EnrollmentStatus>,
    ) -> Result<Vec<StudentSummary>, ProblemDetails> {
        student_management_enrollment_features::qry_students::handler::execute(
            &self.state,
            operator,
            query,
            status_filter,
        )
        .await
    }

    pub async fn student_detail(
        &self,
        operator: &OperatorPrincipal,
        student_no: &str,
    ) -> Result<StudentDetailPayload, ProblemDetails> {
        student_management_enrollment_features::qry_student_detail::handler::execute(
            &self.state,
            operator,
            student_no,
        )
        .await
    }

    pub async fn enroll_student(
        &self,
        operator: &OperatorPrincipal,
        req: EnrollStudentRequest,
    ) -> Result<StudentSummary, ProblemDetails> {
        student_management_enrollment_features::cmd_enroll_student::handler::execute(
            &self.state,
            operator,
            req,
        )
        .await
    }

    pub async fn transfer_homeroom(
        &self,
        operator: &OperatorPrincipal,
        student_no: &str,
        req: TransferHomeroomRequest,
    ) -> Result<StudentSummary, ProblemDetails> {
        student_management_enrollment_features::cmd_transfer_homeroom::handler::execute(
            &self.state,
            operator,
            student_no,
            req,
        )
        .await
    }

    pub async fn suspend_student(
        &self,
        operator: &OperatorPrincipal,
        student_no: &str,
        req: TransitionStudentRequest,
    ) -> Result<StudentSummary, ProblemDetails> {
        student_management_enrollment_features::cmd_suspend_student::handler::execute(
            &self.state,
            operator,
            student_no,
            req,
        )
        .await
    }

    pub async fn resume_student(
        &self,
        operator: &OperatorPrincipal,
        student_no: &str,
        req: TransitionStudentRequest,
    ) -> Result<StudentSummary, ProblemDetails> {
        student_management_enrollment_features::cmd_resume_student::handler::execute(
            &self.state,
            operator,
            student_no,
            req,
        )
        .await
    }

    pub async fn withdraw_student(
        &self,
        operator: &OperatorPrincipal,
        student_no: &str,
        req: TransitionStudentRequest,
    ) -> Result<StudentSummary, ProblemDetails> {
        student_management_enrollment_features::cmd_withdraw_student::handler::execute(
            &self.state,
            operator,
            student_no,
            req,
        )
        .await
    }

    pub async fn graduate_student(
        &self,
        operator: &OperatorPrincipal,
        student_no: &str,
        req: TransitionStudentRequest,
    ) -> Result<StudentSummary, ProblemDetails> {
        student_management_enrollment_features::cmd_graduate_student::handler::execute(
            &self.state,
            operator,
            student_no,
            req,
        )
        .await
    }

    pub async fn reverse_enrollment(
        &self,
        operator: &OperatorPrincipal,
        student_no: &str,
        req: ReverseEnrollmentRequest,
    ) -> Result<StudentSummary, ProblemDetails> {
        student_management_enrollment_features::cmd_reverse_enrollment::handler::execute(
            &self.state,
            operator,
            student_no,
            req,
        )
        .await
    }

    pub async fn list_courses(
        &self,
        operator: &OperatorPrincipal,
    ) -> Result<Vec<CourseSummary>, ProblemDetails> {
        student_management_grading_features::qry_courses::handler::execute(&self.state, operator)
            .await
    }

    pub async fn create_course(
        &self,
        operator: &OperatorPrincipal,
        req: CreateCourseRequest,
    ) -> Result<CourseSummary, ProblemDetails> {
        student_management_grading_features::cmd_create_course::handler::execute(
            &self.state,
            operator,
            req,
        )
        .await
    }

    pub async fn list_teachings(
        &self,
        operator: &OperatorPrincipal,
    ) -> Result<Vec<TeachingSummary>, ProblemDetails> {
        student_management_grading_features::qry_teachings::handler::execute(&self.state, operator)
            .await
    }

    pub async fn assign_teaching(
        &self,
        operator: &OperatorPrincipal,
        req: AssignTeachingRequest,
    ) -> Result<TeachingSummary, ProblemDetails> {
        student_management_grading_features::cmd_assign_teaching::handler::execute(
            &self.state,
            operator,
            req,
        )
        .await
    }

    pub async fn list_grades(
        &self,
        operator: &OperatorPrincipal,
        student_no: Option<&str>,
    ) -> Result<Vec<GradeEntrySummary>, ProblemDetails> {
        student_management_grading_features::qry_grades::handler::execute(
            &self.state,
            operator,
            student_no,
        )
        .await
    }
}
