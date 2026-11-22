mod app;
mod features;
mod gateway;
mod pages;
mod ui;

use nsh_core::prelude::*;

/// Configures the desktop client to communicate with a remote API server.
#[cfg(not(target_arch = "wasm32"))]
pub use crate::app::init::install_remote;
pub use crate::app::shell::App;

#[native_module("student-management/native")]
fn native(registry: &mut Registry) {
    registry.register::<App>();
}
