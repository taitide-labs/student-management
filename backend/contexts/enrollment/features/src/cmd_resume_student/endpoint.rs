use axum::{Json, Router, extract::Path, http::StatusCode, routing::post};
use student_management_contracts::{ProblemDetails, StudentSummary, TransitionStudentRequest};
use student_management_infra::{HttpMethod, SliceDescriptor, SliceStatus, state::AppState};

#[utoipa::path(
    post,
    path = "/api/v1/students/{student_no}/resume",
    tag = "enrollment",
    params(("student_no" = String, Path, description = "学号")),
    request_body = TransitionStudentRequest,
    responses((status = 200, description = "办理复学成功", body = StudentSummary))
)]
pub async fn resume_student(
    Path(_student_no): Path<String>,
    Json(_req): Json<TransitionStudentRequest>,
) -> (StatusCode, Json<ProblemDetails>) {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(ProblemDetails::not_implemented(
            "enrollment::cmd_resume_student",
        )),
    )
}

fn route() -> Router<AppState> {
    Router::new().route("/api/v1/students/{student_no}/resume", post(resume_student))
}

inventory::submit!(SliceDescriptor {
    context: "enrollment",
    slice: "cmd_resume_student",
    method: HttpMethod::Post,
    path: "/api/v1/students/{student_no}/resume",
    status: SliceStatus::Planned,
    route,
});
