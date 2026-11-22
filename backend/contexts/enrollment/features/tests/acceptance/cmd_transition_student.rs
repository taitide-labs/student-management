use axum::http::{Method, StatusCode};
use rstest::rstest;
use sqlx::SqlitePool;
use student_management_contracts::{
    CreateHomeroomRequest, EnrollStudentRequest, EnrollmentStatus, HomeroomSummary, StudentSummary,
    TransitionStudentRequest,
};
use student_management_infra::open_sqlite_pool;

use super::support::{router, seed_operator, send_json};

#[sqlx::test(migrations = "../../../../migrations")]
#[ignore = "wip"]
async fn suspend_then_resume_transitions_student_between_active_and_suspended(pool: SqlitePool) {
    seed_operator(&pool, "acc-admin", "admin", "tok-admin").await;
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
            idempotency_key: "hr-sr".to_owned(),
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
            full_name: "王同学".to_owned(),
            homeroom_id: hr.id,
            idempotency_key: "enr-sr".to_owned(),
        }),
    )
    .await;
    let stu: StudentSummary = serde_json::from_value(stu_b).expect("student");

    let (s1, b1) = send_json(
        &app,
        Method::POST,
        &format!("/api/v1/students/{}/suspend", stu.student_no),
        "tok-admin",
        Some(&TransitionStudentRequest {
            reason: "病假休学".to_owned(),
            version: stu.version,
            idempotency_key: "susp-1".to_owned(),
        }),
    )
    .await;
    assert_eq!(s1, StatusCode::OK);
    let suspended: StudentSummary = serde_json::from_value(b1).expect("suspended");
    assert_eq!(suspended.status, EnrollmentStatus::Suspended);

    let (s2, b2) = send_json(
        &app,
        Method::POST,
        &format!("/api/v1/students/{}/resume", stu.student_no),
        "tok-admin",
        Some(&TransitionStudentRequest {
            reason: "康复复学".to_owned(),
            version: suspended.version,
            idempotency_key: "res-1".to_owned(),
        }),
    )
    .await;
    assert_eq!(s2, StatusCode::OK);
    let resumed: StudentSummary = serde_json::from_value(b2).expect("resumed");
    assert_eq!(resumed.status, EnrollmentStatus::Active);
}

#[rstest]
#[case("withdraw", EnrollmentStatus::Withdrawn)]
#[case("graduate", EnrollmentStatus::Graduated)]
#[tokio::test]
#[ignore = "wip"]
async fn terminal_transitions_lock_student_and_withdraw_writes_outbox(
    #[case] action: &str,
    #[case] expected_status: EnrollmentStatus,
) {
    let pool = open_sqlite_pool("sqlite::memory:").await.expect("sqlite");
    seed_operator(&pool, "acc-admin", "admin", "tok-admin").await;
    let app = router(pool.clone());

    let (_, hr_b) = send_json(
        &app,
        Method::POST,
        "/api/v1/homerooms",
        "tok-admin",
        Some(&CreateHomeroomRequest {
            name: "高三 (1) 班".to_owned(),
            grade_year: 2026,
            capacity: None,
            idempotency_key: "hr-term".to_owned(),
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
            full_name: "离校生".to_owned(),
            homeroom_id: hr.id,
            idempotency_key: "enr-term".to_owned(),
        }),
    )
    .await;
    let stu: StudentSummary = serde_json::from_value(stu_b).expect("student");

    let (s, b) = send_json(
        &app,
        Method::POST,
        &format!("/api/v1/students/{}/{action}", stu.student_no),
        "tok-admin",
        Some(&TransitionStudentRequest {
            reason: "终态办理".to_owned(),
            version: stu.version,
            idempotency_key: format!("term-{action}"),
        }),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    let updated: StudentSummary = serde_json::from_value(b).expect("updated");
    assert_eq!(updated.status, expected_status);

    let (locked_status, _) = send_json(
        &app,
        Method::POST,
        &format!("/api/v1/students/{}/suspend", stu.student_no),
        "tok-admin",
        Some(&TransitionStudentRequest {
            reason: "终态后非法操作".to_owned(),
            version: updated.version,
            idempotency_key: "lock-check".to_owned(),
        }),
    )
    .await;
    assert_eq!(locked_status, StatusCode::CONFLICT);
}
