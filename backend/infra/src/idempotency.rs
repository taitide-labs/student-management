use serde::{Serialize, de::DeserializeOwned};
use sqlx::{Row, SqliteConnection};

use crate::error::InfraError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdempotencyOutcome<T> {
    Fresh,
    Replay(T),
}

pub fn hash_payload<T: Serialize>(payload: &T) -> Result<String, InfraError> {
    let serialized = serde_json::to_vec(payload)?;
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in serialized {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    Ok(format!("{hash:016x}"))
}

pub async fn check_idempotency<T: DeserializeOwned>(
    conn: &mut SqliteConnection,
    scope: &str,
    idempotency_key: &str,
    request_hash: &str,
) -> Result<IdempotencyOutcome<T>, InfraError> {
    let row = sqlx::query(
        "SELECT request_hash, response_json FROM idempotency_records WHERE scope = ? AND idempotency_key = ?",
    )
    .bind(scope)
    .bind(idempotency_key)
    .fetch_optional(conn)
    .await?;

    let Some(row) = row else {
        return Ok(IdempotencyOutcome::Fresh);
    };

    let saved_hash: String = row.try_get("request_hash")?;
    if saved_hash != request_hash {
        return Err(InfraError::IdempotencyConflict);
    }

    let response_json: String = row.try_get("response_json")?;
    let cached: T = serde_json::from_str(&response_json)?;
    Ok(IdempotencyOutcome::Replay(cached))
}

pub async fn save_idempotency<T: Serialize>(
    conn: &mut SqliteConnection,
    scope: &str,
    idempotency_key: &str,
    request_hash: &str,
    response: &T,
    created_at: &str,
) -> Result<(), InfraError> {
    let response_json = serde_json::to_string(response)?;
    sqlx::query(
        "INSERT INTO idempotency_records (scope, idempotency_key, request_hash, response_json, created_at) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(scope)
    .bind(idempotency_key)
    .bind(request_hash)
    .bind(response_json)
    .bind(created_at)
    .execute(conn)
    .await?;
    Ok(())
}
