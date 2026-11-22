use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode, header::AUTHORIZATION},
    routing::post,
};
use student_management_contracts::{ProblemDetails, StudentSummary, TransitionStudentRequest};
use student_management_infra::{
    AppState, HttpMethod, SliceDescriptor, SliceStatus, authenticate_bearer,
};

use super::handler;

#[utoipa::path(
    post,
    path = "/api/v1/students/{student_no}/resume",
    tag = "enrollment",
    params(("student_no" = String, Path, description = "学号")),
    request_body = TransitionStudentRequest,
    responses((status = 200, description = "办理复学成功", body = StudentSummary))
)]
pub async fn resume_student(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(student_no): Path<String>,
    Json(req): Json<TransitionStudentRequest>,
) -> Result<(StatusCode, Json<StudentSummary>), (StatusCode, Json<ProblemDetails>)> {
    let auth_header = headers.get(AUTHORIZATION).and_then(|v| v.to_str().ok());
    let operator = authenticate_bearer(&state.pool, auth_header)
        .await
        .map_err(problem_response)?;
    let summary = handler::execute(&state, &operator, &student_no, req)
        .await
        .map_err(problem_response)?;
    Ok((StatusCode::OK, Json(summary)))
}

fn problem_response(problem: ProblemDetails) -> (StatusCode, Json<ProblemDetails>) {
    let status = StatusCode::from_u16(problem.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    (status, Json(problem))
}

fn route() -> Router<AppState> {
    Router::new().route("/api/v1/students/{student_no}/resume", post(resume_student))
}

inventory::submit!(SliceDescriptor {
    context: "enrollment",
    slice: "cmd_resume_student",
    method: HttpMethod::Post,
    path: "/api/v1/students/{student_no}/resume",
    status: SliceStatus::Active,
    route,
});
