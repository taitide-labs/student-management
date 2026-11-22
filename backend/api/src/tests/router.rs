use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use student_management_infra::registered_slices;
use student_management_runtime::build_app_state;
use tower::ServiceExt;

use crate::{build_router, build_router_with_planned};

#[tokio::test]
async fn api_registers_all_27_slices_and_hides_planned_in_production() {
    let state = build_app_state("sqlite::memory:")
        .await
        .expect("build runtime state");
    assert_eq!(registered_slices().len(), 27);

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
