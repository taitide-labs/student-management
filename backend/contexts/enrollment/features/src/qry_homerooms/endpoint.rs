use axum::{Json, Router, extract::State, http::StatusCode, routing::get};
use student_management_contracts::{HomeroomSummary, ProblemDetails};
use student_management_infra::{AppState, HttpMethod, SliceDescriptor, SliceStatus};

use super::handler;

#[utoipa::path(
    get,
    path = "/api/v1/homerooms",
    tag = "enrollment",
    responses((status = 200, description = "行政班列表", body = Vec<HomeroomSummary>))
)]
pub async fn list_homerooms(
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<Vec<HomeroomSummary>>), (StatusCode, Json<ProblemDetails>)> {
    let homerooms = handler::execute(&state).await.map_err(|problem| {
        let status =
            StatusCode::from_u16(problem.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        (status, Json(problem))
    })?;
    Ok((StatusCode::OK, Json(homerooms)))
}

fn route() -> Router<AppState> {
    Router::new().route("/api/v1/homerooms", get(list_homerooms))
}

inventory::submit!(SliceDescriptor {
    context: "enrollment",
    slice: "qry_homerooms",
    method: HttpMethod::Get,
    path: "/api/v1/homerooms",
    status: SliceStatus::Active,
    route,
});
