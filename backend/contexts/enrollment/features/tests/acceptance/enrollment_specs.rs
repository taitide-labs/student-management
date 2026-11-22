use sqlx::SqlitePool;

#[sqlx::test(migrations = "../../../../migrations")]
#[ignore = "wip"]
async fn enroll_student_assigns_immutable_student_no_and_enforces_capacity(pool: SqlitePool) {
    drop(pool);
}

#[sqlx::test(migrations = "../../../../migrations")]
#[ignore = "wip"]
async fn transfer_suspend_resume_withdraw_and_reverse_preserve_audit_trail(pool: SqlitePool) {
    drop(pool);
}
