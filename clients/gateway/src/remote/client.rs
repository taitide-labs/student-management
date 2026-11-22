use serde::{Serialize, de::DeserializeOwned};
use student_management_contracts::{
    AccountSession, AccountSummary, AmendGradeRequest, AssignTeachingRequest,
    BindPersonProfileRequest, ChangeAccountRoleRequest, CourseSummary, CreateCourseRequest,
    CreateHomeroomRequest, DisableAccountRequest, EnrollStudentRequest, EnrollmentStatus,
    GradeAmendmentRecord, GradeEntrySummary, HomeroomSummary, ProblemDetails,
    ProvisionAccountRequest, PublishGradesRequest, ReverseEnrollmentRequest, Role,
    SaveDraftGradeRequest, SignInRequest, StudentDetailPayload, StudentSummary, TeachingSummary,
    TransferHomeroomRequest, TransitionStudentRequest, UpdateHomeroomCapacityRequest,
};

use crate::contract::{Gateway, GatewayError};

#[derive(Debug, Clone)]
pub struct RemoteGatewayConfig {
    pub base_url: String,
}

#[derive(Clone)]
pub struct RemoteGateway {
    base_url: String,
    #[cfg(not(target_arch = "wasm32"))]
    agent: ureq::Agent,
}

impl RemoteGateway {
    #[must_use]
    pub fn new(config: &RemoteGatewayConfig) -> Self {
        let base_url = config.base_url.trim_end_matches('/').to_owned();
        #[cfg(not(target_arch = "wasm32"))]
        let agent = ureq::Agent::config_builder()
            .http_status_as_error(false)
            .build()
            .new_agent();
        Self {
            base_url,
            #[cfg(not(target_arch = "wasm32"))]
            agent,
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn get_json<T: DeserializeOwned>(
        &self,
        path: &str,
        token: Option<&str>,
    ) -> Result<T, GatewayError> {
        let mut req = self.agent.get(self.url(path));
        if let Some(tok) = token {
            req = req.header("authorization", &format!("Bearer {tok}"));
        }
        let resp = req
            .call()
            .map_err(|err| GatewayError::Transport(err.to_string()))?;
        decode_response(resp)
    }

    #[cfg(target_arch = "wasm32")]
    fn get_json<T: DeserializeOwned>(
        &self,
        _path: &str,
        _token: Option<&str>,
    ) -> Result<T, GatewayError> {
        let _ = &self.base_url;
        Err(GatewayError::Transport(
            "WASM 目标请通过浏览器宿主传输适配器连接远端网关".to_owned(),
        ))
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn send_json<B: Serialize, T: DeserializeOwned>(
        &self,
        method: &str,
        path: &str,
        token: Option<&str>,
        body: &B,
    ) -> Result<T, GatewayError> {
        let auth = token.map(|tok| format!("Bearer {tok}"));
        let url = self.url(path);
        let resp = match method {
            "POST" => {
                let mut req = self.agent.post(&url);
                if let Some(h) = auth.as_deref() {
                    req = req.header("authorization", h);
                }
                req.send_json(body)
            }
            "PUT" => {
                let mut req = self.agent.put(&url);
                if let Some(h) = auth.as_deref() {
                    req = req.header("authorization", h);
                }
                req.send_json(body)
            }
            "PATCH" => {
                let mut req = self.agent.patch(&url);
                if let Some(h) = auth.as_deref() {
                    req = req.header("authorization", h);
                }
                req.send_json(body)
            }
            _ => {
                return Err(GatewayError::Transport(format!(
                    "不支持的 HTTP 方法：{method}"
                )));
            }
        }
        .map_err(|err| GatewayError::Transport(err.to_string()))?;
        decode_response(resp)
    }

    #[cfg(target_arch = "wasm32")]
    fn send_json<B: Serialize, T: DeserializeOwned>(
        &self,
        _method: &str,
        _path: &str,
        _token: Option<&str>,
        _body: &B,
    ) -> Result<T, GatewayError> {
        Err(GatewayError::Transport(
            "WASM 目标请通过浏览器宿主传输适配器连接远端网关".to_owned(),
        ))
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn decode_response<T: DeserializeOwned>(
    mut resp: ureq::http::Response<ureq::Body>,
) -> Result<T, GatewayError> {
    let status = resp.status();
    let raw = resp
        .body_mut()
        .read_to_vec()
        .map_err(|err| GatewayError::Transport(err.to_string()))?;

    if status.is_success() {
        return serde_json::from_slice(&raw)
            .map_err(|err| GatewayError::Transport(format!("响应反序列化失败：{err}")));
    }

    if let Ok(problem) = serde_json::from_slice::<ProblemDetails>(&raw) {
        return Err(GatewayError::Problem(problem));
    }

    let detail = String::from_utf8_lossy(&raw).into_owned();
    Err(GatewayError::Problem(ProblemDetails {
        problem_type: "https://student-management.local/problems/http".to_owned(),
        title: format!("HTTP {}", status.as_u16()),
        status: status.as_u16(),
        detail: if detail.is_empty() {
            status.canonical_reason().unwrap_or("请求失败").to_owned()
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

impl Gateway for RemoteGateway {
    fn sign_in(&self, req: &SignInRequest) -> Result<AccountSession, GatewayError> {
        self.send_json("POST", "/api/v1/auth/sign-in", None, req)
    }

    fn switch_role(&self, _role: Role) -> Result<AccountSession, GatewayError> {
        Err(GatewayError::Problem(ProblemDetails::not_implemented(
            "identity::switch_role",
        )))
    }

    fn list_homerooms(&self) -> Result<Vec<HomeroomSummary>, GatewayError> {
        self.get_json("/api/v1/homerooms", None)
    }

    fn create_homeroom(
        &mut self,
        session: &AccountSession,
        req: &CreateHomeroomRequest,
    ) -> Result<HomeroomSummary, GatewayError> {
        self.send_json(
            "POST",
            "/api/v1/homerooms",
            Some(&session.access_token),
            req,
        )
    }

    fn update_homeroom_capacity(
        &mut self,
        session: &AccountSession,
        homeroom_id: &str,
        req: &UpdateHomeroomCapacityRequest,
    ) -> Result<HomeroomSummary, GatewayError> {
        let path = format!(
            "/api/v1/homerooms/{}/capacity",
            encode_query_component(homeroom_id)
        );
        self.send_json("PATCH", &path, Some(&session.access_token), req)
    }

    fn list_students(
        &self,
        session: &AccountSession,
        query: &str,
        status_filter: Option<EnrollmentStatus>,
    ) -> Result<Vec<StudentSummary>, GatewayError> {
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
        self.get_json(&path, Some(&session.access_token))
    }

    fn student_detail(
        &self,
        session: &AccountSession,
        student_no: &str,
    ) -> Result<StudentDetailPayload, GatewayError> {
        let path = format!("/api/v1/students/{}", encode_query_component(student_no));
        self.get_json(&path, Some(&session.access_token))
    }

    fn enroll_student(
        &mut self,
        session: &AccountSession,
        req: &EnrollStudentRequest,
    ) -> Result<StudentSummary, GatewayError> {
        self.send_json("POST", "/api/v1/students", Some(&session.access_token), req)
    }

    fn transfer_homeroom(
        &mut self,
        session: &AccountSession,
        student_no: &str,
        req: &TransferHomeroomRequest,
    ) -> Result<StudentSummary, GatewayError> {
        let path = format!(
            "/api/v1/students/{}/transfer",
            encode_query_component(student_no)
        );
        self.send_json("POST", &path, Some(&session.access_token), req)
    }

    fn suspend_student(
        &mut self,
        session: &AccountSession,
        student_no: &str,
        req: &TransitionStudentRequest,
    ) -> Result<StudentSummary, GatewayError> {
        let path = format!(
            "/api/v1/students/{}/suspend",
            encode_query_component(student_no)
        );
        self.send_json("POST", &path, Some(&session.access_token), req)
    }

    fn resume_student(
        &mut self,
        session: &AccountSession,
        student_no: &str,
        req: &TransitionStudentRequest,
    ) -> Result<StudentSummary, GatewayError> {
        let path = format!(
            "/api/v1/students/{}/resume",
            encode_query_component(student_no)
        );
        self.send_json("POST", &path, Some(&session.access_token), req)
    }

    fn withdraw_student(
        &mut self,
        session: &AccountSession,
        student_no: &str,
        req: &TransitionStudentRequest,
    ) -> Result<StudentSummary, GatewayError> {
        let path = format!(
            "/api/v1/students/{}/withdraw",
            encode_query_component(student_no)
        );
        self.send_json("POST", &path, Some(&session.access_token), req)
    }

    fn graduate_student(
        &mut self,
        session: &AccountSession,
        student_no: &str,
        req: &TransitionStudentRequest,
    ) -> Result<StudentSummary, GatewayError> {
        let path = format!(
            "/api/v1/students/{}/graduate",
            encode_query_component(student_no)
        );
        self.send_json("POST", &path, Some(&session.access_token), req)
    }

    fn reverse_enrollment(
        &mut self,
        session: &AccountSession,
        student_no: &str,
        req: &ReverseEnrollmentRequest,
    ) -> Result<StudentSummary, GatewayError> {
        let path = format!(
            "/api/v1/students/{}/reverse",
            encode_query_component(student_no)
        );
        self.send_json("POST", &path, Some(&session.access_token), req)
    }

    fn list_courses(&self, session: &AccountSession) -> Result<Vec<CourseSummary>, GatewayError> {
        self.get_json("/api/v1/courses", Some(&session.access_token))
    }

    fn create_course(
        &mut self,
        session: &AccountSession,
        req: &CreateCourseRequest,
    ) -> Result<CourseSummary, GatewayError> {
        self.send_json("POST", "/api/v1/courses", Some(&session.access_token), req)
    }

    fn list_teachings(
        &self,
        session: &AccountSession,
    ) -> Result<Vec<TeachingSummary>, GatewayError> {
        self.get_json("/api/v1/teachings", Some(&session.access_token))
    }

    fn assign_teaching(
        &mut self,
        session: &AccountSession,
        req: &AssignTeachingRequest,
    ) -> Result<TeachingSummary, GatewayError> {
        self.send_json(
            "POST",
            "/api/v1/teachings",
            Some(&session.access_token),
            req,
        )
    }

    fn list_grades(
        &self,
        session: &AccountSession,
        student_no: Option<&str>,
    ) -> Result<Vec<GradeEntrySummary>, GatewayError> {
        let path = match student_no.map(str::trim).filter(|val| !val.is_empty()) {
            Some(no) => format!("/api/v1/grades?student_no={}", encode_query_component(no)),
            None => "/api/v1/grades".to_owned(),
        };
        self.get_json(&path, Some(&session.access_token))
    }

    fn save_draft_grade(
        &mut self,
        session: &AccountSession,
        req: &SaveDraftGradeRequest,
    ) -> Result<GradeEntrySummary, GatewayError> {
        self.send_json(
            "PUT",
            "/api/v1/grades/draft",
            Some(&session.access_token),
            req,
        )
    }

    fn publish_grades(
        &mut self,
        session: &AccountSession,
        req: &PublishGradesRequest,
    ) -> Result<Vec<GradeEntrySummary>, GatewayError> {
        self.send_json(
            "POST",
            "/api/v1/grades/publish",
            Some(&session.access_token),
            req,
        )
    }

    fn amend_grade(
        &mut self,
        session: &AccountSession,
        grade_id: &str,
        req: &AmendGradeRequest,
    ) -> Result<GradeEntrySummary, GatewayError> {
        let path = format!("/api/v1/grades/{}/amend", encode_query_component(grade_id));
        self.send_json("POST", &path, Some(&session.access_token), req)
    }

    fn list_grade_amendments(
        &self,
        session: &AccountSession,
        grade_id: Option<&str>,
    ) -> Result<Vec<GradeAmendmentRecord>, GatewayError> {
        let path = match grade_id.map(str::trim).filter(|val| !val.is_empty()) {
            Some(id) => format!(
                "/api/v1/grade-amendments?grade_id={}",
                encode_query_component(id)
            ),
            None => "/api/v1/grade-amendments".to_owned(),
        };
        self.get_json(&path, Some(&session.access_token))
    }

    fn list_accounts(&self, session: &AccountSession) -> Result<Vec<AccountSummary>, GatewayError> {
        self.get_json("/api/v1/accounts", Some(&session.access_token))
    }

    fn provision_account(
        &mut self,
        session: &AccountSession,
        req: &ProvisionAccountRequest,
    ) -> Result<AccountSummary, GatewayError> {
        self.send_json("POST", "/api/v1/accounts", Some(&session.access_token), req)
    }

    fn bind_person_profile(
        &mut self,
        session: &AccountSession,
        account_id: &str,
        req: &BindPersonProfileRequest,
    ) -> Result<AccountSummary, GatewayError> {
        let path = format!(
            "/api/v1/accounts/{}/person-profile",
            encode_query_component(account_id)
        );
        self.send_json("PUT", &path, Some(&session.access_token), req)
    }

    fn change_account_role(
        &mut self,
        session: &AccountSession,
        account_id: &str,
        req: &ChangeAccountRoleRequest,
    ) -> Result<AccountSummary, GatewayError> {
        let path = format!(
            "/api/v1/accounts/{}/role",
            encode_query_component(account_id)
        );
        self.send_json("PATCH", &path, Some(&session.access_token), req)
    }

    fn disable_account(
        &mut self,
        session: &AccountSession,
        account_id: &str,
        req: &DisableAccountRequest,
    ) -> Result<AccountSummary, GatewayError> {
        let path = format!(
            "/api/v1/accounts/{}/disable",
            encode_query_component(account_id)
        );
        self.send_json("POST", &path, Some(&session.access_token), req)
    }
}
