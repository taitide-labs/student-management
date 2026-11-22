use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode, header::AUTHORIZATION},
    routing::put,
};
use student_management_contracts::{GradeEntrySummary, ProblemDetails, SaveDraftGradeRequest};
use student_management_infra::{
    AppState, HttpMethod, SliceDescriptor, SliceStatus, authenticate_bearer,
};

use super::handler;

#[utoipa::path(
    put,
    path = "/api/v1/grades/draft",
    tag = "grading",
    request_body = SaveDraftGradeRequest,
    responses(
        (status = 200, description = "暂存草稿成绩成功", body = GradeEntrySummary),
        (status = 409, description = "成绩已锁定或学生非在籍", body = ProblemDetails)
    )
)]
pub async fn save_draft_grade(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<SaveDraftGradeRequest>,
) -> Result<(StatusCode, Json<GradeEntrySummary>), (StatusCode, Json<ProblemDetails>)> {
    let auth_header = headers.get(AUTHORIZATION).and_then(|v| v.to_str().ok());
    let operator = authenticate_bearer(&state.pool, auth_header)
        .await
        .map_err(problem_response)?;
    let summary = handler::execute(&state, &operator, req)
        .await
        .map_err(problem_response)?;
    Ok((StatusCode::OK, Json(summary)))
}

fn problem_response(problem: ProblemDetails) -> (StatusCode, Json<ProblemDetails>) {
    let status = StatusCode::from_u16(problem.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    (status, Json(problem))
}

fn route() -> Router<AppState> {
    Router::new().route("/api/v1/grades/draft", put(save_draft_grade))
}

inventory::submit!(SliceDescriptor {
    context: "grading",
    slice: "cmd_save_draft_grade",
    method: HttpMethod::Put,
    path: "/api/v1/grades/draft",
    status: SliceStatus::Active,
    route,
});
