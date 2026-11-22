use axum::http::{Method, StatusCode};
use rstest::rstest;
use sqlx::SqlitePool;
use student_management_contracts::{AccountStatus, AccountSummary, ProvisionAccountRequest, Role};
use student_management_infra::open_sqlite_pool;

use super::support::{router, seed_account, send_json};

#[sqlx::test(migrations = "../../../../migrations")]
#[ignore = "wip"]
async fn admin_provisions_teacher_and_lists_via_qry_accounts(pool: SqlitePool) {
    seed_account(&pool, "acc-admin", "admin", "admin", "pw", "tok-admin").await;
    let app = router(pool);

    let (status, body) = send_json(
        &app,
        Method::POST,
        "/api/v1/accounts",
        Some("tok-admin"),
        Some(&ProvisionAccountRequest {
            username: "lin_teacher".to_owned(),
            display_name: "林老师".to_owned(),
            role: Role::Teacher,
            bound_student_no: None,
            idempotency_key: "prov-1".to_owned(),
        }),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    let created: AccountSummary = serde_json::from_value(body).expect("account summary");
    assert_eq!(created.role, Role::Teacher);
    assert_eq!(created.status, AccountStatus::Active);

    let (list_status, list_body) = send_json::<()>(
        &app,
        Method::GET,
        "/api/v1/accounts",
        Some("tok-admin"),
        None,
    )
    .await;
    assert_eq!(list_status, StatusCode::OK);
    let accounts: Vec<AccountSummary> = serde_json::from_value(list_body).expect("account list");
    assert!(accounts.iter().any(|acc| acc.username == "lin_teacher"));
}

#[rstest]
#[case("tok-teacher", Role::Teacher, None, StatusCode::FORBIDDEN)]
#[case("tok-admin", Role::Teacher, Some("20261001"), StatusCode::BAD_REQUEST)]
#[tokio::test]
#[ignore = "wip"]
async fn provision_account_rejects_unauthorized_caller_and_non_student_binding(
    #[case] token: &str,
    #[case] role: Role,
    #[case] bound_student_no: Option<&str>,
    #[case] expected: StatusCode,
) {
    let pool = open_sqlite_pool("sqlite::memory:").await.expect("sqlite");
    seed_account(&pool, "acc-admin", "admin", "admin", "pw", "tok-admin").await;
    seed_account(
        &pool,
        "acc-teacher",
        "teacher",
        "teacher",
        "pw",
        "tok-teacher",
    )
    .await;
    let app = router(pool);

    let (status, _) = send_json(
        &app,
        Method::POST,
        "/api/v1/accounts",
        Some(token),
        Some(&ProvisionAccountRequest {
            username: "bad_user".to_owned(),
            display_name: "测试账号".to_owned(),
            role,
            bound_student_no: bound_student_no.map(str::to_owned),
            idempotency_key: "prov-bad".to_owned(),
        }),
    )
    .await;

    assert_eq!(status, expected);
}
