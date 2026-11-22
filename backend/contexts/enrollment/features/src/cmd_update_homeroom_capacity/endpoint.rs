use axum::{Json, Router, extract::Path, http::StatusCode, routing::patch};
use student_management_contracts::{
    HomeroomSummary, ProblemDetails, UpdateHomeroomCapacityRequest,
};
use student_management_infra::{HttpMethod, SliceDescriptor, SliceStatus, state::AppState};

#[utoipa::path(
    patch,
    path = "/api/v1/homerooms/{homeroom_id}/capacity",
    tag = "enrollment",
    params(("homeroom_id" = String, Path, description = "行政班标识")),
    request_body = UpdateHomeroomCapacityRequest,
    responses(
        (status = 200, description = "调整容量成功", body = HomeroomSummary),
        (status = 409, description = "低于在籍人数或版本冲突", body = ProblemDetails)
    )
)]
pub async fn update_homeroom_capacity(
    Path(_homeroom_id): Path<String>,
    Json(_req): Json<UpdateHomeroomCapacityRequest>,
) -> (StatusCode, Json<ProblemDetails>) {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(ProblemDetails::not_implemented(
            "enrollment::cmd_update_homeroom_capacity",
        )),
    )
}

fn route() -> Router<AppState> {
    Router::new().route(
        "/api/v1/homerooms/{homeroom_id}/capacity",
        patch(update_homeroom_capacity),
    )
}

inventory::submit!(SliceDescriptor {
    context: "enrollment",
    slice: "cmd_update_homeroom_capacity",
    method: HttpMethod::Patch,
    path: "/api/v1/homerooms/{homeroom_id}/capacity",
    status: SliceStatus::Planned,
    route,
});
