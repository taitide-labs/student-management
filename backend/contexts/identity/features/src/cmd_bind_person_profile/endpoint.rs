use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode, header::AUTHORIZATION},
    routing::put,
};
use student_management_contracts::{AccountSummary, BindPersonProfileRequest, ProblemDetails};
use student_management_infra::{
    AppState, HttpMethod, SliceDescriptor, SliceStatus, authenticate_bearer,
};

use super::handler;

#[utoipa::path(
    put,
    path = "/api/v1/accounts/{account_id}/person-profile",
    tag = "identity",
    params(("account_id" = String, Path, description = "目标账号标识")),
    request_body = BindPersonProfileRequest,
    responses(
        (status = 200, description = "绑定人员标识成功", body = AccountSummary),
        (status = 409, description = "角色或版本冲突", body = ProblemDetails)
    )
)]
pub async fn bind_person_profile(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(account_id): Path<String>,
    Json(req): Json<BindPersonProfileRequest>,
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
        "/api/v1/accounts/{account_id}/person-profile",
        put(bind_person_profile),
    )
}

inventory::submit!(SliceDescriptor {
    context: "identity",
    slice: "cmd_bind_person_profile",
    method: HttpMethod::Put,
    path: "/api/v1/accounts/{account_id}/person-profile",
    status: SliceStatus::Active,
    route,
});
