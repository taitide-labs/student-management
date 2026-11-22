use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Method, Request, StatusCode},
};
use rust_decimal::Decimal;
use serde::Serialize;
use serde_json::Value;
use student_management_api::build_router;
use student_management_contracts::{
    AssignTeachingRequest, CourseSummary, CreateCourseRequest, CreateHomeroomRequest,
    EnrollStudentRequest, GradeEntrySummary, GradeStatus, HomeroomSummary, ProvisionAccountRequest,
    Role, SaveDraftGradeRequest, SignInRequest, StudentSummary, TeachingSummary,
    TransitionStudentRequest,
};
use student_management_infra::dispatch_pending;
use student_management_runtime::build_app_state;
use tower::ServiceExt;

async fn call_api<T: Serialize>(
    app: &Router,
    method: Method,
    uri: &str,
    token: Option<&str>,
    body: Option<&T>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json");
    if let Some(tok) = token {
        builder = builder.header("authorization", format!("Bearer {tok}"));
    }
    let raw_req = match body {
        Some(payload) => serde_json::to_vec(payload).expect("serialize request"),
        None => Vec::new(),
    };
    let req = builder.body(Body::from(raw_req)).expect("build request");
    let resp = app.clone().oneshot(req).await.expect("execute api");
    let status = resp.status();
    let raw = to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .expect("read body");
    let parsed = if raw.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&raw).expect("parse response json")
    };
    (status, parsed)
}

async fn seed_e2e_teaching_and_student(
    app: &Router,
    admin_token: &str,
) -> (StudentSummary, TeachingSummary) {
    let (_, teacher_v) = call_api(
        app,
        Method::POST,
        "/api/v1/accounts",
        Some(admin_token),
        Some(&ProvisionAccountRequest {
            username: "lin_e2e".to_owned(),
            display_name: "林老师".to_owned(),
            role: Role::Teacher,
            bound_student_no: None,
            idempotency_key: "e2e-t".to_owned(),
        }),
    )
    .await;
    let teacher_id = teacher_v
        .get("account_id")
        .and_then(Value::as_str)
        .expect("teacher id");

    let (_, hr_v) = call_api(
        app,
        Method::POST,
        "/api/v1/homerooms",
        Some(admin_token),
        Some(&CreateHomeroomRequest {
            name: "高一 (1) 班".to_owned(),
            grade_year: 2026,
            capacity: Some(30),
            idempotency_key: "e2e-hr".to_owned(),
        }),
    )
    .await;
    let hr: HomeroomSummary = serde_json::from_value(hr_v).expect("homeroom");

    let (_, stu_v) = call_api(
        app,
        Method::POST,
        "/api/v1/students",
        Some(admin_token),
        Some(&EnrollStudentRequest {
            full_name: "跨上下文测试生".to_owned(),
            homeroom_id: hr.id.clone(),
            idempotency_key: "e2e-stu".to_owned(),
        }),
    )
    .await;
    let stu: StudentSummary = serde_json::from_value(stu_v).expect("student");

    let (_, c_v) = call_api(
        app,
        Method::POST,
        "/api/v1/courses",
        Some(admin_token),
        Some(&CreateCourseRequest {
            code: "MATH101".to_owned(),
            name: "高等数学".to_owned(),
            max_score: Decimal::from(100),
            pass_score: Decimal::from(60),
            idempotency_key: "e2e-c".to_owned(),
        }),
    )
    .await;
    let course: CourseSummary = serde_json::from_value(c_v).expect("course");

    let (_, teach_v) = call_api(
        app,
        Method::POST,
        "/api/v1/teachings",
        Some(admin_token),
        Some(&AssignTeachingRequest {
            course_id: course.id,
            homeroom_id: hr.id,
            term: "2026-秋".to_owned(),
            teacher_account_id: teacher_id.to_owned(),
            idempotency_key: "e2e-teach".to_owned(),
        }),
    )
    .await;
    let teaching: TeachingSummary = serde_json::from_value(teach_v).expect("teaching");
    (stu, teaching)
}

#[tokio::test]
#[ignore = "wip"]
async fn cross_context_enroll_grade_and_withdraw_voids_only_drafts_e2e() {
    let state = build_app_state("sqlite::memory:")
        .await
        .expect("build runtime");
    let app = build_router(state.clone());

    let (_, admin_sess) = call_api(
        &app,
        Method::POST,
        "/api/v1/auth/sign-in",
        None,
        Some(&SignInRequest {
            username: "admin".to_owned(),
            password: "admin".to_owned(),
        }),
    )
    .await;
    let admin_token = admin_sess
        .get("access_token")
        .and_then(Value::as_str)
        .expect("admin token");
    let (stu, teaching) = seed_e2e_teaching_and_student(&app, admin_token).await;

    let (draft_s, _) = call_api(
        &app,
        Method::PUT,
        "/api/v1/grades/draft",
        Some(admin_token),
        Some(&SaveDraftGradeRequest {
            teaching_id: teaching.id,
            student_no: stu.student_no.clone(),
            score: Decimal::from(80),
            idempotency_key: "e2e-g".to_owned(),
        }),
    )
    .await;
    assert_eq!(draft_s, StatusCode::OK);

    let (w_s, _) = call_api(
        &app,
        Method::POST,
        &format!("/api/v1/students/{}/withdraw", stu.student_no),
        Some(admin_token),
        Some(&TransitionStudentRequest {
            reason: "转校退学".to_owned(),
            version: stu.version,
            idempotency_key: "e2e-w".to_owned(),
        }),
    )
    .await;
    assert_eq!(w_s, StatusCode::OK);

    dispatch_pending(&state.pool, &state.clock.now_rfc3339())
        .await
        .expect("drain outbox");

    let (_, grades_v) = call_api::<()>(
        &app,
        Method::GET,
        &format!("/api/v1/grades?student_no={}", stu.student_no),
        Some(admin_token),
        None,
    )
    .await;
    let grades: Vec<GradeEntrySummary> = serde_json::from_value(grades_v).expect("grades");
    assert_eq!(grades.first().map(|g| g.status), Some(GradeStatus::Voided));
}
