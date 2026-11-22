use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode, header::AUTHORIZATION},
    routing::post,
};
use student_management_contracts::{AmendGradeRequest, GradeEntrySummary, ProblemDetails};
use student_management_infra::{
    AppState, HttpMethod, SliceDescriptor, SliceStatus, authenticate_bearer,
};

use super::handler;

#[utoipa::path(
    post,
    path = "/api/v1/grades/{grade_id}/amend",
    tag = "grading",
    params(("grade_id" = String, Path, description = "成绩记录标识")),
    request_body = AmendGradeRequest,
    responses((status = 200, description = "管理员修订成绩成功", body = GradeEntrySummary))
)]
pub async fn amend_grade(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(grade_id): Path<String>,
    Json(req): Json<AmendGradeRequest>,
) -> Result<(StatusCode, Json<GradeEntrySummary>), (StatusCode, Json<ProblemDetails>)> {
    let auth_header = headers.get(AUTHORIZATION).and_then(|v| v.to_str().ok());
    let operator = authenticate_bearer(&state.pool, auth_header)
        .await
        .map_err(problem_response)?;
    let summary = handler::execute(&state, &operator, &grade_id, req)
        .await
        .map_err(problem_response)?;
    Ok((StatusCode::OK, Json(summary)))
}

fn problem_response(problem: ProblemDetails) -> (StatusCode, Json<ProblemDetails>) {
    let status = StatusCode::from_u16(problem.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    (status, Json(problem))
}

fn route() -> Router<AppState> {
    Router::new().route("/api/v1/grades/{grade_id}/amend", post(amend_grade))
}

inventory::submit!(SliceDescriptor {
    context: "grading",
    slice: "cmd_amend_grade",
    method: HttpMethod::Post,
    path: "/api/v1/grades/{grade_id}/amend",
    status: SliceStatus::Active,
    route,
});
