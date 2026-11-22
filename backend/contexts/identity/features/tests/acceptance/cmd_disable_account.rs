use axum::http::{Method, StatusCode};
use sqlx::SqlitePool;
use student_management_contracts::{
    AccountStatus, AccountSummary, DisableAccountRequest, SignInRequest,
};

use super::support::{router, seed_account, send_json};

#[sqlx::test(migrations = "../../../../migrations")]
async fn disable_account_blocks_self_disable_and_revokes_target_sign_in(pool: SqlitePool) {
    seed_account(&pool, "acc-admin", "admin", "admin", "pw", "tok-admin").await;
    seed_account(&pool, "acc-teacher", "lin", "teacher", "pw", "tok-teacher").await;
    let app = router(pool);

    let (self_status, _) = send_json(
        &app,
        Method::POST,
        "/api/v1/accounts/acc-admin/disable",
        Some("tok-admin"),
        Some(&DisableAccountRequest {
            reason: "自我停用测试".to_owned(),
            version: 1,
            idempotency_key: "dis-self".to_owned(),
        }),
    )
    .await;
    assert_eq!(self_status, StatusCode::CONFLICT);

    let (status, body) = send_json(
        &app,
        Method::POST,
        "/api/v1/accounts/acc-teacher/disable",
        Some("tok-admin"),
        Some(&DisableAccountRequest {
            reason: "教师调离".to_owned(),
            version: 1,
            idempotency_key: "dis-1".to_owned(),
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let disabled: AccountSummary = serde_json::from_value(body).expect("account summary");
    assert_eq!(disabled.status, AccountStatus::Disabled);

    let (signin_status, _) = send_json(
        &app,
        Method::POST,
        "/api/v1/auth/sign-in",
        None,
        Some(&SignInRequest {
            username: "lin".to_owned(),
            password: "pw".to_owned(),
        }),
    )
    .await;
    assert_eq!(signin_status, StatusCode::FORBIDDEN);
}
