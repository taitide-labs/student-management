use sqlx::SqlitePool;
use student_management_contracts::events::IntegrationEvent;
use student_management_infra::outbox::{DispatchFuture, OutboxSubscription};

fn on_event<'a>(_pool: &'a SqlitePool, _event: &'a IntegrationEvent) -> DispatchFuture<'a> {
    Box::pin(async { Ok(()) })
}

inventory::submit!(OutboxSubscription {
    name: "grading::evt_void_draft_grades",
    handler: on_event,
});
