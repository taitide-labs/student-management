use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode, header::AUTHORIZATION},
    routing::get,
};
use student_management_contracts::{CourseSummary, ProblemDetails};
use student_management_infra::{
    AppState, HttpMethod, SliceDescriptor, SliceStatus, authenticate_bearer,
};

use super::handler;

#[utoipa::path(
    get,
    path = "/api/v1/courses",
    tag = "grading",
    responses((status = 200, description = "课程列表", body = Vec<CourseSummary>))
)]
pub async fn list_courses(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<(StatusCode, Json<Vec<CourseSummary>>), (StatusCode, Json<ProblemDetails>)> {
    let auth_header = headers.get(AUTHORIZATION).and_then(|v| v.to_str().ok());
    let operator = authenticate_bearer(&state.pool, auth_header)
        .await
        .map_err(problem_response)?;
    let courses = handler::execute(&state, &operator)
        .await
        .map_err(problem_response)?;
    Ok((StatusCode::OK, Json(courses)))
}

fn problem_response(problem: ProblemDetails) -> (StatusCode, Json<ProblemDetails>) {
    let status = StatusCode::from_u16(problem.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    (status, Json(problem))
}

fn route() -> Router<AppState> {
    Router::new().route("/api/v1/courses", get(list_courses))
}

inventory::submit!(SliceDescriptor {
    context: "grading",
    slice: "qry_courses",
    method: HttpMethod::Get,
    path: "/api/v1/courses",
    status: SliceStatus::Active,
    route,
});
