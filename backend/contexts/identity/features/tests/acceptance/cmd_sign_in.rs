use axum::http::{Method, StatusCode};
use rstest::rstest;
use sqlx::SqlitePool;
use student_management_contracts::{AccountSession, ProblemDetails, Role, SignInRequest};
use student_management_infra::open_sqlite_pool;

use super::support::{router, seed_account, send_json};

#[sqlx::test(migrations = "../../../../migrations")]
#[ignore = "wip"]
async fn sign_in_issues_session_for_valid_credentials(pool: SqlitePool) {
    seed_account(&pool, "acc-1", "admin", "admin", "admin-pass", "tok-seed").await;
    let app = router(pool);

    let (status, body) = send_json(
        &app,
        Method::POST,
        "/api/v1/auth/sign-in",
        None,
        Some(&SignInRequest {
            username: "admin".to_owned(),
            password: "admin-pass".to_owned(),
        }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let session: AccountSession = serde_json::from_value(body).expect("account session");
    assert_eq!(session.role, Role::Admin);
    assert!(!session.access_token.is_empty());
}

#[rstest]
#[case("wrong-pass")]
#[case("")]
#[tokio::test]
#[ignore = "wip"]
async fn sign_in_rejects_invalid_credentials_with_problem_details(#[case] bad_password: &str) {
    let pool = open_sqlite_pool("sqlite::memory:").await.expect("sqlite");
    seed_account(&pool, "acc-1", "admin", "admin", "admin-pass", "tok-seed").await;
    let app = router(pool);

    let (status, body) = send_json(
        &app,
        Method::POST,
        "/api/v1/auth/sign-in",
        None,
        Some(&SignInRequest {
            username: "admin".to_owned(),
            password: bad_password.to_owned(),
        }),
    )
    .await;

    assert!(status == StatusCode::UNAUTHORIZED || status == StatusCode::BAD_REQUEST);
    let problem: ProblemDetails = serde_json::from_value(body).expect("problem details");
    assert_eq!(problem.status, status.as_u16());
}
