pub mod contract;
#[cfg(feature = "local")]
pub mod local;
#[cfg(feature = "mock")]
pub mod mock;
#[cfg(feature = "remote")]
pub mod remote;

#[cfg(test)]
mod tests;
