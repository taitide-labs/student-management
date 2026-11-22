use axum::{Json, Router, routing::get};
use serde::Serialize;
use student_management_infra::{AppState, SliceStatus, registered_slices};
use student_management_runtime::link_all_contexts;
use tower_http::cors::CorsLayer;

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: &'static str,
    service: &'static str,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "student-management-api",
    })
}

pub fn build_router(state: AppState) -> Router {
    build_router_with_planned(state, false)
}

pub fn build_router_with_planned(state: AppState, include_planned: bool) -> Router {
    link_all_contexts();
    let mut router = Router::new().route("/health", get(health));

    for descriptor in registered_slices() {
        if include_planned || descriptor.status == SliceStatus::Active {
            router = router.merge((descriptor.route)());
        }
    }

    router.layer(CorsLayer::permissive()).with_state(state)
}
