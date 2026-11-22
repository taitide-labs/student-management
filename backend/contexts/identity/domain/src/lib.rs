pub mod account;
pub mod error;

pub use account::{Account, AccountProvisionInput};
pub use error::IdentityDomainError;

#[cfg(test)]
mod tests;
