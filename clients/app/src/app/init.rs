use std::rc::Rc;

use nsh_core::{
    desktop::{TitleBarOptions, TitleBarStyle, WindowAttributes},
    element::desktop::main_window,
    platform::Appearance,
    prelude::*,
};
use student_management_gateway::{
    contract::GatewayClient,
    mock::MockGateway,
    remote::{NshRemoteGateway, RemoteGatewayConfig},
};

use crate::{gateway, ui::platform};

#[derive(Debug, Clone)]
pub struct RemoteEndpoint(pub String);

pub fn install_remote(base_url: &str) {
    let remote = NshRemoteGateway::new(&RemoteGatewayConfig {
        base_url: base_url.trim().to_owned(),
    });
    gateway::install(GatewayClient::new(Rc::new(remote)));
}

/// 仅用于桌面独立入口解析 `--server`；`nsh_platform_*::AppOptions` 暂未透出 `EngineContext`，
/// 且 `nsh-macros` 的 `#[derive(Props)]` 会注入 `#[allow(clippy::all)]` 与工作区 `forbid` lint 冲突，
/// 故无法由二进制入口透传参数至根组件；E2E 与多引擎宿主仍统一经 `EngineContext` 注入。
#[cfg(not(target_arch = "wasm32"))]
fn server_from_args() -> Option<String> {
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if let Some(value) = arg.strip_prefix("--server=")
            && !value.trim().is_empty()
        {
            return Some(value.to_owned());
        }
        if arg == "--server"
            && let Some(value) = args.next()
            && !value.trim().is_empty()
        {
            return Some(value);
        }
    }
    None
}

pub fn configure_gateway() {
    if gateway::is_configured() {
        return;
    }

    if let Some(endpoint) = engine_context::<RemoteEndpoint>()
        && !endpoint.0.trim().is_empty()
    {
        install_remote(&endpoint.0);
        return;
    }

    if let Some(base_url) = option_env!("SMS_SERVER")
        && !base_url.trim().is_empty()
    {
        install_remote(base_url);
        return;
    }

    #[cfg(not(target_arch = "wasm32"))]
    if let Some(base_url) = server_from_args() {
        install_remote(&base_url);
        return;
    }

    let client = GatewayClient::new(Rc::new(MockGateway::seeded()));
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
