use axum::http::{Method, StatusCode};
use sqlx::SqlitePool;
use student_management_contracts::{
    CreateHomeroomRequest, EnrollStudentRequest, EnrollmentStatus, HomeroomSummary, StudentSummary,
};

use super::support::{router, seed_operator, send_json};

#[sqlx::test(migrations = "../../../../migrations")]
async fn enroll_student_replays_idempotently_and_enforces_homeroom_capacity(pool: SqlitePool) {
    seed_operator(&pool, "acc-admin", "admin", "tok-admin").await;
    seed_operator(&pool, "acc-teacher", "teacher", "tok-teacher").await;
    let app = router(pool);

    let (hr_status, hr_body) = send_json(
        &app,
        Method::POST,
        "/api/v1/homerooms",
        "tok-admin",
        Some(&CreateHomeroomRequest {
            name: "高一 (2) 班".to_owned(),
            grade_year: 2026,
            capacity: Some(1),
            idempotency_key: "hr-cap1".to_owned(),
        }),
    )
    .await;
    assert_eq!(hr_status, StatusCode::CREATED);
    let hr: HomeroomSummary = serde_json::from_value(hr_body).expect("homeroom");

    let req = EnrollStudentRequest {
        full_name: "张晓明".to_owned(),
        homeroom_id: hr.id.clone(),
        idempotency_key: "enroll-zxm".to_owned(),
    };
    let (s1, b1) = send_json(
        &app,
        Method::POST,
        "/api/v1/students",
        "tok-admin",
        Some(&req),
    )
    .await;
    assert_eq!(s1, StatusCode::CREATED);
    let first: StudentSummary = serde_json::from_value(b1).expect("student");
    assert_eq!(first.status, EnrollmentStatus::Active);
    assert!(first.student_no.starts_with("2026"));

    let (s_replay, b_replay) = send_json(
        &app,
        Method::POST,
        "/api/v1/students",
        "tok-admin",
        Some(&req),
    )
    .await;
    assert_eq!(s_replay, StatusCode::CREATED);
    let replayed: StudentSummary = serde_json::from_value(b_replay).expect("replayed");
    assert_eq!(replayed.student_no, first.student_no);

    let (s_full, _) = send_json(
        &app,
        Method::POST,
        "/api/v1/students",
        "tok-admin",
        Some(&EnrollStudentRequest {
            full_name: "李四".to_owned(),
            homeroom_id: hr.id,
            idempotency_key: "enroll-ls".to_owned(),
        }),
    )
    .await;
    assert_eq!(s_full, StatusCode::CONFLICT);
}
