use axum::{Json, Router, extract::Path, http::StatusCode, routing::post};
use student_management_contracts::{ProblemDetails, StudentSummary, TransferHomeroomRequest};
use student_management_infra::{HttpMethod, SliceDescriptor, SliceStatus, state::AppState};

#[utoipa::path(
    post,
    path = "/api/v1/students/{student_no}/transfer",
    tag = "enrollment",
    params(("student_no" = String, Path, description = "学号")),
    request_body = TransferHomeroomRequest,
    responses(
        (status = 200, description = "转班成功", body = StudentSummary),
        (status = 409, description = "状态锁定、容量已满或版本冲突", body = ProblemDetails)
    )
)]
pub async fn transfer_homeroom(
    Path(_student_no): Path<String>,
    Json(_req): Json<TransferHomeroomRequest>,
) -> (StatusCode, Json<ProblemDetails>) {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(ProblemDetails::not_implemented(
            "enrollment::cmd_transfer_homeroom",
        )),
    )
}

fn route() -> Router<AppState> {
    Router::new().route(
        "/api/v1/students/{student_no}/transfer",
        post(transfer_homeroom),
    )
}

inventory::submit!(SliceDescriptor {
    context: "enrollment",
    slice: "cmd_transfer_homeroom",
    method: HttpMethod::Post,
    path: "/api/v1/students/{student_no}/transfer",
    status: SliceStatus::Planned,
    route,
});
