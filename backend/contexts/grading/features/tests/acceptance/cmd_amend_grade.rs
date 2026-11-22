use axum::http::{Method, StatusCode};
use rust_decimal::Decimal;
use sqlx::SqlitePool;
use student_management_contracts::{
    AmendGradeRequest, GradeAmendmentRecord, GradeEntrySummary, GradeStatus,
};

use super::support::{FakeEnrollmentPort, router, seed_operator, send_json};

#[sqlx::test(migrations = "../../../../migrations")]
#[ignore = "wip"]
async fn amend_grade_requires_admin_and_records_immutable_audit_trail(pool: SqlitePool) {
    seed_operator(&pool, "acc-admin", "admin", "tok-admin").await;
    seed_operator(&pool, "acc-teacher", "teacher", "tok-teacher").await;
    sqlx::query(
        "INSERT INTO courses (id, code, name, max_score, pass_score, version, created_at) VALUES ('c-1', 'ENG101', '大学英语', '100', '60', 1, '2026-11-22T12:00:00Z')",
    )
    .execute(&pool)
    .await
    .expect("seed course");
    sqlx::query(
        "INSERT INTO teaching_assignments (id, course_id, homeroom_id, term, teacher_account_id, teacher_name, version, created_at) VALUES ('t-1', 'c-1', 'hr-1', '2026-秋', 'acc-teacher', '林老师', 1, '2026-11-22T12:00:00Z')",
    )
    .execute(&pool)
    .await
    .expect("seed teaching");
    sqlx::query(
        "INSERT INTO grade_entries (id, teaching_id, student_no, score, status, version, updated_at) VALUES ('g-1', 't-1', '20261001', '82', 'published', 2, '2026-11-22T12:00:00Z')",
    )
    .execute(&pool)
    .await
    .expect("seed grade");

    let app = router(
        pool,
        FakeEnrollmentPort::with_active_student("20261001", "hr-1"),
    );

    let (forbid_status, _) = send_json(
        &app,
        Method::POST,
        "/api/v1/grades/g-1/amend",
        "tok-teacher",
        Some(&AmendGradeRequest {
            new_score: Decimal::from(90),
            reason: "教师越权改分".to_owned(),
            version: 2,
            idempotency_key: "am-t".to_owned(),
        }),
    )
    .await;
    assert_eq!(forbid_status, StatusCode::FORBIDDEN);

    let (ok_status, ok_b) = send_json(
        &app,
        Method::POST,
        "/api/v1/grades/g-1/amend",
        "tok-admin",
        Some(&AmendGradeRequest {
            new_score: Decimal::from(91),
            reason: "统分复核加分".to_owned(),
            version: 2,
            idempotency_key: "am-a".to_owned(),
        }),
    )
    .await;
    assert_eq!(ok_status, StatusCode::OK);
    let amended: GradeEntrySummary = serde_json::from_value(ok_b).expect("amended");
    assert_eq!(amended.score, Decimal::from(91));
    assert_eq!(amended.status, GradeStatus::Published);

    let (audit_status, audit_b) = send_json::<()>(
        &app,
        Method::GET,
        "/api/v1/grade-amendments?grade_id=g-1",
        "tok-admin",
        None,
    )
    .await;
    assert_eq!(audit_status, StatusCode::OK);
    let records: Vec<GradeAmendmentRecord> = serde_json::from_value(audit_b).expect("amendments");
    assert_eq!(records.len(), 1);
    assert_eq!(
        records.first().map(|r| r.previous_score),
        Some(Decimal::from(82))
    );
}
