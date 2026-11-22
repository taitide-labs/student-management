use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode, header::AUTHORIZATION},
    routing::post,
};
use student_management_contracts::{CreateHomeroomRequest, HomeroomSummary, ProblemDetails};
use student_management_infra::{
    AppState, HttpMethod, SliceDescriptor, SliceStatus, authenticate_bearer,
};

use super::handler;

#[utoipa::path(
    post,
    path = "/api/v1/homerooms",
    tag = "enrollment",
    request_body = CreateHomeroomRequest,
    responses(
        (status = 201, description = "创建行政班成功", body = HomeroomSummary),
        (status = 409, description = "班级名称重复", body = ProblemDetails)
    )
)]
pub async fn create_homeroom(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateHomeroomRequest>,
) -> Result<(StatusCode, Json<HomeroomSummary>), (StatusCode, Json<ProblemDetails>)> {
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
    Router::new().route("/api/v1/homerooms", post(create_homeroom))
}

inventory::submit!(SliceDescriptor {
    context: "enrollment",
    slice: "cmd_create_homeroom",
    method: HttpMethod::Post,
    path: "/api/v1/homerooms",
    status: SliceStatus::Active,
    route,
});
