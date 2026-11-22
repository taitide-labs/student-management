use nsh_engine::{EngineConfig, Points, Root, Size};
use nsh_testing::Headless;

#[test]
fn walking_skeleton_full_stack_smoke_without_screenshots() -> Result<(), String> {
    let mut app = Headless::start(EngineConfig {
        viewport: Size::new(Points(1200.0), Points(800.0)),
        ..EngineConfig::new(Root::of::<student_management_app::App>())
    })
    .map_err(|err| err.to_string())?;

    app.wait_for("学生管理系统")?;
    app.press("role:button[管理员]")?;
    app.wait_for("20261001")?;
    app.press("张晓明")?;
    app.wait_for("学籍异动")?;
    Ok(())
}
