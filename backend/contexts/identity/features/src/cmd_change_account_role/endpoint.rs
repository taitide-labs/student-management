use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode, header::AUTHORIZATION},
    routing::patch,
};
use student_management_contracts::{AccountSummary, ChangeAccountRoleRequest, ProblemDetails};
use student_management_infra::{
    AppState, HttpMethod, SliceDescriptor, SliceStatus, authenticate_bearer,
};

use super::handler;

#[utoipa::path(
    patch,
    path = "/api/v1/accounts/{account_id}/role",
    tag = "identity",
    params(("account_id" = String, Path, description = "目标账号标识")),
    request_body = ChangeAccountRoleRequest,
    responses(
        (status = 200, description = "角色变更成功", body = AccountSummary),
        (status = 409, description = "版本冲突或账号已停用", body = ProblemDetails)
    )
)]
pub async fn change_account_role(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(account_id): Path<String>,
    Json(req): Json<ChangeAccountRoleRequest>,
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
        "/api/v1/accounts/{account_id}/role",
        patch(change_account_role),
    )
}

inventory::submit!(SliceDescriptor {
    context: "identity",
    slice: "cmd_change_account_role",
    method: HttpMethod::Patch,
    path: "/api/v1/accounts/{account_id}/role",
    status: SliceStatus::Active,
    route,
});
