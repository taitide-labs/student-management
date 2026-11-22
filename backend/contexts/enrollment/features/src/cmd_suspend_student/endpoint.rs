use axum::{Json, Router, extract::Path, http::StatusCode, routing::post};
use student_management_contracts::{ProblemDetails, StudentSummary, TransitionStudentRequest};
use student_management_infra::{HttpMethod, SliceDescriptor, SliceStatus, state::AppState};

#[utoipa::path(
    post,
    path = "/api/v1/students/{student_no}/suspend",
    tag = "enrollment",
    params(("student_no" = String, Path, description = "学号")),
    request_body = TransitionStudentRequest,
    responses((status = 200, description = "办理休学成功", body = StudentSummary))
)]
pub async fn suspend_student(
    Path(_student_no): Path<String>,
    Json(_req): Json<TransitionStudentRequest>,
) -> (StatusCode, Json<ProblemDetails>) {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(ProblemDetails::not_implemented(
            "enrollment::cmd_suspend_student",
        )),
    )
}

fn route() -> Router<AppState> {
    Router::new().route(
        "/api/v1/students/{student_no}/suspend",
        post(suspend_student),
    )
}

inventory::submit!(SliceDescriptor {
    context: "enrollment",
    slice: "cmd_suspend_student",
    method: HttpMethod::Post,
    path: "/api/v1/students/{student_no}/suspend",
    status: SliceStatus::Planned,
    route,
});
