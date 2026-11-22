use axum::{Json, Router, http::StatusCode, routing::get};
use student_management_contracts::{AccountSummary, ProblemDetails};
use student_management_infra::{HttpMethod, SliceDescriptor, SliceStatus, state::AppState};

#[utoipa::path(
    get,
    path = "/api/v1/accounts",
    tag = "identity",
    responses(
        (status = 200, description = "账号列表", body = Vec<AccountSummary>),
        (status = 403, description = "权限不足", body = ProblemDetails)
    )
)]
pub async fn list_accounts() -> (StatusCode, Json<ProblemDetails>) {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(ProblemDetails::not_implemented("identity::qry_accounts")),
    )
}

fn route() -> Router<AppState> {
    Router::new().route("/api/v1/accounts", get(list_accounts))
}

inventory::submit!(SliceDescriptor {
    context: "identity",
    slice: "qry_accounts",
    method: HttpMethod::Get,
    path: "/api/v1/accounts",
    status: SliceStatus::Planned,
    route,
});
