use axum::{Json, Router, http::StatusCode, routing::post};
use student_management_contracts::{AccountSummary, ProblemDetails, ProvisionAccountRequest};
use student_management_infra::{HttpMethod, SliceDescriptor, SliceStatus, state::AppState};

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
    Json(_req): Json<ProvisionAccountRequest>,
) -> (StatusCode, Json<ProblemDetails>) {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(ProblemDetails::not_implemented(
            "identity::cmd_provision_account",
        )),
    )
}

fn route() -> Router<AppState> {
    Router::new().route("/api/v1/accounts", post(provision_account))
}

inventory::submit!(SliceDescriptor {
    context: "identity",
    slice: "cmd_provision_account",
    method: HttpMethod::Post,
    path: "/api/v1/accounts",
    status: SliceStatus::Planned,
    route,
});
