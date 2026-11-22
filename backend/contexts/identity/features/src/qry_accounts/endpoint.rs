use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode, header::AUTHORIZATION},
    routing::get,
};
use student_management_contracts::{AccountSummary, ProblemDetails};
use student_management_infra::{
    AppState, HttpMethod, SliceDescriptor, SliceStatus, authenticate_bearer,
};

use super::handler;

#[utoipa::path(
    get,
    path = "/api/v1/accounts",
    tag = "identity",
    responses(
        (status = 200, description = "账号列表", body = Vec<AccountSummary>),
        (status = 403, description = "权限不足", body = ProblemDetails)
    )
)]
pub async fn list_accounts(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<(StatusCode, Json<Vec<AccountSummary>>), (StatusCode, Json<ProblemDetails>)> {
    let auth_header = headers.get(AUTHORIZATION).and_then(|v| v.to_str().ok());
    let operator = authenticate_bearer(&state.pool, auth_header)
        .await
        .map_err(problem_response)?;
    let accounts = handler::execute(&state, &operator)
        .await
        .map_err(problem_response)?;
    Ok((StatusCode::OK, Json(accounts)))
}

fn problem_response(problem: ProblemDetails) -> (StatusCode, Json<ProblemDetails>) {
    let status = StatusCode::from_u16(problem.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    (status, Json(problem))
}

fn route() -> Router<AppState> {
    Router::new().route("/api/v1/accounts", get(list_accounts))
}

inventory::submit!(SliceDescriptor {
    context: "identity",
    slice: "qry_accounts",
    method: HttpMethod::Get,
    path: "/api/v1/accounts",
    status: SliceStatus::Active,
    route,
});
