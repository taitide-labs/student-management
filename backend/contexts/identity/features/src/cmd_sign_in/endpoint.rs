use axum::{Json, Router, http::StatusCode, routing::post};
use student_management_contracts::{AccountSession, ProblemDetails, SignInRequest};
use student_management_infra::{HttpMethod, SliceDescriptor, SliceStatus, state::AppState};

#[utoipa::path(
    post,
    path = "/api/v1/auth/sign-in",
    tag = "identity",
    request_body = SignInRequest,
    responses(
        (status = 200, description = "登录成功", body = AccountSession),
        (status = 401, description = "凭证无效", body = ProblemDetails)
    )
)]
pub async fn sign_in(Json(_req): Json<SignInRequest>) -> (StatusCode, Json<ProblemDetails>) {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(ProblemDetails::not_implemented("identity::cmd_sign_in")),
    )
}

fn route() -> Router<AppState> {
    Router::new().route("/api/v1/auth/sign-in", post(sign_in))
}

inventory::submit!(SliceDescriptor {
    context: "identity",
    slice: "cmd_sign_in",
    method: HttpMethod::Post,
    path: "/api/v1/auth/sign-in",
    status: SliceStatus::Planned,
    route,
});
