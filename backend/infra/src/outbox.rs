use std::{future::Future, pin::Pin};

use sqlx::{Row, SqliteConnection, SqlitePool};
use student_management_contracts::events::IntegrationEvent;

use crate::error::InfraError;

pub type DispatchFuture<'a> = Pin<Box<dyn Future<Output = Result<(), InfraError>> + Send + 'a>>;
pub type OutboxSubscriber = for<'a> fn(&'a SqlitePool, &'a IntegrationEvent) -> DispatchFuture<'a>;

pub struct OutboxSubscription {
    pub name: &'static str,
    pub handler: OutboxSubscriber,
}

inventory::collect!(OutboxSubscription);

pub async fn enqueue_event(
    conn: &mut SqliteConnection,
    event: &IntegrationEvent,
) -> Result<(), InfraError> {
    let (event_id, event_type, occurred_at) = match event {
        IntegrationEvent::StudentWithdrawn {
            event_id,
            occurred_at,
            ..
        } => (event_id.as_str(), "student_withdrawn", occurred_at.as_str()),
        _ => ("unknown", "unknown", ""),
    };
    let payload_json = serde_json::to_string(event)?;
    sqlx::query(
        "INSERT INTO outbox_events (event_id, event_type, payload_json, occurred_at) VALUES (?, ?, ?, ?)",
    )
    .bind(event_id)
    .bind(event_type)
    .bind(payload_json)
    .bind(occurred_at)
    .execute(conn)
    .await?;
    Ok(())
}

pub async fn dispatch_pending(pool: &SqlitePool, dispatched_at: &str) -> Result<usize, InfraError> {
    let rows = sqlx::query(
        "SELECT event_id, payload_json FROM outbox_events WHERE dispatched_at IS NULL ORDER BY occurred_at ASC",
    )
    .fetch_all(pool)
    .await?;

    let mut count = 0usize;
    for row in rows {
        let event_id: String = row.try_get("event_id")?;
        let payload_json: String = row.try_get("payload_json")?;
        let event: IntegrationEvent = serde_json::from_str(&payload_json)?;

        for sub in inventory::iter::<OutboxSubscription> {
            (sub.handler)(pool, &event).await?;
        }

        sqlx::query("UPDATE outbox_events SET dispatched_at = ? WHERE event_id = ?")
            .bind(dispatched_at)
            .bind(event_id)
            .execute(pool)
            .await?;
        count += 1;
    }

    Ok(count)
}
