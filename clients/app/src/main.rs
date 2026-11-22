#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

#[cfg(target_os = "linux")]
use nsh_platform_linux as platform;
#[cfg(target_os = "macos")]
use nsh_platform_macos as platform;
#[cfg(target_os = "windows")]
use nsh_platform_windows as platform;

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
fn parse_server_arg() -> Option<String> {
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if let Some(value) = arg.strip_prefix("--server=") {
            return Some(value.to_owned());
        }
        if arg == "--server" {
            return args.next();
        }
    }
    None
}

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
fn main() -> impl std::process::Termination {
    if let Some(server) = parse_server_arg()
        && !server.trim().is_empty()
    {
        student_management_app::install_remote(&server);
    }

    platform::run_app(
        platform::AppSource::Rust(platform::Root::of::<student_management_app::App>()),
        platform::AppOptions {
            title: "学生管理系统".to_owned(),
            #[cfg(any(target_os = "linux", target_os = "windows"))]
            app_id: "dev.taitide.studentmanagement".to_owned(),
            size: (1200, 800),
            ..Default::default()
        },
    )
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn main() {}
