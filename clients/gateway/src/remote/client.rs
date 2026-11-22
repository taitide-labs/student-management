use serde::{Serialize, de::DeserializeOwned};
use student_management_contracts::{
    AccountSession, AccountSummary, AmendGradeRequest, AssignTeachingRequest,
    BindPersonProfileRequest, ChangeAccountRoleRequest, CourseSummary, CreateCourseRequest,
    CreateHomeroomRequest, DisableAccountRequest, EnrollStudentRequest, EnrollmentStatus,
    GradeAmendmentRecord, GradeEntrySummary, HomeroomSummary, ProblemDetails,
    ProvisionAccountRequest, PublishGradesRequest, ReverseEnrollmentRequest, SaveDraftGradeRequest,
    SignInRequest, StudentDetailPayload, StudentSummary, TeachingSummary, TransferHomeroomRequest,
    TransitionStudentRequest, UpdateHomeroomCapacityRequest,
};

use crate::{
    contract::{Gateway, GatewayError, GatewayFuture},
    remote::transport::RemoteTransport,
};

#[derive(Debug, Clone)]
pub struct RemoteGatewayConfig {
    pub base_url: String,
}

#[derive(Clone)]
pub struct GenericRemoteGateway<T> {
    base_url: String,
    transport: T,
}

#[cfg(all(feature = "remote", not(target_arch = "wasm32")))]
pub type RemoteGateway = GenericRemoteGateway<super::transport::UreqTransport>;

#[cfg(feature = "nsh")]
pub type NshRemoteGateway = GenericRemoteGateway<super::transport::NshTransport>;

impl<T: Default> GenericRemoteGateway<T> {
    #[must_use]
    pub fn new(config: &RemoteGatewayConfig) -> Self {
        Self::with_transport(config, T::default())
    }
}

impl<T> GenericRemoteGateway<T> {
    #[must_use]
    pub fn with_transport(config: &RemoteGatewayConfig, transport: T) -> Self {
        Self {
            base_url: config.base_url.trim_end_matches('/').to_owned(),
            transport,
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }
}

impl<T: RemoteTransport> GenericRemoteGateway<T> {
    fn get_json<'a, R: DeserializeOwned + 'a>(
        &'a self,
        path: String,
        token: Option<&'a str>,
    ) -> GatewayFuture<'a, Result<R, GatewayError>> {
        Box::pin(async move {
            let url = self.url(&path);
            let (status, raw) = self.transport.send("GET", &url, token, None).await?;
            decode_response(status, &raw)
        })
    }

    fn send_json<'a, B: Serialize, R: DeserializeOwned + 'a>(
        &'a self,
        method: &'a str,
        path: String,
        token: Option<&'a str>,
        body: &B,
    ) -> GatewayFuture<'a, Result<R, GatewayError>> {
        let encoded = serde_json::to_vec(body)
            .map_err(|err| GatewayError::Transport(format!("请求序列化失败：{err}")));
        Box::pin(async move {
            let payload = encoded?;
            let url = self.url(&path);
            let (status, raw) = self
                .transport
                .send(method, &url, token, Some(payload))
                .await?;
            decode_response(status, &raw)
        })
    }
}

fn decode_response<T: DeserializeOwned>(status: u16, raw: &[u8]) -> Result<T, GatewayError> {
    if (200..300).contains(&status) {
        return serde_json::from_slice(raw)
            .map_err(|err| GatewayError::Transport(format!("响应反序列化失败：{err}")));
    }

    if let Ok(problem) = serde_json::from_slice::<ProblemDetails>(raw) {
        return Err(GatewayError::Problem(problem));
    }

    let detail = String::from_utf8_lossy(raw).into_owned();
    Err(GatewayError::Problem(ProblemDetails {
        problem_type: "https://student-management.local/problems/http".to_owned(),
        title: format!("HTTP {status}"),
        status,
        detail: if detail.is_empty() {
            format!("HTTP {status}")
        } else {
            detail
        },
        code: "http_error".to_owned(),
    }))
}

fn encode_query_component(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for byte in raw.as_bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            out.push(*byte as char);
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

fn status_query_value(status: EnrollmentStatus) -> &'static str {
    match status {
        EnrollmentStatus::Active => "active",
        EnrollmentStatus::Suspended => "suspended",
        EnrollmentStatus::Withdrawn => "withdrawn",
        EnrollmentStatus::Graduated => "graduated",
    }
}

impl<T: RemoteTransport> Gateway for GenericRemoteGateway<T> {
    fn sign_in<'a>(
        &'a self,
        req: &'a SignInRequest,
    ) -> GatewayFuture<'a, Result<AccountSession, GatewayError>> {
        self.send_json("POST", "/api/v1/auth/sign-in".to_owned(), None, req)
    }

    fn list_homerooms(&self) -> GatewayFuture<'_, Result<Vec<HomeroomSummary>, GatewayError>> {
        self.get_json("/api/v1/homerooms".to_owned(), None)
    }

    fn create_homeroom<'a>(
        &'a self,
        session: &'a AccountSession,
        req: &'a CreateHomeroomRequest,
    ) -> GatewayFuture<'a, Result<HomeroomSummary, GatewayError>> {
        self.send_json(
            "POST",
            "/api/v1/homerooms".to_owned(),
            Some(&session.access_token),
            req,
        )
    }

    fn update_homeroom_capacity<'a>(
        &'a self,
        session: &'a AccountSession,
        homeroom_id: &'a str,
        req: &'a UpdateHomeroomCapacityRequest,
    ) -> GatewayFuture<'a, Result<HomeroomSummary, GatewayError>> {
        let path = format!(
            "/api/v1/homerooms/{}/capacity",
            encode_query_component(homeroom_id)
        );
        self.send_json("PATCH", path, Some(&session.access_token), req)
    }

    fn list_students<'a>(
        &'a self,
        session: &'a AccountSession,
        query: &'a str,
        status_filter: Option<EnrollmentStatus>,
    ) -> GatewayFuture<'a, Result<Vec<StudentSummary>, GatewayError>> {
        let mut params = Vec::new();
        if !query.trim().is_empty() {
            params.push(format!("q={}", encode_query_component(query.trim())));
        }
        if let Some(status) = status_filter {
            params.push(format!("status={}", status_query_value(status)));
        }
        let path = if params.is_empty() {
            "/api/v1/students".to_owned()
        } else {
            format!("/api/v1/students?{}", params.join("&"))
        };
        self.get_json(path, Some(&session.access_token))
    }

    fn student_detail<'a>(
        &'a self,
        session: &'a AccountSession,
        student_no: &'a str,
    ) -> GatewayFuture<'a, Result<StudentDetailPayload, GatewayError>> {
        let path = format!("/api/v1/students/{}", encode_query_component(student_no));
        self.get_json(path, Some(&session.access_token))
    }

    fn enroll_student<'a>(
        &'a self,
        session: &'a AccountSession,
        req: &'a EnrollStudentRequest,
    ) -> GatewayFuture<'a, Result<StudentSummary, GatewayError>> {
        self.send_json(
            "POST",
            "/api/v1/students".to_owned(),
            Some(&session.access_token),
            req,
        )
    }

    fn transfer_homeroom<'a>(
        &'a self,
        session: &'a AccountSession,
        student_no: &'a str,
        req: &'a TransferHomeroomRequest,
    ) -> GatewayFuture<'a, Result<StudentSummary, GatewayError>> {
        let path = format!(
            "/api/v1/students/{}/transfer",
            encode_query_component(student_no)
        );
        self.send_json("POST", path, Some(&session.access_token), req)
    }

    fn suspend_student<'a>(
        &'a self,
        session: &'a AccountSession,
        student_no: &'a str,
        req: &'a TransitionStudentRequest,
    ) -> GatewayFuture<'a, Result<StudentSummary, GatewayError>> {
        let path = format!(
            "/api/v1/students/{}/suspend",
            encode_query_component(student_no)
        );
        self.send_json("POST", path, Some(&session.access_token), req)
    }

    fn resume_student<'a>(
        &'a self,
        session: &'a AccountSession,
        student_no: &'a str,
        req: &'a TransitionStudentRequest,
    ) -> GatewayFuture<'a, Result<StudentSummary, GatewayError>> {
        let path = format!(
            "/api/v1/students/{}/resume",
            encode_query_component(student_no)
        );
        self.send_json("POST", path, Some(&session.access_token), req)
    }

    fn withdraw_student<'a>(
        &'a self,
        session: &'a AccountSession,
        student_no: &'a str,
        req: &'a TransitionStudentRequest,
    ) -> GatewayFuture<'a, Result<StudentSummary, GatewayError>> {
        let path = format!(
            "/api/v1/students/{}/withdraw",
            encode_query_component(student_no)
        );
        self.send_json("POST", path, Some(&session.access_token), req)
    }

    fn graduate_student<'a>(
        &'a self,
        session: &'a AccountSession,
        student_no: &'a str,
        req: &'a TransitionStudentRequest,
    ) -> GatewayFuture<'a, Result<StudentSummary, GatewayError>> {
        let path = format!(
            "/api/v1/students/{}/graduate",
            encode_query_component(student_no)
        );
        self.send_json("POST", path, Some(&session.access_token), req)
    }

    fn reverse_enrollment<'a>(
        &'a self,
        session: &'a AccountSession,
        student_no: &'a str,
        req: &'a ReverseEnrollmentRequest,
    ) -> GatewayFuture<'a, Result<StudentSummary, GatewayError>> {
        let path = format!(
            "/api/v1/students/{}/reverse",
            encode_query_component(student_no)
        );
        self.send_json("POST", path, Some(&session.access_token), req)
    }

    fn list_courses<'a>(
        &'a self,
        session: &'a AccountSession,
    ) -> GatewayFuture<'a, Result<Vec<CourseSummary>, GatewayError>> {
        self.get_json("/api/v1/courses".to_owned(), Some(&session.access_token))
    }

    fn create_course<'a>(
        &'a self,
        session: &'a AccountSession,
        req: &'a CreateCourseRequest,
    ) -> GatewayFuture<'a, Result<CourseSummary, GatewayError>> {
        self.send_json(
            "POST",
            "/api/v1/courses".to_owned(),
            Some(&session.access_token),
            req,
        )
    }

    fn list_teachings<'a>(
        &'a self,
        session: &'a AccountSession,
    ) -> GatewayFuture<'a, Result<Vec<TeachingSummary>, GatewayError>> {
        self.get_json("/api/v1/teachings".to_owned(), Some(&session.access_token))
    }

    fn assign_teaching<'a>(
        &'a self,
        session: &'a AccountSession,
        req: &'a AssignTeachingRequest,
    ) -> GatewayFuture<'a, Result<TeachingSummary, GatewayError>> {
        self.send_json(
            "POST",
            "/api/v1/teachings".to_owned(),
            Some(&session.access_token),
            req,
        )
    }

    fn list_grades<'a>(
        &'a self,
        session: &'a AccountSession,
        student_no: Option<&'a str>,
    ) -> GatewayFuture<'a, Result<Vec<GradeEntrySummary>, GatewayError>> {
        let path = match student_no.map(str::trim).filter(|val| !val.is_empty()) {
            Some(no) => format!("/api/v1/grades?student_no={}", encode_query_component(no)),
            None => "/api/v1/grades".to_owned(),
        };
        self.get_json(path, Some(&session.access_token))
    }

    fn save_draft_grade<'a>(
        &'a self,
        session: &'a AccountSession,
        req: &'a SaveDraftGradeRequest,
    ) -> GatewayFuture<'a, Result<GradeEntrySummary, GatewayError>> {
        self.send_json(
            "PUT",
            "/api/v1/grades/draft".to_owned(),
            Some(&session.access_token),
            req,
        )
    }

    fn publish_grades<'a>(
        &'a self,
        session: &'a AccountSession,
        req: &'a PublishGradesRequest,
    ) -> GatewayFuture<'a, Result<Vec<GradeEntrySummary>, GatewayError>> {
        self.send_json(
            "POST",
            "/api/v1/grades/publish".to_owned(),
            Some(&session.access_token),
            req,
        )
    }

    fn amend_grade<'a>(
        &'a self,
        session: &'a AccountSession,
        grade_id: &'a str,
        req: &'a AmendGradeRequest,
    ) -> GatewayFuture<'a, Result<GradeEntrySummary, GatewayError>> {
        let path = format!("/api/v1/grades/{}/amend", encode_query_component(grade_id));
        self.send_json("POST", path, Some(&session.access_token), req)
    }

    fn list_grade_amendments<'a>(
        &'a self,
        session: &'a AccountSession,
        grade_id: Option<&'a str>,
    ) -> GatewayFuture<'a, Result<Vec<GradeAmendmentRecord>, GatewayError>> {
        let path = match grade_id.map(str::trim).filter(|val| !val.is_empty()) {
            Some(id) => format!(
                "/api/v1/grade-amendments?grade_id={}",
                encode_query_component(id)
            ),
            None => "/api/v1/grade-amendments".to_owned(),
        };
        self.get_json(path, Some(&session.access_token))
    }

    fn list_accounts<'a>(
        &'a self,
        session: &'a AccountSession,
    ) -> GatewayFuture<'a, Result<Vec<AccountSummary>, GatewayError>> {
        self.get_json("/api/v1/accounts".to_owned(), Some(&session.access_token))
    }

    fn provision_account<'a>(
        &'a self,
        session: &'a AccountSession,
        req: &'a ProvisionAccountRequest,
    ) -> GatewayFuture<'a, Result<AccountSummary, GatewayError>> {
        self.send_json(
            "POST",
            "/api/v1/accounts".to_owned(),
            Some(&session.access_token),
            req,
        )
    }

    fn bind_person_profile<'a>(
        &'a self,
        session: &'a AccountSession,
        account_id: &'a str,
        req: &'a BindPersonProfileRequest,
    ) -> GatewayFuture<'a, Result<AccountSummary, GatewayError>> {
        let path = format!(
            "/api/v1/accounts/{}/person-profile",
            encode_query_component(account_id)
        );
        self.send_json("PUT", path, Some(&session.access_token), req)
    }

    fn change_account_role<'a>(
        &'a self,
        session: &'a AccountSession,
        account_id: &'a str,
        req: &'a ChangeAccountRoleRequest,
    ) -> GatewayFuture<'a, Result<AccountSummary, GatewayError>> {
        let path = format!(
            "/api/v1/accounts/{}/role",
            encode_query_component(account_id)
        );
        self.send_json("PATCH", path, Some(&session.access_token), req)
    }

    fn disable_account<'a>(
        &'a self,
        session: &'a AccountSession,
        account_id: &'a str,
        req: &'a DisableAccountRequest,
    ) -> GatewayFuture<'a, Result<AccountSummary, GatewayError>> {
        let path = format!(
            "/api/v1/accounts/{}/disable",
            encode_query_component(account_id)
        );
        self.send_json("POST", path, Some(&session.access_token), req)
    }
}
