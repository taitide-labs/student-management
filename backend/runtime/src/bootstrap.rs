use std::sync::Arc;

use student_management_enrollment_features::port::SqliteEnrollmentQueryPort;
use student_management_identity_features::port::SqliteIdentityLookupPort;
use student_management_infra::{
    AppState, InfraError, SystemClock, UuidGenerator, open_sqlite_pool,
};

pub fn link_all_contexts() {
    student_management_identity_features::link();
    student_management_enrollment_features::link();
    student_management_grading_features::link();
}

pub async fn build_app_state(database_url: &str) -> Result<AppState, InfraError> {
    link_all_contexts();
    let pool = open_sqlite_pool(database_url).await?;
    let identity_port = Arc::new(SqliteIdentityLookupPort::new(pool.clone()));
    let enrollment_port = Arc::new(SqliteEnrollmentQueryPort::new(pool.clone()));
    Ok(
        AppState::new(pool, Arc::new(SystemClock), Arc::new(UuidGenerator))
            .with_ports(identity_port, enrollment_port),
    )
}
