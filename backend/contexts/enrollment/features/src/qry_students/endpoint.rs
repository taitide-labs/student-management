use axum::{Json, Router, extract::Query, http::StatusCode, routing::get};
use serde::Deserialize;
use student_management_contracts::{EnrollmentStatus, ProblemDetails, StudentSummary};
use student_management_infra::{HttpMethod, SliceDescriptor, SliceStatus, state::AppState};

#[derive(Debug, Deserialize)]
pub struct StudentsQuery {
    pub q: Option<String>,
    pub status: Option<EnrollmentStatus>,
}

#[utoipa::path(
    get,
    path = "/api/v1/students",
    tag = "enrollment",
    params(
        ("q" = Option<String>, Query, description = "学号或姓名关键字"),
        ("status" = Option<EnrollmentStatus>, Query, description = "学籍状态过滤")
    ),
    responses((status = 200, description = "学生列表", body = Vec<StudentSummary>))
)]
pub async fn list_students(
    Query(_query): Query<StudentsQuery>,
) -> (StatusCode, Json<ProblemDetails>) {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(ProblemDetails::not_implemented("enrollment::qry_students")),
    )
}

fn route() -> Router<AppState> {
    Router::new().route("/api/v1/students", get(list_students))
}

inventory::submit!(SliceDescriptor {
    context: "enrollment",
    slice: "qry_students",
    method: HttpMethod::Get,
    path: "/api/v1/students",
    status: SliceStatus::Planned,
    route,
});
