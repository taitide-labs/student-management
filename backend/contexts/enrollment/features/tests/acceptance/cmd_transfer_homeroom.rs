use axum::http::{Method, StatusCode};
use sqlx::SqlitePool;
use student_management_contracts::{
    CreateHomeroomRequest, EnrollStudentRequest, HomeroomSummary, StudentSummary,
    TransferHomeroomRequest,
};

use super::support::{router, seed_operator, send_json};

#[sqlx::test(migrations = "../../../../migrations")]
async fn transfer_homeroom_updates_assignment_and_rejects_full_target(pool: SqlitePool) {
    seed_operator(&pool, "acc-admin", "admin", "tok-admin").await;
    let app = router(pool);

    let (_, b1) = send_json(
        &app,
        Method::POST,
        "/api/v1/homerooms",
        "tok-admin",
        Some(&CreateHomeroomRequest {
            name: "高一 (1) 班".to_owned(),
            grade_year: 2026,
            capacity: Some(10),
            idempotency_key: "hr-a".to_owned(),
        }),
    )
    .await;
    let (_, b2) = send_json(
        &app,
        Method::POST,
        "/api/v1/homerooms",
        "tok-admin",
        Some(&CreateHomeroomRequest {
            name: "高一 (2) 班".to_owned(),
            grade_year: 2026,
            capacity: Some(1),
            idempotency_key: "hr-b".to_owned(),
        }),
    )
    .await;
    let hr_a: HomeroomSummary = serde_json::from_value(b1).expect("hr a");
    let hr_b: HomeroomSummary = serde_json::from_value(b2).expect("hr b");

    let (_, stu_body) = send_json(
        &app,
        Method::POST,
        "/api/v1/students",
        "tok-admin",
        Some(&EnrollStudentRequest {
            full_name: "转班生".to_owned(),
            homeroom_id: hr_a.id,
            idempotency_key: "enr-t".to_owned(),
        }),
    )
    .await;
    let student: StudentSummary = serde_json::from_value(stu_body).expect("student");

    let (ok_status, moved_body) = send_json(
        &app,
        Method::POST,
        &format!("/api/v1/students/{}/transfer", student.student_no),
        "tok-admin",
        Some(&TransferHomeroomRequest {
            target_homeroom_id: hr_b.id,
            reason: "走读调班".to_owned(),
            version: student.version,
            idempotency_key: "tr-1".to_owned(),
        }),
    )
    .await;
    assert_eq!(ok_status, StatusCode::OK);
    let moved: StudentSummary = serde_json::from_value(moved_body).expect("moved");
    assert_eq!(moved.homeroom_name, "高一 (2) 班");
}
