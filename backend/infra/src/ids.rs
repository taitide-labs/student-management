use std::sync::atomic::{AtomicU64, Ordering};

use student_management_contracts::ids::IdGenerator;
use uuid::Uuid;

#[derive(Debug, Default, Clone, Copy)]
pub struct UuidGenerator;

impl IdGenerator for UuidGenerator {
    fn next_id(&self, prefix: &str) -> String {
        format!("{prefix}-{}", Uuid::new_v4())
    }
}

#[derive(Debug, Default)]
pub struct SequentialIdGenerator {
    counter: AtomicU64,
}

impl SequentialIdGenerator {
    #[must_use]
    pub const fn new(start: u64) -> Self {
        Self {
            counter: AtomicU64::new(start),
        }
    }
}

impl IdGenerator for SequentialIdGenerator {
    fn next_id(&self, prefix: &str) -> String {
        let seq = self.counter.fetch_add(1, Ordering::Relaxed) + 1;
        format!("{prefix}-{seq:04}")
    }
}
