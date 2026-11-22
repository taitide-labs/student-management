use axum::{Json, Router, extract::Query, http::StatusCode, routing::get};
use serde::Deserialize;
use student_management_contracts::{GradeAmendmentRecord, ProblemDetails};
use student_management_infra::{HttpMethod, SliceDescriptor, SliceStatus, state::AppState};

#[derive(Debug, Deserialize)]
pub struct GradeAmendmentsQuery {
    pub grade_id: Option<String>,
}

#[utoipa::path(
    get,
    path = "/api/v1/grade-amendments",
    tag = "grading",
    params(("grade_id" = Option<String>, Query, description = "按成绩记录过滤")),
    responses((status = 200, description = "成绩修订审计列表", body = Vec<GradeAmendmentRecord>))
)]
pub async fn list_grade_amendments(
    Query(_query): Query<GradeAmendmentsQuery>,
) -> (StatusCode, Json<ProblemDetails>) {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(ProblemDetails::not_implemented(
            "grading::qry_grade_amendments",
        )),
    )
}

fn route() -> Router<AppState> {
    Router::new().route("/api/v1/grade-amendments", get(list_grade_amendments))
}

inventory::submit!(SliceDescriptor {
    context: "grading",
    slice: "qry_grade_amendments",
    method: HttpMethod::Get,
    path: "/api/v1/grade-amendments",
    status: SliceStatus::Planned,
    route,
});
