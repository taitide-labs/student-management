use nsh_engine::{EngineConfig, Points, Root, Size};
use nsh_testing::Headless;
use rust_decimal::Decimal;
use student_management_contracts::{
    AssignTeachingRequest, CreateCourseRequest, CreateHomeroomRequest, EnrollStudentRequest,
    GradeStatus, ProvisionAccountRequest, Role,
};

use super::server;

async fn seed_grading_prerequisites(srv: &server::LiveApiServer) -> String {
    let admin = srv.admin_principal();
    let hr = srv
        .backend
        .create_homeroom(
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
    let stu = srv
        .backend
        .enroll_student(
            &admin,
            EnrollStudentRequest {
                full_name: "张晓明".to_owned(),
                homeroom_id: hr.id.clone(),
                idempotency_key: "e2e-grd-stu".to_owned(),
            },
        )
        .await
        .expect("seed student");

    let teacher = srv
        .backend
        .provision_account(
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
    let course = srv
        .backend
        .create_course(
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
    let _teaching = srv
        .backend
        .assign_teaching(
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
    student_management_app::install_remote(base_url);
    let mut app = Headless::start(EngineConfig {
        viewport: Size::new(Points(1200.0), Points(800.0)),
        ..EngineConfig::new(Root::of::<student_management_app::App>())
    })
    .map_err(|err| err.to_string())?;

    app.wait_for("学生管理系统")?;
    app.press("role:button[登录]")?;
    app.wait_for("role:button[成绩]")?;
    app.press("role:button[成绩]")?;
    app.wait_for("role:button[录入成绩…]")?;

    app.press("role:button[录入成绩…]")?;
    app.wait_for("保存成绩草稿")?;
    app.clear("text:20261001")?;
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

    let grades = srv
        .backend
        .list_grades(&admin, Some(&student_no))
        .await
        .expect("query persisted grades");
    let grade = grades.first().expect("grade entry persisted");
    assert_eq!(grade.score, Decimal::from(95));
    assert_eq!(grade.status, GradeStatus::Published);

    let audits = srv
        .backend
        .list_grade_amendments(&admin, Some(&grade.id))
        .await
        .expect("query persisted amendment audits");
    assert_eq!(audits.len(), 1);
    assert_eq!(
        audits.first().map(|a| a.reason.as_str()),
        Some("试卷复核加分")
    );
}
