use axum::{Json, Router, http::StatusCode, routing::post};
use student_management_contracts::{CreateHomeroomRequest, HomeroomSummary, ProblemDetails};
use student_management_infra::{HttpMethod, SliceDescriptor, SliceStatus, state::AppState};

#[utoipa::path(
    post,
    path = "/api/v1/homerooms",
    tag = "enrollment",
    request_body = CreateHomeroomRequest,
    responses(
        (status = 201, description = "创建行政班成功", body = HomeroomSummary),
        (status = 409, description = "班级名称重复", body = ProblemDetails)
    )
)]
pub async fn create_homeroom(
    Json(_req): Json<CreateHomeroomRequest>,
) -> (StatusCode, Json<ProblemDetails>) {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(ProblemDetails::not_implemented(
            "enrollment::cmd_create_homeroom",
        )),
    )
}

fn route() -> Router<AppState> {
    Router::new().route("/api/v1/homerooms", post(create_homeroom))
}

inventory::submit!(SliceDescriptor {
    context: "enrollment",
    slice: "cmd_create_homeroom",
    method: HttpMethod::Post,
    path: "/api/v1/homerooms",
    status: SliceStatus::Planned,
    route,
});
