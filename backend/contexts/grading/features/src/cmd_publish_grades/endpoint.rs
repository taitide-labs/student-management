use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode, header::AUTHORIZATION},
    routing::post,
};
use student_management_contracts::{GradeEntrySummary, ProblemDetails, PublishGradesRequest};
use student_management_infra::{
    AppState, HttpMethod, SliceDescriptor, SliceStatus, authenticate_bearer,
};

use super::handler;

#[utoipa::path(
    post,
    path = "/api/v1/grades/publish",
    tag = "grading",
    request_body = PublishGradesRequest,
    responses((status = 200, description = "发布并锁定成绩成功", body = Vec<GradeEntrySummary>))
)]
pub async fn publish_grades(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<PublishGradesRequest>,
) -> Result<(StatusCode, Json<Vec<GradeEntrySummary>>), (StatusCode, Json<ProblemDetails>)> {
    let auth_header = headers.get(AUTHORIZATION).and_then(|v| v.to_str().ok());
    let operator = authenticate_bearer(&state.pool, auth_header)
        .await
        .map_err(problem_response)?;
    let items = handler::execute(&state, &operator, req)
        .await
        .map_err(problem_response)?;
    Ok((StatusCode::OK, Json(items)))
}

fn problem_response(problem: ProblemDetails) -> (StatusCode, Json<ProblemDetails>) {
    let status = StatusCode::from_u16(problem.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    (status, Json(problem))
}

fn route() -> Router<AppState> {
    Router::new().route("/api/v1/grades/publish", post(publish_grades))
}

inventory::submit!(SliceDescriptor {
    context: "grading",
    slice: "cmd_publish_grades",
    method: HttpMethod::Post,
    path: "/api/v1/grades/publish",
    status: SliceStatus::Active,
    route,
});
