use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode, header::AUTHORIZATION},
    routing::post,
};
use student_management_contracts::{EnrollStudentRequest, ProblemDetails, StudentSummary};
use student_management_infra::{
    AppState, HttpMethod, SliceDescriptor, SliceStatus, authenticate_bearer,
};

use super::handler;

#[utoipa::path(
    post,
    path = "/api/v1/students",
    tag = "enrollment",
    request_body = EnrollStudentRequest,
    responses(
        (status = 201, description = "入学建档成功", body = StudentSummary),
        (status = 409, description = "班级满员或幂等冲突", body = ProblemDetails)
    )
)]
pub async fn enroll_student(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<EnrollStudentRequest>,
) -> Result<(StatusCode, Json<StudentSummary>), (StatusCode, Json<ProblemDetails>)> {
    let auth_header = headers.get(AUTHORIZATION).and_then(|v| v.to_str().ok());
    let operator = authenticate_bearer(&state.pool, auth_header)
        .await
        .map_err(problem_response)?;
    let summary = handler::execute(&state, &operator, req)
        .await
        .map_err(problem_response)?;
    Ok((StatusCode::CREATED, Json(summary)))
}

fn problem_response(problem: ProblemDetails) -> (StatusCode, Json<ProblemDetails>) {
    let status = StatusCode::from_u16(problem.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    (status, Json(problem))
}

fn route() -> Router<AppState> {
    Router::new().route("/api/v1/students", post(enroll_student))
}

inventory::submit!(SliceDescriptor {
    context: "enrollment",
    slice: "cmd_enroll_student",
    method: HttpMethod::Post,
    path: "/api/v1/students",
    status: SliceStatus::Active,
    route,
});
