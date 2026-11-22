use std::sync::Arc;

use sqlx::SqlitePool;
use student_management_contracts::{
    clock::Clock,
    ids::IdGenerator,
    ports::{EnrollmentQueryPort, IdentityLookupPort},
};

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub clock: Arc<dyn Clock>,
    pub ids: Arc<dyn IdGenerator>,
    pub identity_lookup: Option<Arc<dyn IdentityLookupPort>>,
    pub enrollment_query: Option<Arc<dyn EnrollmentQueryPort>>,
}

impl AppState {
    #[must_use]
    pub fn new(pool: SqlitePool, clock: Arc<dyn Clock>, ids: Arc<dyn IdGenerator>) -> Self {
        Self {
            pool,
            clock,
            ids,
            identity_lookup: None,
            enrollment_query: None,
        }
    }

    #[must_use]
    pub fn with_ports(
        mut self,
        identity_lookup: Arc<dyn IdentityLookupPort>,
        enrollment_query: Arc<dyn EnrollmentQueryPort>,
    ) -> Self {
        self.identity_lookup = Some(identity_lookup);
        self.enrollment_query = Some(enrollment_query);
        self
    }
}
