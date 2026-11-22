use std::rc::Rc;

use crate::{
    contract::{
        EnrollStudentRequest, EnrollmentStatus, Gateway, GatewayClient, GatewayError, Role,
        SignInRequest,
    },
    mock::MockGateway,
};

#[tokio::test]
async fn student_sees_only_own_profile_and_cannot_query_others() {
    let gw = MockGateway::seeded();
    let session = gw
        .sign_in(&SignInRequest {
            username: "student".to_owned(),
            password: "demo".to_owned(),
        })
        .await
        .expect("demo student sign-in should succeed");
    let visible = gw
        .list_students(&session, "", None)
        .await
        .expect("list students");
    assert_eq!(visible.len(), 1);
    assert_eq!(
        visible.first().map(|s| s.student_no.as_str()),
        Some("20261001")
    );

    let err = gw
        .student_detail(&session, "20261002")
        .await
        .expect_err("student cannot read another student's profile");
    assert!(matches!(err, GatewayError::Problem(p) if p.status == 403));
}

#[tokio::test]
async fn enroll_rejects_full_homeroom_and_non_admin() {
    let mock = Rc::new(MockGateway::seeded());
    let teacher = mock.switch_role(Role::Teacher).expect("teacher session");
    let client = GatewayClient::new(Rc::clone(&mock) as Rc<dyn Gateway>);
    let err = client
        .enroll_student(
            &teacher,
            &EnrollStudentRequest {
                full_name: "新人".to_owned(),
                homeroom_id: "hr-2026-1".to_owned(),
                idempotency_key: "k-1".to_owned(),
            },
        )
        .await
        .expect_err("teacher cannot enroll");
    assert!(matches!(err, GatewayError::Problem(p) if p.status == 403));

    let admin = mock.switch_role(Role::Admin).expect("admin session");
    let full_err = client
        .enroll_student(
            &admin,
            &EnrollStudentRequest {
                full_name: "满员测试".to_owned(),
                homeroom_id: "hr-2026-2".to_owned(),
                idempotency_key: "k-2".to_owned(),
            },
        )
        .await
        .expect_err("full homeroom must reject");
    assert!(matches!(full_err, GatewayError::Problem(p) if p.code == "homeroom_capacity_exceeded"));

    let enrolled = client
        .enroll_student(
            &admin,
            &EnrollStudentRequest {
                full_name: "陈新宇".to_owned(),
                homeroom_id: "hr-2026-1".to_owned(),
                idempotency_key: "k-3".to_owned(),
            },
        )
        .await
        .expect("admin enroll should succeed");
    assert_eq!(enrolled.status, EnrollmentStatus::Active);
    assert!(enrolled.student_no.starts_with("2026"));
}
