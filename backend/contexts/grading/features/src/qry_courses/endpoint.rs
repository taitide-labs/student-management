use axum::{Json, Router, http::StatusCode, routing::get};
use student_management_contracts::{CourseSummary, ProblemDetails};
use student_management_infra::{HttpMethod, SliceDescriptor, SliceStatus, state::AppState};

#[utoipa::path(
    get,
    path = "/api/v1/courses",
    tag = "grading",
    responses((status = 200, description = "课程列表", body = Vec<CourseSummary>))
)]
pub async fn list_courses() -> (StatusCode, Json<ProblemDetails>) {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(ProblemDetails::not_implemented("grading::qry_courses")),
    )
}

fn route() -> Router<AppState> {
    Router::new().route("/api/v1/courses", get(list_courses))
}

inventory::submit!(SliceDescriptor {
    context: "grading",
    slice: "qry_courses",
    method: HttpMethod::Get,
    path: "/api/v1/courses",
    status: SliceStatus::Planned,
    route,
});
