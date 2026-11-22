use axum::http::{Method, StatusCode};
use sqlx::SqlitePool;
use student_management_contracts::{
    CreateHomeroomRequest, EnrollStudentRequest, EnrollmentStatus, HomeroomSummary,
    ReverseEnrollmentRequest, StudentDetailPayload, StudentSummary, TransitionStudentRequest,
};

use super::support::{router, seed_operator, send_json};

#[sqlx::test(migrations = "../../../../migrations")]
async fn reverse_enrollment_recovers_terminal_student_with_audit_trail(pool: SqlitePool) {
    seed_operator(&pool, "acc-admin", "admin", "tok-admin").await;
    let app = router(pool);

    let (_, hr_b) = send_json(
        &app,
        Method::POST,
        "/api/v1/homerooms",
        "tok-admin",
        Some(&CreateHomeroomRequest {
            name: "高一 (3) 班".to_owned(),
            grade_year: 2026,
            capacity: None,
            idempotency_key: "hr-rev".to_owned(),
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
            full_name: "冲正生".to_owned(),
            homeroom_id: hr.id,
            idempotency_key: "enr-rev".to_owned(),
        }),
    )
    .await;
    let stu: StudentSummary = serde_json::from_value(stu_b).expect("student");

    let (_, w_b) = send_json(
        &app,
        Method::POST,
        &format!("/api/v1/students/{}/withdraw", stu.student_no),
        "tok-admin",
        Some(&TransitionStudentRequest {
            reason: "误录退学".to_owned(),
            version: stu.version,
            idempotency_key: "w-rev".to_owned(),
        }),
    )
    .await;
    let withdrawn: StudentSummary = serde_json::from_value(w_b).expect("withdrawn");

    let (rev_status, rev_body) = send_json(
        &app,
        Method::POST,
        &format!("/api/v1/students/{}/reverse", stu.student_no),
        "tok-admin",
        Some(&ReverseEnrollmentRequest {
            target_status: EnrollmentStatus::Active,
            reason: "核实为同名误操作，冲正恢复在籍".to_owned(),
            version: withdrawn.version,
            idempotency_key: "rev-1".to_owned(),
        }),
    )
    .await;
    assert_eq!(rev_status, StatusCode::OK);
    let restored: StudentSummary = serde_json::from_value(rev_body).expect("restored");
    assert_eq!(restored.status, EnrollmentStatus::Active);

    let (_, detail_b) = send_json::<()>(
        &app,
        Method::GET,
        &format!("/api/v1/students/{}", stu.student_no),
        "tok-admin",
        None,
    )
    .await;
    let detail: StudentDetailPayload = serde_json::from_value(detail_b).expect("detail");
    assert_eq!(detail.changes.len(), 3);
}
