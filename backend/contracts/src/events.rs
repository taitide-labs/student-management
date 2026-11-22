use serde::{Deserialize, Serialize};

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "event_type", rename_all = "snake_case")]
pub enum IntegrationEvent {
    StudentWithdrawn {
        event_id: String,
        student_no: String,
        homeroom_id: String,
        occurred_at: String,
    },
}
