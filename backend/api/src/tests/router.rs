use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use student_management_infra::{SliceStatus, registered_slices};
use student_management_runtime::build_app_state;
use tower::ServiceExt;

use crate::build_router;

#[tokio::test]
async fn api_registers_all_27_slices_and_hides_planned_in_production() {
    let state = build_app_state("sqlite::memory:")
        .await
        .expect("build runtime state");
    let app = build_router(state);
    let slices = registered_slices();
    assert_eq!(slices.len(), 27);
    assert!(slices.iter().all(|s| s.status == SliceStatus::Active));

    let health_resp = app
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

    let amendments_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/grade-amendments")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("active slice auth check");
    assert_eq!(amendments_resp.status(), StatusCode::UNAUTHORIZED);

    let missing_resp = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/nonexistent")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("unknown route check");
    assert_eq!(missing_resp.status(), StatusCode::NOT_FOUND);
}
