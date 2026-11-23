use std::{
    io::{Write, stderr},
    net::SocketAddr,
};

use student_management_api::{build_router, export_openapi_pretty};
use student_management_runtime::{bootstrap_admin_if_empty, build_app_state};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let mut bootstrap_username = "admin".to_owned();
    let mut bootstrap_password = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--export-openapi" => {
                let target = args
                    .next()
                    .unwrap_or_else(|| "contracts/openapi.json".to_owned());
                let json = export_openapi_pretty()?;
                tokio::fs::write(target, json).await?;
                return Ok(());
            }
            "--bootstrap-username" => {
                if let Some(val) = args.next().filter(|s| !s.trim().is_empty()) {
                    bootstrap_username = val;
                }
            }
            "--bootstrap-password-file" => {
                if let Some(path) = args.next().filter(|s| !s.trim().is_empty()) {
                    let content = tokio::fs::read_to_string(path).await?;
                    let trimmed = content.trim().to_owned();
                    if !trimmed.is_empty() {
                        bootstrap_password = Some(trimmed);
                    }
                }
            }
            _ => {}
        }
    }

    let state = build_app_state("sqlite:student-management.db").await?;
    if let Some((username, initial_secret)) =
        bootstrap_admin_if_empty(&state, &bootstrap_username, bootstrap_password.as_deref()).await?
    {
        let _err = writeln!(
            stderr(),
            "已初始化管理员账号 `{username}`，一次性初始密码：{initial_secret}"
        );
    }
    let router = build_router(state);
    let addr: SocketAddr = "127.0.0.1:8080".parse()?;
    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, router).await?;
    Ok(())
}
