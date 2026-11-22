use std::net::SocketAddr;

use student_management_api::{build_router, export_openapi_pretty};
use student_management_runtime::build_app_state;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    if let Some(arg) = args.next()
        && arg == "--export-openapi"
    {
        let target = args
            .next()
            .unwrap_or_else(|| "contracts/openapi.json".to_owned());
        let json = export_openapi_pretty()?;
        tokio::fs::write(target, json).await?;
        return Ok(());
    }

    let state = build_app_state("sqlite:student-management.db").await?;
    let router = build_router(state);
    let addr: SocketAddr = "127.0.0.1:8080".parse()?;
    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, router).await?;
    Ok(())
}
