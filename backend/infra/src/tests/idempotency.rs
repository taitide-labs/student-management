use crate::{
    IdempotencyOutcome, check_idempotency, hash_payload, open_sqlite_pool, save_idempotency,
};

#[tokio::test]
async fn idempotency_replays_identical_payload_and_rejects_mismatch() {
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
}
