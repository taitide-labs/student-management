use axum::{Json, Router, extract::Path, http::StatusCode, routing::patch};
use student_management_contracts::{AccountSummary, ChangeAccountRoleRequest, ProblemDetails};
use student_management_infra::{HttpMethod, SliceDescriptor, SliceStatus, state::AppState};

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
    Path(_account_id): Path<String>,
    Json(_req): Json<ChangeAccountRoleRequest>,
) -> (StatusCode, Json<ProblemDetails>) {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(ProblemDetails::not_implemented(
            "identity::cmd_change_account_role",
        )),
    )
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
    status: SliceStatus::Planned,
    route,
});
