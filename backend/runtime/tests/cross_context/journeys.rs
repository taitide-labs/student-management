use rust_decimal::Decimal;
use student_management_contracts::{
    AssignTeachingRequest, CreateCourseRequest, CreateHomeroomRequest, DisableAccountRequest,
    EnrollStudentRequest, GradeStatus, OperatorPrincipal, ProvisionAccountRequest,
    PublishGradesRequest, Role, SaveDraftGradeRequest, SignInRequest, TransitionStudentRequest,
};
use student_management_runtime::{LocalBackend, build_app_state};

async fn open_seeded_backend() -> (LocalBackend, OperatorPrincipal) {
    let state = build_app_state("sqlite::memory:")
        .await
        .expect("build runtime state");
    let backend = LocalBackend::new(state);
    backend
        .ensure_bootstrap_admin("admin", "系统管理员", "admin")
        .await
        .expect("bootstrap admin");
    let session = backend
        .sign_in(SignInRequest {
            username: "admin".to_owned(),
            password: "admin".to_owned(),
        })
        .await
        .expect("admin sign-in");
    let admin = OperatorPrincipal {
        account_id: session.account_id,
        display_name: session.display_name,
        role: session.role,
        bound_student_no: session.bound_student_no,
    };
    (backend, admin)
}

async fn seed_teachings(
    backend: &LocalBackend,
    admin: &OperatorPrincipal,
) -> (String, u64, String, String) {
    let hr = backend
        .create_homeroom(
            admin,
            CreateHomeroomRequest {
                name: "高一 1 班".to_owned(),
                grade_year: 2026,
                capacity: Some(30),
                idempotency_key: "rt-hr-1".to_owned(),
            },
        )
        .await
        .expect("create homeroom");
    let stu = backend
        .enroll_student(
            admin,
            EnrollStudentRequest {
                full_name: "跨上下文退学生".to_owned(),
                homeroom_id: hr.id.clone(),
                idempotency_key: "rt-stu-1".to_owned(),
            },
        )
        .await
        .expect("enroll student");
    let teacher = backend
        .provision_account(
            admin,
            ProvisionAccountRequest {
                username: "rt_teacher".to_owned(),
                display_name: "林老师".to_owned(),
                role: Role::Teacher,
                bound_student_no: None,
                idempotency_key: "rt-t-1".to_owned(),
            },
        )
        .await
        .expect("provision teacher");
    let c1 = backend
        .create_course(
            admin,
            CreateCourseRequest {
                code: "MATH101".to_owned(),
                name: "高等数学".to_owned(),
                max_score: Decimal::from(100),
                pass_score: Decimal::from(60),
                idempotency_key: "rt-c1".to_owned(),
            },
        )
        .await
        .expect("create math");
    let c2 = backend
        .create_course(
            admin,
            CreateCourseRequest {
                code: "ENG101".to_owned(),
                name: "大学英语".to_owned(),
                max_score: Decimal::from(100),
                pass_score: Decimal::from(60),
                idempotency_key: "rt-c2".to_owned(),
            },
        )
        .await
        .expect("create eng");
    let t_draft = backend
        .assign_teaching(
            admin,
            AssignTeachingRequest {
                course_id: c1.id,
                homeroom_id: hr.id.clone(),
                term: "2026-秋".to_owned(),
                teacher_account_id: teacher.account_id.clone(),
                idempotency_key: "rt-td-1".to_owned(),
            },
        )
        .await
        .expect("assign math");
    let t_pub = backend
        .assign_teaching(
            admin,
            AssignTeachingRequest {
                course_id: c2.id,
                homeroom_id: hr.id,
                term: "2026-秋".to_owned(),
                teacher_account_id: teacher.account_id,
                idempotency_key: "rt-tp-1".to_owned(),
            },
        )
        .await
        .expect("assign eng");
    (stu.student_no, stu.version, t_draft.id, t_pub.id)
}

#[tokio::test]
async fn withdrawal_voids_only_draft_grades_and_keeps_published() {
    let (backend, admin) = open_seeded_backend().await;
    let (student_no, version, t_draft_id, t_pub_id) = seed_teachings(&backend, &admin).await;

    let _draft = backend
        .save_draft_grade(
            &admin,
            SaveDraftGradeRequest {
                teaching_id: t_draft_id,
                student_no: student_no.clone(),
                score: Decimal::from(75),
                idempotency_key: "rt-g1".to_owned(),
            },
        )
        .await
        .expect("save math draft");
    let _pub_draft = backend
        .save_draft_grade(
            &admin,
            SaveDraftGradeRequest {
                teaching_id: t_pub_id.clone(),
                student_no: student_no.clone(),
                score: Decimal::from(88),
                idempotency_key: "rt-g2".to_owned(),
            },
        )
        .await
        .expect("save eng draft");
    let _pub = backend
        .publish_grades(
            &admin,
            PublishGradesRequest {
                teaching_id: t_pub_id,
                idempotency_key: "rt-pub".to_owned(),
            },
        )
        .await
        .expect("publish eng");

    let _withdrawn = backend
        .withdraw_student(
            &admin,
            &student_no,
            TransitionStudentRequest {
                reason: "转学退学".to_owned(),
                version,
                idempotency_key: "rt-w".to_owned(),
            },
        )
        .await
        .expect("withdraw dispatches outbox");

    let grades = backend
        .list_grades(&admin, Some(&student_no))
        .await
        .expect("list grades");
    let math_status = grades
        .iter()
        .find(|g| g.course_code == "MATH101")
        .map(|g| g.status);
    let eng_status = grades
        .iter()
        .find(|g| g.course_code == "ENG101")
        .map(|g| g.status);
    assert_eq!(math_status, Some(GradeStatus::Voided));
    assert_eq!(eng_status, Some(GradeStatus::Published));
}

#[tokio::test]
async fn disabling_teacher_blocks_sign_in_and_teaching_assignment() {
    let (backend, admin) = open_seeded_backend().await;

    let hr = backend
        .create_homeroom(
            &admin,
            CreateHomeroomRequest {
                name: "高一 2 班".to_owned(),
                grade_year: 2026,
                capacity: Some(30),
                idempotency_key: "rt-dis-hr".to_owned(),
            },
        )
        .await
        .expect("create homeroom");
    let course = backend
        .create_course(
            &admin,
            CreateCourseRequest {
                code: "CS101".to_owned(),
                name: "程序设计".to_owned(),
                max_score: Decimal::from(100),
                pass_score: Decimal::from(60),
                idempotency_key: "rt-dis-c".to_owned(),
            },
        )
        .await
        .expect("create course");
    let teacher = backend
        .provision_account(
            &admin,
            ProvisionAccountRequest {
                username: "disabled_teacher".to_owned(),
                display_name: "离职教师".to_owned(),
                role: Role::Teacher,
                bound_student_no: None,
                idempotency_key: "rt-dis-t".to_owned(),
            },
        )
        .await
        .expect("provision teacher");

    let _disabled = backend
        .disable_account(
            &admin,
            &teacher.account_id,
            DisableAccountRequest {
                reason: "调离教学岗".to_owned(),
                version: teacher.version,
                idempotency_key: "rt-dis-acc".to_owned(),
            },
        )
        .await
        .expect("disable teacher");

    let sign_in_err = backend
        .sign_in(SignInRequest {
            username: "disabled_teacher".to_owned(),
            password: "demo".to_owned(),
        })
        .await
        .expect_err("disabled teacher must fail sign-in");
    assert_eq!(sign_in_err.status, 403);

    let assign_err = backend
        .assign_teaching(
            &admin,
            AssignTeachingRequest {
                course_id: course.id,
                homeroom_id: hr.id,
                term: "2026-秋".to_owned(),
                teacher_account_id: teacher.account_id,
                idempotency_key: "rt-dis-assign".to_owned(),
            },
        )
        .await
        .expect_err("disabled teacher must fail port verification");
    assert_eq!(assign_err.status, 409);
}
