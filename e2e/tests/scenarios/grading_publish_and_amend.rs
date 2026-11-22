use std::sync::Arc;

use nsh_engine::{EngineConfig, EngineContext, Points, Root, Size};
use nsh_testing::{Headless, UreqNetwork};
use rust_decimal::Decimal;
use student_management_app::RemoteEndpoint;
use student_management_contracts::{
    AssignTeachingRequest, CreateCourseRequest, CreateHomeroomRequest, EnrollStudentRequest,
    GradeStatus, ProvisionAccountRequest, Role,
};
use student_management_runtime::{enrollment, grading, identity};

use super::server;

async fn seed_grading_prerequisites(srv: &server::LiveApiServer) -> String {
    let admin = srv.admin_principal();
    let hr = enrollment::cmd_create_homeroom::handler::execute(
        &srv.state,
        &admin,
        CreateHomeroomRequest {
            name: "高一 1 班".to_owned(),
            grade_year: 2026,
            capacity: Some(30),
            idempotency_key: "e2e-grd-hr".to_owned(),
        },
    )
    .await
    .expect("seed homeroom");
    let stu = enrollment::cmd_enroll_student::handler::execute(
        &srv.state,
        &admin,
        EnrollStudentRequest {
            full_name: "张晓明".to_owned(),
            homeroom_id: hr.id.clone(),
            idempotency_key: "e2e-grd-stu".to_owned(),
        },
    )
    .await
    .expect("seed student");

    let teacher = identity::cmd_provision_account::handler::execute(
        &srv.state,
        &admin,
        ProvisionAccountRequest {
            username: "e2e_teacher".to_owned(),
            display_name: "林老师".to_owned(),
            role: Role::Teacher,
            bound_student_no: None,
            idempotency_key: "e2e-grd-t".to_owned(),
        },
    )
    .await
    .expect("seed teacher");
    let course = grading::cmd_create_course::handler::execute(
        &srv.state,
        &admin,
        CreateCourseRequest {
            code: "MATH101".to_owned(),
            name: "高等数学".to_owned(),
            max_score: Decimal::from(100),
            pass_score: Decimal::from(60),
            idempotency_key: "e2e-grd-c".to_owned(),
        },
    )
    .await
    .expect("seed course");
    let _teaching = grading::cmd_assign_teaching::handler::execute(
        &srv.state,
        &admin,
        AssignTeachingRequest {
            course_id: course.id,
            homeroom_id: hr.id,
            term: "2026-秋".to_owned(),
            teacher_account_id: teacher.account_id,
            idempotency_key: "e2e-grd-tch".to_owned(),
        },
    )
    .await
    .expect("seed teaching");
    stu.student_no
}

fn run_grading_ui_journey(base_url: &str, student_no: &str) -> Result<(), String> {
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
    app.wait_for("role:button[成绩]")?;
    app.press("role:button[成绩]")?;
    app.wait_for("role:button[录入成绩…]")?;

    app.press("role:button[录入成绩…]")?;
    app.wait_for("保存成绩草稿")?;
    app.type_text("text:请输入学生学号", student_no)?;
    app.type_text("text:请输入分值", "91.5")?;
    app.press("role:button[保存草稿]")?;
    app.wait_for("91.5")?;

    app.press("role:button[锁分发布]")?;
    app.wait_for("已锁定")?;
    app.press("role:button[修订…]")?;
    app.wait_for("合规修订成绩")?;
    app.type_text("text:请输入修订后分值", "95")?;
    app.type_text("text:请输入修订原因", "试卷复核加分")?;
    app.press("role:button[确认修订]")?;
    app.wait_for("95")?;
    Ok(())
}

#[tokio::test]
async fn app_grading_draft_publish_and_amend_persists_through_real_api_e2e() {
    let srv = server::start_live_server().await;
    let student_no = seed_grading_prerequisites(&srv).await;
    let admin = srv.admin_principal();

    let base_url = srv.base_url.clone();
    let stu_no = student_no.clone();
    tokio::task::spawn_blocking(move || run_grading_ui_journey(&base_url, &stu_no))
        .await
        .expect("join headless grading ui task")
        .expect("run grading ui journey");

    let grades = grading::qry_grades::handler::execute(&srv.state, &admin, Some(&student_no))
        .await
        .expect("query persisted grades");
    let grade = grades.first().expect("grade entry persisted");
    assert_eq!(grade.score, Decimal::from(95));
    assert_eq!(grade.status, GradeStatus::Published);

    let audits =
        grading::qry_grade_amendments::handler::execute(&srv.state, &admin, Some(&grade.id))
            .await
            .expect("query persisted amendment audits");
    assert_eq!(audits.len(), 1);
    assert_eq!(
        audits.first().map(|a| a.reason.as_str()),
        Some("试卷复核加分")
    );
}
