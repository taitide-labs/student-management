pub mod bootstrap;

pub use bootstrap::{
    bootstrap_admin_if_empty, build_app_state, dispatch_outbox, ensure_bootstrap_admin,
    link_all_contexts, open_app_state, open_local_admin_session, resolve_session,
};
pub use student_management_enrollment_features as enrollment;
pub use student_management_grading_features as grading;
pub use student_management_identity_features as identity;
pub use student_management_infra::AppState;
