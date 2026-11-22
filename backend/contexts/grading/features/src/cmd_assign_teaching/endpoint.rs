use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode, header::AUTHORIZATION},
    routing::post,
};
use student_management_contracts::{AssignTeachingRequest, ProblemDetails, TeachingSummary};
use student_management_infra::{
    AppState, HttpMethod, SliceDescriptor, SliceStatus, authenticate_bearer,
};

use super::handler;

#[utoipa::path(
    post,
    path = "/api/v1/teachings",
    tag = "grading",
    request_body = AssignTeachingRequest,
    responses((status = 201, description = "分派教学任务成功", body = TeachingSummary))
)]
pub async fn assign_teaching(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<AssignTeachingRequest>,
) -> Result<(StatusCode, Json<TeachingSummary>), (StatusCode, Json<ProblemDetails>)> {
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
    Router::new().route("/api/v1/teachings", post(assign_teaching))
}

inventory::submit!(SliceDescriptor {
    context: "grading",
    slice: "cmd_assign_teaching",
    method: HttpMethod::Post,
    path: "/api/v1/teachings",
    status: SliceStatus::Active,
    route,
});
