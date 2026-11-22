use axum::{
    Json, Router,
    extract::{Query, State},
    http::{HeaderMap, StatusCode, header::AUTHORIZATION},
    routing::get,
};
use serde::Deserialize;
use student_management_contracts::{GradeAmendmentRecord, ProblemDetails};
use student_management_infra::{
    AppState, HttpMethod, SliceDescriptor, SliceStatus, authenticate_bearer,
};

use super::handler;

#[derive(Debug, Deserialize)]
pub struct GradeAmendmentsQuery {
    pub grade_id: Option<String>,
}

#[utoipa::path(
    get,
    path = "/api/v1/grade-amendments",
    tag = "grading",
    params(("grade_id" = Option<String>, Query, description = "按成绩记录过滤")),
    responses((status = 200, description = "成绩修订审计列表", body = Vec<GradeAmendmentRecord>))
)]
pub async fn list_grade_amendments(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<GradeAmendmentsQuery>,
) -> Result<(StatusCode, Json<Vec<GradeAmendmentRecord>>), (StatusCode, Json<ProblemDetails>)> {
    let auth_header = headers.get(AUTHORIZATION).and_then(|v| v.to_str().ok());
    let operator = authenticate_bearer(&state.pool, auth_header)
        .await
        .map_err(problem_response)?;
    let records = handler::execute(&state, &operator, query.grade_id.as_deref())
        .await
        .map_err(problem_response)?;
    Ok((StatusCode::OK, Json(records)))
}

fn problem_response(problem: ProblemDetails) -> (StatusCode, Json<ProblemDetails>) {
    let status = StatusCode::from_u16(problem.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    (status, Json(problem))
}

fn route() -> Router<AppState> {
    Router::new().route("/api/v1/grade-amendments", get(list_grade_amendments))
}

inventory::submit!(SliceDescriptor {
    context: "grading",
    slice: "qry_grade_amendments",
    method: HttpMethod::Get,
    path: "/api/v1/grade-amendments",
    status: SliceStatus::Active,
    route,
});
