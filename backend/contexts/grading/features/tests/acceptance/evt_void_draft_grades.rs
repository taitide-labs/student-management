use sqlx::{Row, SqlitePool};
use student_management_contracts::events::IntegrationEvent;
use student_management_infra::{dispatch_pending, enqueue_event};

#[sqlx::test(migrations = "../../../../migrations")]
async fn student_withdrawal_voids_only_draft_grades_and_keeps_published(pool: SqlitePool) {
    student_management_grading_features::link();
    sqlx::query(
        "INSERT INTO courses (id, code, name, max_score, pass_score, version, created_at) VALUES ('c-1', 'MATH101', '高等数学', '100', '60', 1, '2026-11-22T12:00:00Z')",
    )
    .execute(&pool)
    .await
    .expect("seed course");
    sqlx::query(
        "INSERT INTO teaching_assignments (id, course_id, homeroom_id, term, teacher_account_id, teacher_name, version, created_at) VALUES ('t-1', 'c-1', 'hr-1', '2026-秋', 'acc-t1', '林老师', 1, '2026-11-22T12:00:00Z'), ('t-2', 'c-1', 'hr-2', '2026-秋', 'acc-t1', '林老师', 1, '2026-11-22T12:00:00Z')",
    )
    .execute(&pool)
    .await
    .expect("seed teachings");
    sqlx::query(
        "INSERT INTO grade_entries (id, teaching_id, student_no, score, status, version, updated_at) VALUES ('g-draft', 't-1', '20261001', '75', 'draft', 1, '2026-11-22T12:00:00Z'), ('g-pub', 't-2', '20261001', '88', 'published', 2, '2026-11-22T12:00:00Z')",
    )
    .execute(&pool)
    .await
    .expect("seed grades");

    let mut tx = pool.begin().await.expect("begin tx");
    enqueue_event(
        &mut tx,
        &IntegrationEvent::StudentWithdrawn {
            event_id: "evt-withdraw-1".to_owned(),
            student_no: "20261001".to_owned(),
            homeroom_id: "hr-1".to_owned(),
            occurred_at: "2026-11-22T12:05:00Z".to_owned(),
        },
    )
    .await
    .expect("enqueue");
    tx.commit().await.expect("commit");

    dispatch_pending(&pool, "2026-11-22T12:06:00Z")
        .await
        .expect("dispatch");

    let draft_status: String = sqlx::query("SELECT status FROM grade_entries WHERE id = 'g-draft'")
        .fetch_one(&pool)
        .await
        .expect("fetch draft")
        .try_get("status")
        .expect("status");
    let pub_status: String = sqlx::query("SELECT status FROM grade_entries WHERE id = 'g-pub'")
        .fetch_one(&pool)
        .await
        .expect("fetch published")
        .try_get("status")
        .expect("status");

    assert_eq!(draft_status, "voided");
    assert_eq!(pub_status, "published");
}
