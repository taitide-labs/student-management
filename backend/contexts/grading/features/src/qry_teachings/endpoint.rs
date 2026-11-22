use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode, header::AUTHORIZATION},
    routing::get,
};
use student_management_contracts::{ProblemDetails, TeachingSummary};
use student_management_infra::{
    AppState, HttpMethod, SliceDescriptor, SliceStatus, authenticate_bearer,
};

use super::handler;

#[utoipa::path(
    get,
    path = "/api/v1/teachings",
    tag = "grading",
    responses((status = 200, description = "教学任务列表", body = Vec<TeachingSummary>))
)]
pub async fn list_teachings(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<(StatusCode, Json<Vec<TeachingSummary>>), (StatusCode, Json<ProblemDetails>)> {
    let auth_header = headers.get(AUTHORIZATION).and_then(|v| v.to_str().ok());
    let operator = authenticate_bearer(&state.pool, auth_header)
        .await
        .map_err(problem_response)?;
    let items = handler::execute(&state, &operator)
        .await
        .map_err(problem_response)?;
    Ok((StatusCode::OK, Json(items)))
}

fn problem_response(problem: ProblemDetails) -> (StatusCode, Json<ProblemDetails>) {
    let status = StatusCode::from_u16(problem.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    (status, Json(problem))
}

fn route() -> Router<AppState> {
    Router::new().route("/api/v1/teachings", get(list_teachings))
}

inventory::submit!(SliceDescriptor {
    context: "grading",
    slice: "qry_teachings",
    method: HttpMethod::Get,
    path: "/api/v1/teachings",
    status: SliceStatus::Active,
    route,
});
