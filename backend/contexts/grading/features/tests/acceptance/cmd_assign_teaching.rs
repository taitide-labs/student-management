use axum::http::{Method, StatusCode};
use rust_decimal::Decimal;
use sqlx::SqlitePool;
use student_management_contracts::{
    AssignTeachingRequest, CourseSummary, CreateCourseRequest, TeachingSummary,
};

use super::support::{FakeEnrollmentPort, router, seed_operator, send_json};

#[sqlx::test(migrations = "../../../../migrations")]
async fn assign_teaching_verifies_teacher_and_homeroom_via_ports(pool: SqlitePool) {
    seed_operator(&pool, "acc-admin", "admin", "tok-admin").await;
    let app = router(
        pool,
        FakeEnrollmentPort::with_active_student("20261001", "hr-1"),
    );

    let (_, c_body) = send_json(
        &app,
        Method::POST,
        "/api/v1/courses",
        "tok-admin",
        Some(&CreateCourseRequest {
            code: "CS101".to_owned(),
            name: "程序设计".to_owned(),
            max_score: Decimal::from(100),
            pass_score: Decimal::from(60),
            idempotency_key: "c-cs".to_owned(),
        }),
    )
    .await;
    let course: CourseSummary = serde_json::from_value(c_body).expect("course");

    let (bad_teacher, _) = send_json(
        &app,
        Method::POST,
        "/api/v1/teachings",
        "tok-admin",
        Some(&AssignTeachingRequest {
            course_id: course.id.clone(),
            homeroom_id: "hr-1".to_owned(),
            term: "2026-秋".to_owned(),
            teacher_account_id: "acc-admin".to_owned(),
            idempotency_key: "t-bad".to_owned(),
        }),
    )
    .await;
    assert_eq!(bad_teacher, StatusCode::FORBIDDEN);

    let (ok_status, t_body) = send_json(
        &app,
        Method::POST,
        "/api/v1/teachings",
        "tok-admin",
        Some(&AssignTeachingRequest {
            course_id: course.id,
            homeroom_id: "hr-1".to_owned(),
            term: "2026-秋".to_owned(),
            teacher_account_id: "acc-teacher".to_owned(),
            idempotency_key: "t-ok".to_owned(),
        }),
    )
    .await;
    assert_eq!(ok_status, StatusCode::CREATED);
    let teaching: TeachingSummary = serde_json::from_value(t_body).expect("teaching");
    assert_eq!(teaching.teacher_name, "林老师");
}
