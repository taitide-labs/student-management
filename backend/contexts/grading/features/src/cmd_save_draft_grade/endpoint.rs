use axum::{Json, Router, http::StatusCode, routing::put};
use student_management_contracts::{GradeEntrySummary, ProblemDetails, SaveDraftGradeRequest};
use student_management_infra::{HttpMethod, SliceDescriptor, SliceStatus, state::AppState};

#[utoipa::path(
    put,
    path = "/api/v1/grades/draft",
    tag = "grading",
    request_body = SaveDraftGradeRequest,
    responses(
        (status = 200, description = "暂存草稿成绩成功", body = GradeEntrySummary),
        (status = 409, description = "成绩已锁定或学生非在籍", body = ProblemDetails)
    )
)]
pub async fn save_draft_grade(
    Json(_req): Json<SaveDraftGradeRequest>,
) -> (StatusCode, Json<ProblemDetails>) {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(ProblemDetails::not_implemented(
            "grading::cmd_save_draft_grade",
        )),
    )
}

fn route() -> Router<AppState> {
    Router::new().route("/api/v1/grades/draft", put(save_draft_grade))
}

inventory::submit!(SliceDescriptor {
    context: "grading",
    slice: "cmd_save_draft_grade",
    method: HttpMethod::Put,
    path: "/api/v1/grades/draft",
    status: SliceStatus::Planned,
    route,
});
