use student_management_contracts::Role;

use crate::{authenticate_bearer, ensure_role, hash_password, open_sqlite_pool};

#[tokio::test]
async fn bearer_authentication_resolves_active_session_and_blocks_disabled_or_revoked() {
    let pool = open_sqlite_pool("sqlite::memory:").await.expect("sqlite");
    let hash = hash_password("pw").expect("hash");

    sqlx::query(
        "INSERT INTO accounts (account_id, username, display_name, password_hash, role, status, version, created_at, updated_at) VALUES \
         ('acc-1', 'admin', '教务员', ?, 'admin', 'active', 1, '2026-11-22T12:00:00Z', '2026-11-22T12:00:00Z'), \
         ('acc-2', 'dis', '已停用', ?, 'teacher', 'disabled', 1, '2026-11-22T12:00:00Z', '2026-11-22T12:00:00Z')",
    )
    .bind(&hash)
    .bind(&hash)
    .execute(&pool)
    .await
    .expect("seed accounts");

    sqlx::query(
        "INSERT INTO sessions (token, account_id, created_at, revoked_at) VALUES \
         ('tok-ok', 'acc-1', '2026-11-22T12:00:00Z', NULL), \
         ('tok-revoked', 'acc-1', '2026-11-22T12:00:00Z', '2026-11-22T12:05:00Z'), \
         ('tok-disabled', 'acc-2', '2026-11-22T12:00:00Z', NULL)",
    )
    .execute(&pool)
    .await
    .expect("seed sessions");

    let principal = authenticate_bearer(&pool, Some("Bearer tok-ok"))
        .await
        .expect("valid session");
    assert_eq!(principal.account_id, "acc-1");
    assert_eq!(principal.role, Role::Admin);
    assert!(ensure_role(&principal, &[Role::Admin]).is_ok());
    assert!(ensure_role(&principal, &[Role::Teacher]).is_err());

    assert!(authenticate_bearer(&pool, None).await.is_err());
    assert!(
        authenticate_bearer(&pool, Some("Bearer tok-revoked"))
            .await
            .is_err()
    );
    assert!(
        authenticate_bearer(&pool, Some("Bearer tok-disabled"))
            .await
            .is_err()
    );
}
