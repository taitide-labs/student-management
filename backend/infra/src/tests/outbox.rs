use student_management_contracts::events::IntegrationEvent;

use crate::{dispatch_pending, enqueue_event, open_sqlite_pool};

#[tokio::test]
async fn outbox_enqueues_and_marks_events_dispatched() {
    let pool = open_sqlite_pool("sqlite::memory:")
        .await
        .expect("open in-memory sqlite");

    let mut tx = pool.begin().await.expect("begin tx");
    enqueue_event(
        &mut tx,
        &IntegrationEvent::StudentWithdrawn {
            event_id: "evt-1".to_owned(),
            student_no: "20261001".to_owned(),
            homeroom_id: "hr-0001".to_owned(),
            occurred_at: "2026-11-22T12:00:00Z".to_owned(),
        },
    )
    .await
    .expect("enqueue outbox");
    tx.commit().await.expect("commit");

    let dispatched = dispatch_pending(&pool, "2026-11-22T12:01:00Z")
        .await
        .expect("dispatch outbox");
    assert_eq!(dispatched, 1);

    let second = dispatch_pending(&pool, "2026-11-22T12:02:00Z")
        .await
        .expect("idempotent drain");
    assert_eq!(second, 0);
}
