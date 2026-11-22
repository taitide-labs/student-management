pub mod clock;
pub mod db;
pub mod error;
pub mod idempotency;
pub mod ids;
pub mod outbox;
pub mod password;
pub mod registry;
pub mod state;

pub use clock::{FixedClock, SystemClock};
pub use db::{MIGRATOR, open_sqlite_pool};
pub use error::InfraError;
pub use idempotency::{IdempotencyOutcome, check_idempotency, hash_payload, save_idempotency};
pub use ids::{SequentialIdGenerator, UuidGenerator};
pub use outbox::{OutboxSubscriber, OutboxSubscription, dispatch_pending, enqueue_event};
pub use password::{hash_password, verify_password};
pub use registry::{HttpMethod, SliceDescriptor, SliceStatus, registered_slices};
pub use state::AppState;

#[cfg(test)]
mod tests;
