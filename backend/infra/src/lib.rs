pub mod auth;
pub mod clock;
pub mod db;
pub mod error;
pub mod idempotency;
pub mod ids;
pub mod outbox;
pub mod password;
#[cfg(feature = "http")]
pub mod registry;
pub mod state;

pub use auth::{authenticate_bearer, ensure_role, infra_error_to_problem, parse_role};
pub use clock::{FixedClock, SystemClock};
pub use db::{MIGRATOR, open_sqlite_pool};
pub use error::InfraError;
pub use idempotency::{IdempotencyOutcome, check_idempotency, hash_payload, save_idempotency};
pub use ids::{SequentialIdGenerator, UuidGenerator};
pub use outbox::{OutboxSubscriber, OutboxSubscription, dispatch_pending, enqueue_event};
pub use password::{hash_password, verify_password};
#[cfg(feature = "http")]
pub use registry::{
    HttpMethod, SliceDescriptor, SliceStatus, build_context_router, registered_slices,
};
pub use state::AppState;

#[cfg(test)]
mod tests;
