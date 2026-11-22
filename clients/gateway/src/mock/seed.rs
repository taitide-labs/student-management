use rust_decimal::Decimal;

use crate::contract::{
    AccountSession, AccountStatus, AccountSummary, CourseSummary, EnrollmentChangeRecord,
    EnrollmentStatus, GradeEntrySummary, GradeStatus, HomeroomSummary, Role, StudentSummary,
    TeachingSummary,
};

pub(super) fn seed_accounts() -> Vec<AccountSummary> {
    vec![
        AccountSummary {
            account_id: "acc-admin-1".to_owned(),
            username: "admin".to_owned(),
            display_name: "林教务".to_owned(),
            role: Role::Admin,
            bound_student_no: None,
            status: AccountStatus::Active,
            version: 1,
        },
        AccountSummary {
            account_id: "acc-teacher-1".to_owned(),
            username: "teacher".to_owned(),
            display_name: "陈老师".to_owned(),
            role: Role::Teacher,
            bound_student_no: None,
            status: AccountStatus::Active,
            version: 1,
        },
        AccountSummary {
            account_id: "acc-student-1".to_owned(),
            username: "student".to_owned(),
            display_name: "张晓明".to_owned(),
            role: Role::Student,
            bound_student_no: Some("20261001".to_owned()),
            status: AccountStatus::Active,
            version: 1,
        },
    ]
}

pub(super) fn to_session(acc: &AccountSummary) -> AccountSession {
    AccountSession {
        account_id: acc.account_id.clone(),
        username: acc.username.clone(),
        display_name: acc.display_name.clone(),
        role: acc.role,
        bound_student_no: acc.bound_student_no.clone(),
        access_token: format!("stub-token-{}", acc.username),
    }
}

pub(super) fn seed_courses() -> Vec<CourseSummary> {
    vec![
        CourseSummary {
            id: "crs-math101".to_owned(),
            code: "MATH101".to_owned(),
            name: "高等数学基础".to_owned(),
            max_score: Decimal::new(100, 0),
            pass_score: Decimal::new(60, 0),
            version: 1,
        },
        CourseSummary {
            id: "crs-cs102".to_owned(),
            code: "CS102".to_owned(),
            name: "程序设计导论".to_owned(),
            max_score: Decimal::new(100, 0),
            pass_score: Decimal::new(60, 0),
            version: 1,
        },
    ]
}

pub(super) fn seed_teachings() -> Vec<TeachingSummary> {
    vec![
        TeachingSummary {
            id: "teach-1".to_owned(),
            course_id: "crs-math101".to_owned(),
            course_code: "MATH101".to_owned(),
            course_name: "高等数学基础".to_owned(),
            homeroom_id: "hr-2026-1".to_owned(),
            homeroom_name: "高一 1 班".to_owned(),
            term: "2026 秋季".to_owned(),
            teacher_account_id: "acc-teacher-1".to_owned(),
            teacher_name: "陈老师".to_owned(),
            version: 1,
        },
        TeachingSummary {
            id: "teach-2".to_owned(),
            course_id: "crs-cs102".to_owned(),
            course_code: "CS102".to_owned(),
            course_name: "程序设计导论".to_owned(),
            homeroom_id: "hr-2026-1".to_owned(),
            homeroom_name: "高一 1 班".to_owned(),
            term: "2026 秋季".to_owned(),
            teacher_account_id: "acc-teacher-1".to_owned(),
            teacher_name: "陈老师".to_owned(),
            version: 1,
        },
    ]
}

pub(super) fn seed_homerooms() -> Vec<HomeroomSummary> {
    vec![
        HomeroomSummary {
            id: "hr-2026-1".to_owned(),
            name: "高一 1 班".to_owned(),
            grade_year: 2026,
            capacity: Some(40),
            active_count: 3,
            version: 1,
        },
        HomeroomSummary {
            id: "hr-2026-2".to_owned(),
            name: "高一 2 班".to_owned(),
            grade_year: 2026,
            capacity: Some(2),
            active_count: 2,
            version: 1,
        },
        HomeroomSummary {
            id: "hr-2025-1".to_owned(),
            name: "高二 1 班".to_owned(),
            grade_year: 2025,
            capacity: None,
            active_count: 1,
            version: 1,
        },
    ]
}

pub(super) fn seed_students() -> Vec<StudentSummary> {
    vec![
        StudentSummary {
            student_no: "20261001".to_owned(),
            full_name: "张晓明".to_owned(),
            homeroom_id: "hr-2026-1".to_owned(),
            homeroom_name: "高一 1 班".to_owned(),
            status: EnrollmentStatus::Active,
            enrolled_on: "2026-09-01".to_owned(),
            version: 1,
        },
        StudentSummary {
            student_no: "20261002".to_owned(),
            full_name: "李思涵".to_owned(),
            homeroom_id: "hr-2026-1".to_owned(),
            homeroom_name: "高一 1 班".to_owned(),
            status: EnrollmentStatus::Active,
            enrolled_on: "2026-09-01".to_owned(),
            version: 2,
        },
        StudentSummary {
            student_no: "20261003".to_owned(),
            full_name: "王梓轩".to_owned(),
            homeroom_id: "hr-2026-2".to_owned(),
            homeroom_name: "高一 2 班".to_owned(),
            status: EnrollmentStatus::Suspended,
            enrolled_on: "2026-09-01".to_owned(),
            version: 3,
        },
        StudentSummary {
            student_no: "20261004".to_owned(),
            full_name: "赵雨桐".to_owned(),
            homeroom_id: "hr-2026-2".to_owned(),
            homeroom_name: "高一 2 班".to_owned(),
            status: EnrollmentStatus::Withdrawn,
            enrolled_on: "2026-09-01".to_owned(),
            version: 4,
        },
        StudentSummary {
            student_no: "20251005".to_owned(),
            full_name: "周子昂".to_owned(),
            homeroom_id: "hr-2025-1".to_owned(),
            homeroom_name: "高二 1 班".to_owned(),
            status: EnrollmentStatus::Graduated,
            enrolled_on: "2025-09-01".to_owned(),
            version: 5,
        },
    ]
}

pub(super) fn seed_changes() -> Vec<EnrollmentChangeRecord> {
    vec![
        EnrollmentChangeRecord {
            id: "chg-1".to_owned(),
            student_no: "20261001".to_owned(),
            action: "入学建档".to_owned(),
            from_summary: "无".to_owned(),
            to_summary: "高一 1 班 · 在籍".to_owned(),
            reason: "新生报到".to_owned(),
            operator_name: "林教务".to_owned(),
            occurred_at: "2026-09-01 09:00".to_owned(),
        },
        EnrollmentChangeRecord {
            id: "chg-2".to_owned(),
            student_no: "20261003".to_owned(),
            action: "休学登记".to_owned(),
            from_summary: "高一 2 班 · 在籍".to_owned(),
            to_summary: "高一 2 班 · 休学".to_owned(),
            reason: "因病申请休学一学期".to_owned(),
            operator_name: "林教务".to_owned(),
            occurred_at: "2026-10-12 14:20".to_owned(),
        },
    ]
}

pub(super) fn seed_grades() -> Vec<GradeEntrySummary> {
    vec![
        GradeEntrySummary {
            id: "grd-1".to_owned(),
            teaching_id: "teach-1".to_owned(),
            course_code: "MATH101".to_owned(),
            course_name: "高等数学基础".to_owned(),
            term: "2026 秋季".to_owned(),
            student_no: "20261001".to_owned(),
            student_name: "张晓明".to_owned(),
            score: Decimal::new(925, 1),
            max_score: Decimal::new(100, 0),
            status: GradeStatus::Published,
            version: 1,
        },
        GradeEntrySummary {
            id: "grd-2".to_owned(),
            teaching_id: "teach-2".to_owned(),
            course_code: "CS102".to_owned(),
            course_name: "程序设计导论".to_owned(),
            term: "2026 秋季".to_owned(),
            student_no: "20261001".to_owned(),
            student_name: "张晓明".to_owned(),
            score: Decimal::new(88, 0),
            max_score: Decimal::new(100, 0),
            status: GradeStatus::Draft,
            version: 1,
        },
    ]
}
