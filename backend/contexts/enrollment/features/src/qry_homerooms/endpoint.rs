use axum::{Json, Router, http::StatusCode, routing::get};
use student_management_contracts::{HomeroomSummary, ProblemDetails};
use student_management_infra::{HttpMethod, SliceDescriptor, SliceStatus, state::AppState};

#[utoipa::path(
    get,
    path = "/api/v1/homerooms",
    tag = "enrollment",
    responses((status = 200, description = "行政班列表", body = Vec<HomeroomSummary>))
)]
pub async fn list_homerooms() -> (StatusCode, Json<ProblemDetails>) {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(ProblemDetails::not_implemented("enrollment::qry_homerooms")),
    )
}

fn route() -> Router<AppState> {
    Router::new().route("/api/v1/homerooms", get(list_homerooms))
}

inventory::submit!(SliceDescriptor {
    context: "enrollment",
    slice: "qry_homerooms",
    method: HttpMethod::Get,
    path: "/api/v1/homerooms",
    status: SliceStatus::Planned,
    route,
});
