use std::{cell::RefCell, rc::Rc};

use nsh_core::{
    desktop::{TitleBarOptions, TitleBarStyle, WindowAttributes},
    element::desktop::main_window,
    platform::Appearance,
    prelude::*,
};
use student_management_gateway::{contract::GatewayClient, mock::MockGateway};

use crate::{gateway, ui::platform};

pub fn configure_gateway() {
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
