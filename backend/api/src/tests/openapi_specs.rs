use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use student_management_runtime::build_app_state;
use tower::ServiceExt;

use crate::{build_router, build_router_with_planned, export_openapi_pretty};

#[test]
fn openapi_schema_matches_frozen_contracts_file() {
    let generated = export_openapi_pretty().expect("serialize openapi");
    let committed = include_str!("../../../../contracts/openapi.json");
    assert_eq!(generated.trim(), committed.trim());
}

#[tokio::test]
async fn health_works_and_planned_slices_stay_hidden_in_production_router() {
    let state = build_app_state("sqlite::memory:")
        .await
        .expect("build runtime state");

    let prod = build_router(state.clone());
    let health_resp = prod
        .clone()
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("health response");
    assert_eq!(health_resp.status(), StatusCode::OK);

    let hidden_resp = prod
        .oneshot(
            Request::builder()
                .uri("/api/v1/homerooms")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("hidden planned slice");
    assert_eq!(hidden_resp.status(), StatusCode::NOT_FOUND);

    let dev = build_router_with_planned(state, true);
    let planned_resp = dev
        .oneshot(
            Request::builder()
                .uri("/api/v1/homerooms")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("planned response");
    assert_eq!(planned_resp.status(), StatusCode::NOT_IMPLEMENTED);
}
