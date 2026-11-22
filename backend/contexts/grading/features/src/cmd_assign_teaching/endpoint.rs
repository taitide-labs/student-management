use axum::{Json, Router, http::StatusCode, routing::post};
use student_management_contracts::{AssignTeachingRequest, ProblemDetails, TeachingSummary};
use student_management_infra::{HttpMethod, SliceDescriptor, SliceStatus, state::AppState};

#[utoipa::path(
    post,
    path = "/api/v1/teachings",
    tag = "grading",
    request_body = AssignTeachingRequest,
    responses((status = 201, description = "分派教学任务成功", body = TeachingSummary))
)]
pub async fn assign_teaching(
    Json(_req): Json<AssignTeachingRequest>,
) -> (StatusCode, Json<ProblemDetails>) {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(ProblemDetails::not_implemented(
            "grading::cmd_assign_teaching",
        )),
    )
}

fn route() -> Router<AppState> {
    Router::new().route("/api/v1/teachings", post(assign_teaching))
}

inventory::submit!(SliceDescriptor {
    context: "grading",
    slice: "cmd_assign_teaching",
    method: HttpMethod::Post,
    path: "/api/v1/teachings",
    status: SliceStatus::Planned,
    route,
});
