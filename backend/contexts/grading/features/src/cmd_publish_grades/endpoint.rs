use axum::{Json, Router, http::StatusCode, routing::post};
use student_management_contracts::{GradeEntrySummary, ProblemDetails, PublishGradesRequest};
use student_management_infra::{HttpMethod, SliceDescriptor, SliceStatus, state::AppState};

#[utoipa::path(
    post,
    path = "/api/v1/grades/publish",
    tag = "grading",
    request_body = PublishGradesRequest,
    responses((status = 200, description = "发布并锁定成绩成功", body = Vec<GradeEntrySummary>))
)]
pub async fn publish_grades(
    Json(_req): Json<PublishGradesRequest>,
) -> (StatusCode, Json<ProblemDetails>) {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(ProblemDetails::not_implemented(
            "grading::cmd_publish_grades",
        )),
    )
}

fn route() -> Router<AppState> {
    Router::new().route("/api/v1/grades/publish", post(publish_grades))
}

inventory::submit!(SliceDescriptor {
    context: "grading",
    slice: "cmd_publish_grades",
    method: HttpMethod::Post,
    path: "/api/v1/grades/publish",
    status: SliceStatus::Planned,
    route,
});
