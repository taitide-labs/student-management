use axum::{Json, Router, extract::Path, http::StatusCode, routing::post};
use student_management_contracts::{AccountSummary, DisableAccountRequest, ProblemDetails};
use student_management_infra::{HttpMethod, SliceDescriptor, SliceStatus, state::AppState};

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
    Path(_account_id): Path<String>,
    Json(_req): Json<DisableAccountRequest>,
) -> (StatusCode, Json<ProblemDetails>) {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(ProblemDetails::not_implemented(
            "identity::cmd_disable_account",
        )),
    )
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
    status: SliceStatus::Planned,
    route,
});
