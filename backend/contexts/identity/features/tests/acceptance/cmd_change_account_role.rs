use axum::http::{Method, StatusCode};
use sqlx::SqlitePool;
use student_management_contracts::{AccountSummary, ChangeAccountRoleRequest, Role};

use super::support::{router, seed_account, send_json};

#[sqlx::test(migrations = "../../../../migrations")]
async fn change_account_role_from_student_clears_bound_student_no(pool: SqlitePool) {
    seed_account(&pool, "acc-admin", "admin", "admin", "pw", "tok-admin").await;
    seed_account(&pool, "acc-stu", "stu01", "student", "pw", "tok-stu").await;
    sqlx::query("UPDATE accounts SET bound_student_no = '20261001' WHERE account_id = 'acc-stu'")
        .execute(&pool)
        .await
        .expect("bind student no");
    let app = router(pool);

    let (status, body) = send_json(
        &app,
        Method::PATCH,
        "/api/v1/accounts/acc-stu/role",
        Some("tok-admin"),
        Some(&ChangeAccountRoleRequest {
            role: Role::Teacher,
            version: 1,
            idempotency_key: "role-1".to_owned(),
        }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let updated: AccountSummary = serde_json::from_value(body).expect("account summary");
    assert_eq!(updated.role, Role::Teacher);
    assert_eq!(updated.bound_student_no, None);
}
