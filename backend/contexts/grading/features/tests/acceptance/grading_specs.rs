use sqlx::SqlitePool;

#[sqlx::test(migrations = "../../../../migrations")]
#[ignore = "wip"]
async fn teacher_saves_draft_publishes_and_admin_amends_with_audit(pool: SqlitePool) {
    drop(pool);
}

#[sqlx::test(migrations = "../../../../migrations")]
#[ignore = "wip"]
async fn student_withdrawal_voids_only_draft_grades_and_keeps_published(pool: SqlitePool) {
    drop(pool);
}
