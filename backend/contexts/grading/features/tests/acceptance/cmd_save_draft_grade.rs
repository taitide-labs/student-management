use axum::http::{Method, StatusCode};
use rust_decimal::Decimal;
use sqlx::SqlitePool;
use student_management_contracts::{
    AssignTeachingRequest, CourseSummary, CreateCourseRequest, GradeEntrySummary, GradeStatus,
    PublishGradesRequest, SaveDraftGradeRequest, TeachingSummary,
};

use super::support::{FakeEnrollmentPort, router, seed_operator, send_json};

#[sqlx::test(migrations = "../../../../migrations")]
#[ignore = "wip"]
async fn teacher_saves_draft_rejects_withdrawn_and_locks_after_publish(pool: SqlitePool) {
    seed_operator(&pool, "acc-admin", "admin", "tok-admin").await;
    seed_operator(&pool, "acc-teacher", "teacher", "tok-teacher").await;
    let app = router(
        pool,
        FakeEnrollmentPort::with_active_student("20261001", "hr-1"),
    );

    let (_, c_b) = send_json(
        &app,
        Method::POST,
        "/api/v1/courses",
        "tok-admin",
        Some(&CreateCourseRequest {
            code: "PHY101".to_owned(),
            name: "大学物理".to_owned(),
            max_score: Decimal::from(100),
            pass_score: Decimal::from(60),
            idempotency_key: "c-phy".to_owned(),
        }),
    )
    .await;
    let course: CourseSummary = serde_json::from_value(c_b).expect("course");

    let (_, t_b) = send_json(
        &app,
        Method::POST,
        "/api/v1/teachings",
        "tok-admin",
        Some(&AssignTeachingRequest {
            course_id: course.id,
            homeroom_id: "hr-1".to_owned(),
            term: "2026-秋".to_owned(),
            teacher_account_id: "acc-teacher".to_owned(),
            idempotency_key: "t-phy".to_owned(),
        }),
    )
    .await;
    let teaching: TeachingSummary = serde_json::from_value(t_b).expect("teaching");

    let (draft_status, draft_b) = send_json(
        &app,
        Method::PUT,
        "/api/v1/grades/draft",
        "tok-teacher",
        Some(&SaveDraftGradeRequest {
            teaching_id: teaching.id.clone(),
            student_no: "20261001".to_owned(),
            score: Decimal::from(88),
            idempotency_key: "g-1".to_owned(),
        }),
    )
    .await;
    assert_eq!(draft_status, StatusCode::OK);
    let draft: GradeEntrySummary = serde_json::from_value(draft_b).expect("draft");
    assert_eq!(draft.status, GradeStatus::Draft);

    let (pub_status, _) = send_json(
        &app,
        Method::POST,
        "/api/v1/grades/publish",
        "tok-teacher",
        Some(&PublishGradesRequest {
            teaching_id: teaching.id.clone(),
            idempotency_key: "pub-1".to_owned(),
        }),
    )
    .await;
    assert_eq!(pub_status, StatusCode::OK);

    let (locked_status, _) = send_json(
        &app,
        Method::PUT,
        "/api/v1/grades/draft",
        "tok-teacher",
        Some(&SaveDraftGradeRequest {
            teaching_id: teaching.id,
            student_no: "20261001".to_owned(),
            score: Decimal::from(95),
            idempotency_key: "g-2".to_owned(),
        }),
    )
    .await;
    assert_eq!(locked_status, StatusCode::CONFLICT);
}

#[sqlx::test(migrations = "../../../../migrations")]
async fn save_draft_grade_rejects_withdrawn_student(pool: SqlitePool) {
    seed_operator(&pool, "acc-teacher", "teacher", "tok-teacher").await;
    let app = router(
        pool,
        FakeEnrollmentPort::with_withdrawn_student("20261099", "hr-1"),
    );

    let (status, _) = send_json(
        &app,
        Method::PUT,
        "/api/v1/grades/draft",
        "tok-teacher",
        Some(&SaveDraftGradeRequest {
            teaching_id: "t-any".to_owned(),
            student_no: "20261099".to_owned(),
            score: Decimal::from(70),
            idempotency_key: "g-w".to_owned(),
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
}
