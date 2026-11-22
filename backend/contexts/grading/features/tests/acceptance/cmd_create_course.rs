use axum::http::{Method, StatusCode};
use rust_decimal::Decimal;
use sqlx::SqlitePool;
use student_management_contracts::{CourseSummary, CreateCourseRequest};

use super::support::{FakeEnrollmentPort, router, seed_operator, send_json};

#[sqlx::test(migrations = "../../../../migrations")]
#[ignore = "wip"]
async fn create_course_succeeds_and_rejects_pass_above_max(pool: SqlitePool) {
    seed_operator(&pool, "acc-admin", "admin", "tok-admin").await;
    let app = router(
        pool,
        FakeEnrollmentPort::with_active_student("20261001", "hr-1"),
    );

    let (bad_status, _) = send_json(
        &app,
        Method::POST,
        "/api/v1/courses",
        "tok-admin",
        Some(&CreateCourseRequest {
            code: "MATH101".to_owned(),
            name: "高等数学".to_owned(),
            max_score: Decimal::from(100),
            pass_score: Decimal::from(105),
            idempotency_key: "c-bad".to_owned(),
        }),
    )
    .await;
    assert_eq!(bad_status, StatusCode::BAD_REQUEST);

    let (ok_status, body) = send_json(
        &app,
        Method::POST,
        "/api/v1/courses",
        "tok-admin",
        Some(&CreateCourseRequest {
            code: "math101".to_owned(),
            name: "高等数学".to_owned(),
            max_score: Decimal::from(100),
            pass_score: Decimal::from(60),
            idempotency_key: "c-ok".to_owned(),
        }),
    )
    .await;
    assert_eq!(ok_status, StatusCode::CREATED);
    let course: CourseSummary = serde_json::from_value(body).expect("course");
    assert_eq!(course.code, "MATH101");
}
