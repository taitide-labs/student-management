pub mod error;
pub mod homeroom;
pub mod student;

pub use error::EnrollmentDomainError;
pub use homeroom::Homeroom;
pub use student::{EnrollmentChangeDraft, EnrollmentTransition, Student};

#[cfg(test)]
mod tests;
