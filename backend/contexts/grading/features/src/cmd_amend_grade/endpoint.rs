use axum::{Json, Router, extract::Path, http::StatusCode, routing::post};
use student_management_contracts::{AmendGradeRequest, GradeEntrySummary, ProblemDetails};
use student_management_infra::{HttpMethod, SliceDescriptor, SliceStatus, state::AppState};

#[utoipa::path(
    post,
    path = "/api/v1/grades/{grade_id}/amend",
    tag = "grading",
    params(("grade_id" = String, Path, description = "成绩记录标识")),
    request_body = AmendGradeRequest,
    responses((status = 200, description = "管理员修订成绩成功", body = GradeEntrySummary))
)]
pub async fn amend_grade(
    Path(_grade_id): Path<String>,
    Json(_req): Json<AmendGradeRequest>,
) -> (StatusCode, Json<ProblemDetails>) {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(ProblemDetails::not_implemented("grading::cmd_amend_grade")),
    )
}

fn route() -> Router<AppState> {
    Router::new().route("/api/v1/grades/{grade_id}/amend", post(amend_grade))
}

inventory::submit!(SliceDescriptor {
    context: "grading",
    slice: "cmd_amend_grade",
    method: HttpMethod::Post,
    path: "/api/v1/grades/{grade_id}/amend",
    status: SliceStatus::Planned,
    route,
});
