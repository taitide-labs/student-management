use std::sync::Arc;

use nsh_engine::{EngineConfig, EngineContext, Points, Root, Size};
use nsh_testing::{Headless, UreqNetwork};
use student_management_app::RemoteEndpoint;
use student_management_contracts::CreateHomeroomRequest;
use student_management_runtime::enrollment;

use super::server;

fn run_enroll_ui_journey(base_url: &str) -> Result<(), String> {
    let mut app = Headless::start(EngineConfig {
        viewport: Size::new(Points(1200.0), Points(800.0)),
        network: Some(Arc::new(UreqNetwork::new())),
        context: EngineContext::default().with(RemoteEndpoint(base_url.to_owned())),
        ..EngineConfig::new(Root::of::<student_management_app::App>())
    })
    .map_err(|err| err.to_string())?;

    app.wait_for("学生管理系统")?;
    app.type_text("text:请输入登录账号", "admin")?;
    app.type_text("text:输入登录密码", "admin")?;
    app.press("role:button[登录]")?;
    app.wait_for("未匹配到学生档案")?;

    app.press("role:button[办理入学…]")?;
    app.wait_for("新生入学建档")?;
    app.type_text("text:请输入学生姓名", "端到端新生")?;
    app.press("role:combobox[目标行政班]")?;
    app.wait_for("高一 1 班")?;
    app.press("高一 1 班")?;
    app.press("role:button[确认入学]")?;
    app.wait_for("text:/^已办理入学/")?;
    Ok(())
}

#[tokio::test]
async fn app_login_and_enroll_student_persists_through_real_api_e2e() {
    let srv = server::start_live_server().await;
    let admin = srv.admin_principal();
    let _hr = enrollment::cmd_create_homeroom::handler::execute(
        &srv.state,
        &admin,
        CreateHomeroomRequest {
            name: "高一 1 班".to_owned(),
            grade_year: 2026,
            capacity: Some(30),
            idempotency_key: "e2e-hr-1".to_owned(),
        },
    )
    .await
    .expect("seed homeroom");

    let base_url = srv.base_url.clone();
    tokio::task::spawn_blocking(move || run_enroll_ui_journey(&base_url))
        .await
        .expect("join headless ui task")
        .expect("run enroll ui journey");

    let students =
        enrollment::qry_students::handler::execute(&srv.state, &admin, "端到端新生", None)
            .await
            .expect("query persisted student");
    assert_eq!(students.len(), 1);
    assert_eq!(
        students.first().map(|s| s.full_name.as_str()),
        Some("端到端新生")
    );
}
