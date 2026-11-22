use std::{cell::RefCell, rc::Rc};

use nsh_core::{
    desktop::{TitleBarOptions, TitleBarStyle, WindowAttributes},
    element::desktop::main_window,
    platform::Appearance,
    prelude::*,
};
#[cfg(not(target_arch = "wasm32"))]
use student_management_gateway::remote::{RemoteGateway, RemoteGatewayConfig};
use student_management_gateway::{contract::GatewayClient, mock::MockGateway};

use crate::{gateway, ui::platform};

#[cfg(not(target_arch = "wasm32"))]
pub fn install_remote(base_url: &str) {
    let remote = RemoteGateway::new(&RemoteGatewayConfig {
        base_url: base_url.trim().to_owned(),
    });
    gateway::install(GatewayClient::new(Rc::new(RefCell::new(remote))));
}

pub fn configure_gateway() {
    if gateway::is_configured() {
        return;
    }

    #[cfg(not(target_arch = "wasm32"))]
    if let Some(base_url) = option_env!("SMS_SERVER")
        && !base_url.trim().is_empty()
    {
        install_remote(base_url);
        return;
    }

    let client = GatewayClient::new(Rc::new(RefCell::new(MockGateway::seeded())));
    gateway::install(client);
}

pub fn configure_window() {
    if !platform::is_compact_desktop() {
        return;
    }
    let mut title_bar = TitleBarOptions::default();
    title_bar.style = TitleBarStyle::Hidden;
    title_bar.height = Some(px(52.0));
    let mut attrs = WindowAttributes::default();
    attrs.title_bar = Some(title_bar);
    main_window().set(attrs);
}

pub fn toggle_color_scheme() {
    let next = if dark() {
        Appearance::Light
    } else {
        Appearance::Dark
    };
    set_color_scheme(next);
}
