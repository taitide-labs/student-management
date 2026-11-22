use axum::{
    Json, Router,
    extract::{Query, State},
    http::{HeaderMap, StatusCode, header::AUTHORIZATION},
    routing::get,
};
use serde::Deserialize;
use student_management_contracts::{EnrollmentStatus, ProblemDetails, StudentSummary};
use student_management_infra::{
    AppState, HttpMethod, SliceDescriptor, SliceStatus, authenticate_bearer,
};

use super::handler;

#[derive(Debug, Deserialize)]
pub struct StudentsQuery {
    pub q: Option<String>,
    pub status: Option<EnrollmentStatus>,
}

#[utoipa::path(
    get,
    path = "/api/v1/students",
    tag = "enrollment",
    params(
        ("q" = Option<String>, Query, description = "学号或姓名关键字"),
        ("status" = Option<EnrollmentStatus>, Query, description = "学籍状态过滤")
    ),
    responses((status = 200, description = "学生列表", body = Vec<StudentSummary>))
)]
pub async fn list_students(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<StudentsQuery>,
) -> Result<(StatusCode, Json<Vec<StudentSummary>>), (StatusCode, Json<ProblemDetails>)> {
    let auth_header = headers.get(AUTHORIZATION).and_then(|v| v.to_str().ok());
    let operator = authenticate_bearer(&state.pool, auth_header)
        .await
        .map_err(problem_response)?;
    let items = handler::execute(
        &state,
        &operator,
        query.q.as_deref().unwrap_or(""),
        query.status,
    )
    .await
    .map_err(problem_response)?;
    Ok((StatusCode::OK, Json(items)))
}

fn problem_response(problem: ProblemDetails) -> (StatusCode, Json<ProblemDetails>) {
    let status = StatusCode::from_u16(problem.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    (status, Json(problem))
}

fn route() -> Router<AppState> {
    Router::new().route("/api/v1/students", get(list_students))
}

inventory::submit!(SliceDescriptor {
    context: "enrollment",
    slice: "qry_students",
    method: HttpMethod::Get,
    path: "/api/v1/students",
    status: SliceStatus::Active,
    route,
});
