use axum::{Json, Router, http::StatusCode, routing::post};
use student_management_contracts::{CourseSummary, CreateCourseRequest, ProblemDetails};
use student_management_infra::{HttpMethod, SliceDescriptor, SliceStatus, state::AppState};

#[utoipa::path(
    post,
    path = "/api/v1/courses",
    tag = "grading",
    request_body = CreateCourseRequest,
    responses((status = 201, description = "创建课程成功", body = CourseSummary))
)]
pub async fn create_course(
    Json(_req): Json<CreateCourseRequest>,
) -> (StatusCode, Json<ProblemDetails>) {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(ProblemDetails::not_implemented(
            "grading::cmd_create_course",
        )),
    )
}

fn route() -> Router<AppState> {
    Router::new().route("/api/v1/courses", post(create_course))
}

inventory::submit!(SliceDescriptor {
    context: "grading",
    slice: "cmd_create_course",
    method: HttpMethod::Post,
    path: "/api/v1/courses",
    status: SliceStatus::Planned,
    route,
});
