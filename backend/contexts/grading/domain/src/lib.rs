pub mod course;
pub mod error;
pub mod grade;

pub use course::Course;
pub use error::GradingDomainError;
pub use grade::{GradeAmendmentDraft, GradeEntry};

#[cfg(test)]
mod tests;
