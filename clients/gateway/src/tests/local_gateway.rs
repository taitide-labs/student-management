use crate::{
    contract::{
        CreateHomeroomRequest, EnrollStudentRequest, EnrollmentStatus, Gateway, Role, SignInRequest,
    },
    local::LocalGatewayHandle,
};

#[tokio::test]
async fn local_gateway_executes_sign_in_homeroom_and_enroll_without_http() {
    let gw = LocalGatewayHandle::open("sqlite::memory:")
        .await
        .expect("open local gateway");
    gw.ensure_bootstrap_admin("admin", "教务主任", "admin-pass")
        .await
        .expect("bootstrap admin");

    let session = gw
        .sign_in(&SignInRequest {
            username: "admin".to_owned(),
            password: "admin-pass".to_owned(),
        })
        .await
        .expect("sign in");
    assert_eq!(session.role, Role::Admin);

    let hr = gw
        .create_homeroom(
            &session,
            &CreateHomeroomRequest {
                name: "高一 (1) 班".to_owned(),
                grade_year: 2026,
                capacity: Some(1),
                idempotency_key: "local-hr-1".to_owned(),
            },
        )
        .await
        .expect("create homeroom");

    let stu = gw
        .enroll_student(
            &session,
            &EnrollStudentRequest {
                full_name: "本地直连生".to_owned(),
                homeroom_id: hr.id.clone(),
                idempotency_key: "local-enr-1".to_owned(),
            },
        )
        .await
        .expect("enroll student");
    assert_eq!(stu.status, EnrollmentStatus::Active);
    assert_eq!(stu.student_no, "20260001");

    let full = gw
        .enroll_student(
            &session,
            &EnrollStudentRequest {
                full_name: "超员学生".to_owned(),
                homeroom_id: hr.id,
                idempotency_key: "local-enr-2".to_owned(),
            },
        )
        .await;
    assert!(full.is_err());
}
