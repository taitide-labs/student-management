use axum::{Json, Router, extract::Path, http::StatusCode, routing::get};
use student_management_contracts::{ProblemDetails, StudentDetailPayload};
use student_management_infra::{HttpMethod, SliceDescriptor, SliceStatus, state::AppState};

#[utoipa::path(
    get,
    path = "/api/v1/students/{student_no}",
    tag = "enrollment",
    params(("student_no" = String, Path, description = "学号")),
    responses(
        (status = 200, description = "学生档案与异动记录", body = StudentDetailPayload),
        (status = 404, description = "学生不存在", body = ProblemDetails)
    )
)]
pub async fn student_detail(Path(_student_no): Path<String>) -> (StatusCode, Json<ProblemDetails>) {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(ProblemDetails::not_implemented(
            "enrollment::qry_student_detail",
        )),
    )
}

fn route() -> Router<AppState> {
    Router::new().route("/api/v1/students/{student_no}", get(student_detail))
}

inventory::submit!(SliceDescriptor {
    context: "enrollment",
    slice: "qry_student_detail",
    method: HttpMethod::Get,
    path: "/api/v1/students/{student_no}",
    status: SliceStatus::Planned,
    route,
});
