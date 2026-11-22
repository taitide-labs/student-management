use sqlx::SqlitePool;

#[sqlx::test(migrations = "../../../../migrations")]
#[ignore = "wip"]
async fn admin_provisions_binds_changes_role_and_disables_account(pool: SqlitePool) {
    drop(pool);
}

#[sqlx::test(migrations = "../../../../migrations")]
#[ignore = "wip"]
async fn disabled_account_revokes_existing_session_and_blocks_sign_in(pool: SqlitePool) {
    drop(pool);
}
