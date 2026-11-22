use axum::http::{Method, StatusCode};
use sqlx::SqlitePool;
use student_management_contracts::{
    CreateHomeroomRequest, EnrollStudentRequest, HomeroomSummary, StudentDetailPayload,
    StudentSummary,
};

use super::support::{router, seed_operator, send_json};

#[sqlx::test(migrations = "../../../../migrations")]
async fn student_detail_returns_change_history_and_enforces_student_scope(pool: SqlitePool) {
    seed_operator(&pool, "acc-admin", "admin", "tok-admin").await;
    seed_operator(&pool, "acc-stu", "student", "tok-stu").await;
    let app = router(pool);

    let (_, hr_b) = send_json(
        &app,
        Method::POST,
        "/api/v1/homerooms",
        "tok-admin",
        Some(&CreateHomeroomRequest {
            name: "高一 (1) 班".to_owned(),
            grade_year: 2026,
            capacity: None,
            idempotency_key: "hr-det".to_owned(),
        }),
    )
    .await;
    let hr: HomeroomSummary = serde_json::from_value(hr_b).expect("hr");
    let (_, stu_b) = send_json(
        &app,
        Method::POST,
        "/api/v1/students",
        "tok-admin",
        Some(&EnrollStudentRequest {
            full_name: "详情生".to_owned(),
            homeroom_id: hr.id,
            idempotency_key: "enr-det".to_owned(),
        }),
    )
    .await;
    let stu: StudentSummary = serde_json::from_value(stu_b).expect("stu");

    let (ok_s, det_b) = send_json::<()>(
        &app,
        Method::GET,
        &format!("/api/v1/students/{}", stu.student_no),
        "tok-admin",
        None,
    )
    .await;
    assert_eq!(ok_s, StatusCode::OK);
    let detail: StudentDetailPayload = serde_json::from_value(det_b).expect("detail");
    assert_eq!(detail.changes.len(), 1);

    let (forbid_s, _) = send_json::<()>(
        &app,
        Method::GET,
        &format!("/api/v1/students/{}", stu.student_no),
        "tok-stu",
        None,
    )
    .await;
    assert_eq!(forbid_s, StatusCode::FORBIDDEN);
}
