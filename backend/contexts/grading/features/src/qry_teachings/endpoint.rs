use axum::{Json, Router, http::StatusCode, routing::get};
use student_management_contracts::{ProblemDetails, TeachingSummary};
use student_management_infra::{HttpMethod, SliceDescriptor, SliceStatus, state::AppState};

#[utoipa::path(
    get,
    path = "/api/v1/teachings",
    tag = "grading",
    responses((status = 200, description = "教学任务列表", body = Vec<TeachingSummary>))
)]
pub async fn list_teachings() -> (StatusCode, Json<ProblemDetails>) {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(ProblemDetails::not_implemented("grading::qry_teachings")),
    )
}

fn route() -> Router<AppState> {
    Router::new().route("/api/v1/teachings", get(list_teachings))
}

inventory::submit!(SliceDescriptor {
    context: "grading",
    slice: "qry_teachings",
    method: HttpMethod::Get,
    path: "/api/v1/teachings",
    status: SliceStatus::Planned,
    route,
});
