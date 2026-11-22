use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode, header::AUTHORIZATION},
    routing::patch,
};
use student_management_contracts::{
    HomeroomSummary, ProblemDetails, UpdateHomeroomCapacityRequest,
};
use student_management_infra::{
    AppState, HttpMethod, SliceDescriptor, SliceStatus, authenticate_bearer,
};

use super::handler;

#[utoipa::path(
    patch,
    path = "/api/v1/homerooms/{homeroom_id}/capacity",
    tag = "enrollment",
    params(("homeroom_id" = String, Path, description = "行政班标识")),
    request_body = UpdateHomeroomCapacityRequest,
    responses(
        (status = 200, description = "调整容量成功", body = HomeroomSummary),
        (status = 409, description = "低于在籍人数或版本冲突", body = ProblemDetails)
    )
)]
pub async fn update_homeroom_capacity(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(homeroom_id): Path<String>,
    Json(req): Json<UpdateHomeroomCapacityRequest>,
) -> Result<(StatusCode, Json<HomeroomSummary>), (StatusCode, Json<ProblemDetails>)> {
    let auth_header = headers.get(AUTHORIZATION).and_then(|v| v.to_str().ok());
    let operator = authenticate_bearer(&state.pool, auth_header)
        .await
        .map_err(problem_response)?;
    let summary = handler::execute(&state, &operator, &homeroom_id, req)
        .await
        .map_err(problem_response)?;
    Ok((StatusCode::OK, Json(summary)))
}

fn problem_response(problem: ProblemDetails) -> (StatusCode, Json<ProblemDetails>) {
    let status = StatusCode::from_u16(problem.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    (status, Json(problem))
}

fn route() -> Router<AppState> {
    Router::new().route(
        "/api/v1/homerooms/{homeroom_id}/capacity",
        patch(update_homeroom_capacity),
    )
}

inventory::submit!(SliceDescriptor {
    context: "enrollment",
    slice: "cmd_update_homeroom_capacity",
    method: HttpMethod::Patch,
    path: "/api/v1/homerooms/{homeroom_id}/capacity",
    status: SliceStatus::Active,
    route,
});
