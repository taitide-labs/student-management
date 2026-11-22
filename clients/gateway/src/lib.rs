pub mod contract;
#[cfg(feature = "local")]
pub mod local;
#[cfg(feature = "mock")]
pub mod mock;
#[cfg(any(feature = "remote", feature = "nsh"))]
pub mod remote;

#[cfg(test)]
mod tests;
