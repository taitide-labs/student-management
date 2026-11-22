use std::num::NonZeroU32;

use proptest::prelude::*;
use student_management_contracts::EnrollmentStatus;
use student_management_primitives::{
    GradeYear, HomeroomName, NonEmptyText, PositiveVersion, ReasonText, StudentNo,
};

use crate::{EnrollmentDomainError, EnrollmentTransition, Homeroom, Student};

#[test]
fn homeroom_capacity_guards_enroll_and_capacity_shrink() {
    let mut hr = Homeroom::create(
        "hr-1".to_owned(),
        HomeroomName::try_new("高一 (1) 班").expect("name"),
        GradeYear::try_new(2026).expect("year"),
        Some(NonZeroU32::new(2).expect("cap")),
    );
    assert!(hr.ensure_seat_available(1).is_ok());
    assert_eq!(
        hr.ensure_seat_available(2),
        Err(EnrollmentDomainError::HomeroomCapacityExceeded {
            capacity: 2,
            active_count: 2,
        })
    );
    assert_eq!(
        hr.update_capacity(hr.version, Some(NonZeroU32::new(1).expect("cap")), 2),
        Err(EnrollmentDomainError::CapacityBelowActiveCount {
            capacity: 1,
            active_count: 2,
        })
    );
}

#[test]
fn terminal_status_blocks_regular_transitions_until_explicit_reversal() {
    let reason = ReasonText::try_new("办理退学").expect("reason");
    let (mut student, _) = Student::enroll(
        StudentNo::try_new("20261001").expect("no"),
        NonEmptyText::try_new("姓名", "张晓明", 32).expect("name"),
        "hr-1",
        "2026-09-01".to_owned(),
    );

    student
        .transition(student.version, EnrollmentTransition::Withdraw, &reason)
        .expect("withdraw");
    assert_eq!(student.status, EnrollmentStatus::Withdrawn);

    assert_eq!(
        student.transition(student.version, EnrollmentTransition::Resume, &reason),
        Err(EnrollmentDomainError::TerminalStatusLocked(
            EnrollmentStatus::Withdrawn
        ))
    );

    let reversal_reason = ReasonText::try_new("误操作冲正恢复在籍").expect("reason");
    let draft = student
        .reverse_terminal_status(student.version, EnrollmentStatus::Active, &reversal_reason)
        .expect("explicit reversal");
    assert_eq!(student.status, EnrollmentStatus::Active);
    assert_eq!(draft.action, "终态冲正");
}

proptest! {
    #[test]
    fn terminal_student_rejects_every_non_reversal_transition(kind_idx in 0u8..5u8) {
        let (mut student, _) = Student::enroll(
            StudentNo::try_new("20261099").expect("no"),
            NonEmptyText::try_new("姓名", "测试学生", 32).expect("name"),
            "hr-1",
            "2026-09-01".to_owned(),
        );
        let reason = ReasonText::try_new("正常毕业").expect("reason");
        student
            .transition(PositiveVersion::try_new(1).expect("v1"), EnrollmentTransition::Graduate, &reason)
            .expect("graduate");

        let res = match kind_idx {
            0 => student.transition(student.version, EnrollmentTransition::Suspend, &reason),
            1 => student.transition(student.version, EnrollmentTransition::Resume, &reason),
            2 => student.transition(student.version, EnrollmentTransition::Withdraw, &reason),
            3 => student.transition(student.version, EnrollmentTransition::Graduate, &reason),
            _ => student.transfer_homeroom(student.version, "hr-2".to_owned(), &reason),
        };
        prop_assert_eq!(
            res,
            Err(EnrollmentDomainError::TerminalStatusLocked(EnrollmentStatus::Graduated))
        );
    }
}
