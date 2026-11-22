use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode, header::AUTHORIZATION},
    routing::post,
};
use student_management_contracts::{AccountSummary, DisableAccountRequest, ProblemDetails};
use student_management_infra::{
    AppState, HttpMethod, SliceDescriptor, SliceStatus, authenticate_bearer,
};

use super::handler;

#[utoipa::path(
    post,
    path = "/api/v1/accounts/{account_id}/disable",
    tag = "identity",
    params(("account_id" = String, Path, description = "目标账号标识")),
    request_body = DisableAccountRequest,
    responses(
        (status = 200, description = "账号停用成功", body = AccountSummary),
        (status = 409, description = "禁止停用本人或版本冲突", body = ProblemDetails)
    )
)]
pub async fn disable_account(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(account_id): Path<String>,
    Json(req): Json<DisableAccountRequest>,
) -> Result<(StatusCode, Json<AccountSummary>), (StatusCode, Json<ProblemDetails>)> {
    let auth_header = headers.get(AUTHORIZATION).and_then(|v| v.to_str().ok());
    let operator = authenticate_bearer(&state.pool, auth_header)
        .await
        .map_err(problem_response)?;
    let summary = handler::execute(&state, &operator, &account_id, req)
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
        "/api/v1/accounts/{account_id}/disable",
        post(disable_account),
    )
}

inventory::submit!(SliceDescriptor {
    context: "identity",
    slice: "cmd_disable_account",
    method: HttpMethod::Post,
    path: "/api/v1/accounts/{account_id}/disable",
    status: SliceStatus::Active,
    route,
});
