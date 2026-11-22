pub mod common;
pub mod enrollment;
pub mod error;
pub mod grading;
pub mod identity;

pub use common::{IdempotencyKey, NonEmptyText, PositiveVersion, ReasonText};
pub use enrollment::{GradeYear, HomeroomName, StudentNo};
pub use error::PrimitiveError;
pub use grading::{CourseCode, MaxScore, Score, TermCode};
pub use identity::Username;

#[cfg(test)]
mod tests;
