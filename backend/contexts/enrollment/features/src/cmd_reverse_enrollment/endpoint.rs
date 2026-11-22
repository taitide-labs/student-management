use axum::{Json, Router, extract::Path, http::StatusCode, routing::post};
use student_management_contracts::{ProblemDetails, ReverseEnrollmentRequest, StudentSummary};
use student_management_infra::{HttpMethod, SliceDescriptor, SliceStatus, state::AppState};

#[utoipa::path(
    post,
    path = "/api/v1/students/{student_no}/reverse",
    tag = "enrollment",
    params(("student_no" = String, Path, description = "学号")),
    request_body = ReverseEnrollmentRequest,
    responses((status = 200, description = "终态误操作冲正成功", body = StudentSummary))
)]
pub async fn reverse_enrollment(
    Path(_student_no): Path<String>,
    Json(_req): Json<ReverseEnrollmentRequest>,
) -> (StatusCode, Json<ProblemDetails>) {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(ProblemDetails::not_implemented(
            "enrollment::cmd_reverse_enrollment",
        )),
    )
}

fn route() -> Router<AppState> {
    Router::new().route(
        "/api/v1/students/{student_no}/reverse",
        post(reverse_enrollment),
    )
}

inventory::submit!(SliceDescriptor {
    context: "enrollment",
    slice: "cmd_reverse_enrollment",
    method: HttpMethod::Post,
    path: "/api/v1/students/{student_no}/reverse",
    status: SliceStatus::Planned,
    route,
});
