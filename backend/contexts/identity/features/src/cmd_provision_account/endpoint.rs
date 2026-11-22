use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode, header::AUTHORIZATION},
    routing::post,
};
use student_management_contracts::{AccountSummary, ProblemDetails, ProvisionAccountRequest};
use student_management_infra::{
    AppState, HttpMethod, SliceDescriptor, SliceStatus, authenticate_bearer,
};

use super::handler;

#[utoipa::path(
    post,
    path = "/api/v1/accounts",
    tag = "identity",
    request_body = ProvisionAccountRequest,
    responses(
        (status = 201, description = "开立账号成功", body = AccountSummary),
        (status = 400, description = "参数无效", body = ProblemDetails),
        (status = 409, description = "账号重复", body = ProblemDetails)
    )
)]
pub async fn provision_account(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<ProvisionAccountRequest>,
) -> Result<(StatusCode, Json<AccountSummary>), (StatusCode, Json<ProblemDetails>)> {
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
    Router::new().route("/api/v1/accounts", post(provision_account))
}

inventory::submit!(SliceDescriptor {
    context: "identity",
    slice: "cmd_provision_account",
    method: HttpMethod::Post,
    path: "/api/v1/accounts",
    status: SliceStatus::Active,
    route,
});
