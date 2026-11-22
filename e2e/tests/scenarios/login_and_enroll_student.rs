use nsh_engine::{EngineConfig, Points, Root, Size};
use nsh_testing::Headless;
use student_management_contracts::CreateHomeroomRequest;

use super::server;

fn run_enroll_ui_journey(base_url: &str) -> Result<(), String> {
    student_management_app::install_remote(base_url);
    let mut app = Headless::start(EngineConfig {
        viewport: Size::new(Points(1200.0), Points(800.0)),
        ..EngineConfig::new(Root::of::<student_management_app::App>())
    })
    .map_err(|err| err.to_string())?;

    app.wait_for("学生管理系统")?;
    app.press("role:button[登录]")?;
    app.wait_for("role:button[办理入学…]")?;

    app.press("role:button[办理入学…]")?;
    app.wait_for("新生入学建档")?;
    app.type_text("text:请输入学生姓名", "端到端新生")?;
    app.press("role:combobox[目标行政班]")?;
    app.wait_for("高一 1 班")?;
    app.press("高一 1 班")?;
    app.press("role:button[确认入学]")?;
    app.wait_for("端到端新生")?;
    Ok(())
}

#[tokio::test]
async fn app_login_and_enroll_student_persists_through_real_api_e2e() {
    let srv = server::start_live_server().await;
    let admin = srv.admin_principal();
    let _hr = srv
        .backend
        .create_homeroom(
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

    let students = srv
        .backend
        .list_students(&admin, "端到端新生", None)
        .await
        .expect("query persisted student");
    assert_eq!(students.len(), 1);
    assert_eq!(
        students.first().map(|s| s.full_name.as_str()),
        Some("端到端新生")
    );
}
