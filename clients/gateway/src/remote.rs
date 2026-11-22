mod client;
mod transport;

#[cfg(feature = "nsh")]
pub use client::NshRemoteGateway;
#[cfg(all(feature = "remote", not(target_arch = "wasm32")))]
pub use client::RemoteGateway;
pub use client::{GenericRemoteGateway, RemoteGatewayConfig};
#[cfg(feature = "nsh")]
pub use transport::NshTransport;
pub use transport::RemoteTransport;
#[cfg(all(feature = "remote", not(target_arch = "wasm32")))]
pub use transport::UreqTransport;
