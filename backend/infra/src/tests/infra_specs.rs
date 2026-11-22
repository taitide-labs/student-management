use student_management_contracts::{clock::Clock, events::IntegrationEvent, ids::IdGenerator};

use crate::{
    FixedClock, IdempotencyOutcome, SequentialIdGenerator, SystemClock, check_idempotency,
    dispatch_pending, enqueue_event, hash_password, hash_payload, open_sqlite_pool,
    save_idempotency, verify_password,
};

#[test]
fn password_hash_verifies_matching_secret_and_rejects_mismatch() {
    let hash = hash_password("secret-123").expect("hash password");
    assert!(verify_password("secret-123", &hash));
    assert!(!verify_password("wrong", &hash));
}

#[test]
fn fixed_and_system_clocks_and_ids_produce_deterministic_and_valid_values() {
    let fixed = FixedClock::new("2026-11-22T12:00:00Z", "2026-11-22");
    assert_eq!(fixed.now_rfc3339(), "2026-11-22T12:00:00Z");
    assert_eq!(fixed.today_iso(), "2026-11-22");
    fixed.set_now("2026-11-23T08:00:00Z", "2026-11-23");
    assert_eq!(fixed.today_iso(), "2026-11-23");

    let sys = SystemClock;
    assert!(sys.now_rfc3339().contains('T'));
    assert_eq!(sys.today_iso().len(), 10);

    let ids = SequentialIdGenerator::new(0);
    assert_eq!(ids.next_id("acc"), "acc-0001");
    assert_eq!(ids.next_id("acc"), "acc-0002");
}

#[tokio::test]
async fn sqlite_migrations_idempotency_and_outbox_roundtrip() {
    let pool = open_sqlite_pool("sqlite::memory:")
        .await
        .expect("open in-memory sqlite");

    let mut tx = pool.begin().await.expect("begin tx");
    let hash = hash_payload(&serde_json::json!({ "name": "高一 (1) 班" })).expect("hash payload");
    let outcome: IdempotencyOutcome<String> =
        check_idempotency(&mut tx, "create_homeroom", "k1", &hash)
            .await
            .expect("fresh check");
    assert_eq!(outcome, IdempotencyOutcome::Fresh);

    save_idempotency(
        &mut tx,
        "create_homeroom",
        "k1",
        &hash,
        &"hr-0001".to_owned(),
        "2026-11-22T12:00:00Z",
    )
    .await
    .expect("save idempotency");

    let replay: IdempotencyOutcome<String> =
        check_idempotency(&mut tx, "create_homeroom", "k1", &hash)
            .await
            .expect("replay check");
    assert_eq!(replay, IdempotencyOutcome::Replay("hr-0001".to_owned()));

    let conflict_hash = hash_payload(&serde_json::json!({ "name": "高一 (2) 班" })).expect("hash");
    let conflict: Result<IdempotencyOutcome<String>, _> =
        check_idempotency(&mut tx, "create_homeroom", "k1", &conflict_hash).await;
    assert!(conflict.is_err());

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
}
