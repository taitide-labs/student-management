use axum::http::{Method, StatusCode};
use sqlx::SqlitePool;
use student_management_contracts::{AccountSummary, BindPersonProfileRequest};

use super::support::{router, seed_account, send_json};

#[sqlx::test(migrations = "../../../../migrations")]
#[ignore = "wip"]
async fn bind_person_profile_updates_student_binding_and_rejects_stale_version(pool: SqlitePool) {
    seed_account(&pool, "acc-admin", "admin", "admin", "pw", "tok-admin").await;
    seed_account(&pool, "acc-stu", "stu01", "student", "pw", "tok-stu").await;
    let app = router(pool);

    let (status, body) = send_json(
        &app,
        Method::PUT,
        "/api/v1/accounts/acc-stu/person-profile",
        Some("tok-admin"),
        Some(&BindPersonProfileRequest {
            student_no: Some("20261001".to_owned()),
            version: 1,
            idempotency_key: "bind-1".to_owned(),
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let updated: AccountSummary = serde_json::from_value(body).expect("account summary");
    assert_eq!(updated.bound_student_no.as_deref(), Some("20261001"));
    assert_eq!(updated.version, 2);

    let (conflict_status, _) = send_json(
        &app,
        Method::PUT,
        "/api/v1/accounts/acc-stu/person-profile",
        Some("tok-admin"),
        Some(&BindPersonProfileRequest {
            student_no: Some("20261002".to_owned()),
            version: 1,
            idempotency_key: "bind-2".to_owned(),
        }),
    )
    .await;
    assert_eq!(conflict_status, StatusCode::CONFLICT);
}
