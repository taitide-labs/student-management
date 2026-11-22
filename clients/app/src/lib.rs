mod app;
mod features;
mod gateway;
mod pages;
mod ui;

use nsh_core::prelude::*;

pub use crate::app::shell::App;

#[native_module("student-management/native")]
fn native(registry: &mut Registry) {
    registry.register::<App>();
}
