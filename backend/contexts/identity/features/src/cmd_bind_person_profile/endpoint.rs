use axum::{Json, Router, extract::Path, http::StatusCode, routing::put};
use student_management_contracts::{AccountSummary, BindPersonProfileRequest, ProblemDetails};
use student_management_infra::{HttpMethod, SliceDescriptor, SliceStatus, state::AppState};

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
    Path(_account_id): Path<String>,
    Json(_req): Json<BindPersonProfileRequest>,
) -> (StatusCode, Json<ProblemDetails>) {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(ProblemDetails::not_implemented(
            "identity::cmd_bind_person_profile",
        )),
    )
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
    status: SliceStatus::Planned,
    route,
});
