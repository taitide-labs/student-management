use axum::{Json, Router, extract::Query, http::StatusCode, routing::get};
use serde::Deserialize;
use student_management_contracts::{GradeEntrySummary, ProblemDetails};
use student_management_infra::{HttpMethod, SliceDescriptor, SliceStatus, state::AppState};

#[derive(Debug, Deserialize)]
pub struct GradesQuery {
    pub student_no: Option<String>,
}

#[utoipa::path(
    get,
    path = "/api/v1/grades",
    tag = "grading",
    params(("student_no" = Option<String>, Query, description = "按学号过滤成绩")),
    responses((status = 200, description = "成绩列表", body = Vec<GradeEntrySummary>))
)]
pub async fn list_grades(Query(_query): Query<GradesQuery>) -> (StatusCode, Json<ProblemDetails>) {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(ProblemDetails::not_implemented("grading::qry_grades")),
    )
}

fn route() -> Router<AppState> {
    Router::new().route("/api/v1/grades", get(list_grades))
}

inventory::submit!(SliceDescriptor {
    context: "grading",
    slice: "qry_grades",
    method: HttpMethod::Get,
    path: "/api/v1/grades",
    status: SliceStatus::Planned,
    route,
});
