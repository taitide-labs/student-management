use axum::http::{Method, StatusCode};
use sqlx::SqlitePool;
use student_management_contracts::{
    CreateHomeroomRequest, EnrollStudentRequest, HomeroomSummary, UpdateHomeroomCapacityRequest,
};

use super::support::{router, seed_operator, send_json};

#[sqlx::test(migrations = "../../../../migrations")]
#[ignore = "wip"]
async fn create_homeroom_and_reject_capacity_below_active_count(pool: SqlitePool) {
    seed_operator(&pool, "acc-admin", "admin", "tok-admin").await;
    let app = router(pool);

    let (status, body) = send_json(
        &app,
        Method::POST,
        "/api/v1/homerooms",
        "tok-admin",
        Some(&CreateHomeroomRequest {
            name: "高一 (1) 班".to_owned(),
            grade_year: 2026,
            capacity: Some(2),
            idempotency_key: "hr-1".to_owned(),
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let hr: HomeroomSummary = serde_json::from_value(body).expect("homeroom summary");

    for (idx, name) in ["学生甲", "学生乙"].iter().enumerate() {
        let (enroll_status, _) = send_json(
            &app,
            Method::POST,
            "/api/v1/students",
            "tok-admin",
            Some(&EnrollStudentRequest {
                full_name: (*name).to_owned(),
                homeroom_id: hr.id.clone(),
                idempotency_key: format!("enr-{idx}"),
            }),
        )
        .await;
        assert_eq!(enroll_status, StatusCode::CREATED);
    }

    let (shrink_status, _) = send_json(
        &app,
        Method::PATCH,
        &format!("/api/v1/homerooms/{}/capacity", hr.id),
        "tok-admin",
        Some(&UpdateHomeroomCapacityRequest {
            capacity: Some(1),
            version: hr.version,
            idempotency_key: "cap-shrink".to_owned(),
        }),
    )
    .await;
    assert_eq!(shrink_status, StatusCode::CONFLICT);
}
