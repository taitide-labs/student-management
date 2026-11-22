use axum::{
    Json, Router,
    extract::{Query, State},
    http::{HeaderMap, StatusCode, header::AUTHORIZATION},
    routing::get,
};
use serde::Deserialize;
use student_management_contracts::{GradeEntrySummary, ProblemDetails};
use student_management_infra::{
    AppState, HttpMethod, SliceDescriptor, SliceStatus, authenticate_bearer,
};

use super::handler;

#[derive(Debug, Deserialize)]
pub struct GradesQuery {
    pub student_no: Option<String>,
}

#[utoipa::path(
    get,
    path = "/api/v1/grades",
    tag = "grading",
    params(("student_no" = Option<String>, Query, description = "按学号过滤成绩")),
    responses((status = 200, description = "成绩列表", body = Vec<GradeEntrySummary>))
)]
pub async fn list_grades(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<GradesQuery>,
) -> Result<(StatusCode, Json<Vec<GradeEntrySummary>>), (StatusCode, Json<ProblemDetails>)> {
    let auth_header = headers.get(AUTHORIZATION).and_then(|v| v.to_str().ok());
    let operator = authenticate_bearer(&state.pool, auth_header)
        .await
        .map_err(problem_response)?;
    let items = handler::execute(&state, &operator, query.student_no.as_deref())
        .await
        .map_err(problem_response)?;
    Ok((StatusCode::OK, Json(items)))
}

fn problem_response(problem: ProblemDetails) -> (StatusCode, Json<ProblemDetails>) {
    let status = StatusCode::from_u16(problem.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    (status, Json(problem))
}

fn route() -> Router<AppState> {
    Router::new().route("/api/v1/grades", get(list_grades))
}

inventory::submit!(SliceDescriptor {
    context: "grading",
    slice: "qry_grades",
    method: HttpMethod::Get,
    path: "/api/v1/grades",
    status: SliceStatus::Active,
    route,
});
