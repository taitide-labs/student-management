use rust_decimal::Decimal;

use crate::{
    contract::{
        AccountStatus, AmendGradeRequest, AssignTeachingRequest, ChangeAccountRoleRequest,
        CreateCourseRequest, CreateHomeroomRequest, DisableAccountRequest, EnrollStudentRequest,
        EnrollmentStatus, Gateway, GatewayError, GradeStatus, ProvisionAccountRequest,
        PublishGradesRequest, ReverseEnrollmentRequest, Role, SaveDraftGradeRequest, SignInRequest,
        TransferHomeroomRequest, TransitionStudentRequest, UpdateHomeroomCapacityRequest,
    },
    mock::MockGateway,
};

#[tokio::test]
async fn enroll_idempotency_replays_same_payload_and_rejects_mismatch() {
    let gw = MockGateway::seeded();
    let admin = gw.switch_role(Role::Admin).expect("admin session");

    let first = gw
        .enroll_student(
            &admin,
            &EnrollStudentRequest {
                full_name: "林可欣".to_owned(),
                homeroom_id: "hr-2026-1".to_owned(),
                idempotency_key: "idem-enroll-1".to_owned(),
            },
        )
        .await
        .expect("first enroll");
    let replay = gw
        .enroll_student(
            &admin,
            &EnrollStudentRequest {
                full_name: "林可欣".to_owned(),
                homeroom_id: "hr-2026-1".to_owned(),
                idempotency_key: "idem-enroll-1".to_owned(),
            },
        )
        .await
        .expect("idempotent replay must return identical student");
    assert_eq!(first.student_no, replay.student_no);

    let mismatch = gw
        .enroll_student(
            &admin,
            &EnrollStudentRequest {
                full_name: "不同姓名".to_owned(),
                homeroom_id: "hr-2026-1".to_owned(),
                idempotency_key: "idem-enroll-1".to_owned(),
            },
        )
        .await
        .expect_err("different payload with same idempotency key must reject 409");
    assert!(matches!(mismatch, GatewayError::Problem(p) if p.code == "idempotency_key_conflict"));
}

#[tokio::test]
async fn status_lifecycle_and_optimistic_version_lock() {
    let gw = MockGateway::seeded();
    let admin = gw.switch_role(Role::Admin).expect("admin session");

    let stale = gw
        .transfer_homeroom(
            &admin,
            "20261001",
            &TransferHomeroomRequest {
                target_homeroom_id: "hr-2025-1".to_owned(),
                reason: "调整班级".to_owned(),
                version: 99,
                idempotency_key: "idem-tr-1".to_owned(),
            },
        )
        .await
        .expect_err("stale version must reject 409");
    assert!(matches!(stale, GatewayError::Problem(p) if p.code == "version_conflict"));

    let withdrawn = gw
        .withdraw_student(
            &admin,
            "20261001",
            &TransitionStudentRequest {
                reason: "转出外校".to_owned(),
                version: 1,
                idempotency_key: "idem-wd-1".to_owned(),
            },
        )
        .await
        .expect("withdraw should succeed");
    assert_eq!(withdrawn.status, EnrollmentStatus::Withdrawn);

    let invalid_resume = gw
        .resume_student(
            &admin,
            "20261001",
            &TransitionStudentRequest {
                reason: "尝试恢复".to_owned(),
                version: withdrawn.version,
                idempotency_key: "idem-rs-1".to_owned(),
            },
        )
        .await
        .expect_err("terminal status cannot resume directly");
    assert!(
        matches!(invalid_resume, GatewayError::Problem(p) if p.code == "invalid_status_transition")
    );

    let reversed = gw
        .reverse_enrollment(
            &admin,
            "20261001",
            &ReverseEnrollmentRequest {
                target_status: EnrollmentStatus::Active,
                reason: "教务误录退学冲正".to_owned(),
                version: withdrawn.version,
                idempotency_key: "idem-rev-1".to_owned(),
            },
        )
        .await
        .expect("reverse enrollment should succeed");
    assert_eq!(reversed.status, EnrollmentStatus::Active);
}

#[tokio::test]
async fn student_reads_only_published_grades() {
    let gw = MockGateway::seeded();
    let student = gw.switch_role(Role::Student).expect("student session");
    let grades = gw
        .list_grades(&student, Some("20261001"))
        .await
        .expect("student grades");
    assert_eq!(grades.len(), 1);
    assert_eq!(
        grades.first().map(|g| g.course_code.as_str()),
        Some("MATH101")
    );
}

#[tokio::test]
async fn homeroom_capacity_cannot_drop_below_active_count() {
    let gw = MockGateway::seeded();
    let admin = gw.switch_role(Role::Admin).expect("admin session");

    let created = gw
        .create_homeroom(
            &admin,
            &CreateHomeroomRequest {
                name: "高一 3 班".to_owned(),
                grade_year: 2026,
                capacity: Some(30),
                idempotency_key: "idem-hr-create-1".to_owned(),
            },
        )
        .await
        .expect("create homeroom");
    assert_eq!(created.name, "高一 3 班");

    let err = gw
        .update_homeroom_capacity(
            &admin,
            "hr-2026-1",
            &UpdateHomeroomCapacityRequest {
                capacity: Some(1),
                version: 1,
                idempotency_key: "idem-hr-cap-1".to_owned(),
            },
        )
        .await
        .expect_err("cannot shrink capacity below active student count");
    assert!(
        matches!(err, GatewayError::Problem(p) if p.code == "homeroom_capacity_below_active_count")
    );
}

#[tokio::test]
async fn grading_draft_publish_and_admin_only_amendment_audit() {
    let gw = MockGateway::seeded();
    let admin = gw.switch_role(Role::Admin).expect("admin session");
    let teacher = gw.switch_role(Role::Teacher).expect("teacher session");

    let course = gw
        .create_course(
            &admin,
            &CreateCourseRequest {
                code: "PHYS101".to_owned(),
                name: "大学物理".to_owned(),
                max_score: Decimal::new(100, 0),
                pass_score: Decimal::new(60, 0),
                idempotency_key: "idem-crs-1".to_owned(),
            },
        )
        .await
        .expect("create course");
    let teaching = gw
        .assign_teaching(
            &admin,
            &AssignTeachingRequest {
                course_id: course.id,
                homeroom_id: "hr-2026-1".to_owned(),
                term: "2026 秋季".to_owned(),
                teacher_account_id: teacher.account_id.clone(),
                idempotency_key: "idem-tch-1".to_owned(),
            },
        )
        .await
        .expect("assign teaching");

    let draft = gw
        .save_draft_grade(
            &teacher,
            &SaveDraftGradeRequest {
                teaching_id: teaching.id.clone(),
                student_no: "20261001".to_owned(),
                score: Decimal::new(865, 1),
                idempotency_key: "idem-grd-save-1".to_owned(),
            },
        )
        .await
        .expect("teacher saves draft grade");
    assert_eq!(draft.status, GradeStatus::Draft);

    let published = gw
        .publish_grades(
            &teacher,
            &PublishGradesRequest {
                teaching_id: teaching.id,
                idempotency_key: "idem-grd-pub-1".to_owned(),
            },
        )
        .await
        .expect("publish grades");
    let entry = published.first().expect("published grade entry");
    assert_eq!(entry.status, GradeStatus::Published);

    let teacher_amend = gw
        .amend_grade(
            &teacher,
            &entry.id,
            &AmendGradeRequest {
                new_score: Decimal::new(90, 0),
                reason: "复核加分".to_owned(),
                version: entry.version,
                idempotency_key: "idem-grd-amd-t".to_owned(),
            },
        )
        .await
        .expect_err("teacher must not amend published grades");
    assert!(matches!(teacher_amend, GatewayError::Problem(p) if p.status == 403));

    let amended = gw
        .amend_grade(
            &admin,
            &entry.id,
            &AmendGradeRequest {
                new_score: Decimal::new(90, 0),
                reason: "试卷复核更正".to_owned(),
                version: entry.version,
                idempotency_key: "idem-grd-amd-a".to_owned(),
            },
        )
        .await
        .expect("admin amends published grade");
    assert_eq!(amended.score, Decimal::new(90, 0));

    let audits = gw
        .list_grade_amendments(&admin, Some(&entry.id))
        .await
        .expect("list grade amendments");
    assert_eq!(audits.len(), 1);
    assert_eq!(
        audits.first().map(|a| a.reason.as_str()),
        Some("试卷复核更正")
    );
}

#[tokio::test]
async fn account_provisioning_role_change_and_disable_blocks_sign_in() {
    let gw = MockGateway::seeded();
    let admin = gw.switch_role(Role::Admin).expect("admin session");

    let created = gw
        .provision_account(
            &admin,
            &ProvisionAccountRequest {
                username: "teacher2".to_owned(),
                display_name: "王老师".to_owned(),
                role: Role::Teacher,
                bound_student_no: None,
                idempotency_key: "idem-acc-1".to_owned(),
            },
        )
        .await
        .expect("provision account");
    assert_eq!(created.status, AccountStatus::Active);

    let promoted = gw
        .change_account_role(
            &admin,
            &created.account_id,
            &ChangeAccountRoleRequest {
                role: Role::Admin,
                version: created.version,
                idempotency_key: "idem-acc-role-1".to_owned(),
            },
        )
        .await
        .expect("change account role");
    assert_eq!(promoted.role, Role::Admin);

    let disabled = gw
        .disable_account(
            &admin,
            &promoted.account_id,
            &DisableAccountRequest {
                reason: "离职停用".to_owned(),
                version: promoted.version,
                idempotency_key: "idem-acc-dis-1".to_owned(),
            },
        )
        .await
        .expect("disable account");
    assert_eq!(disabled.status, AccountStatus::Disabled);

    let login_err = gw
        .sign_in(&SignInRequest {
            username: "teacher2".to_owned(),
            password: "demo".to_owned(),
        })
        .await
        .expect_err("disabled account cannot sign in");
    assert!(matches!(login_err, GatewayError::Problem(p) if p.code == "account_disabled"));
}
