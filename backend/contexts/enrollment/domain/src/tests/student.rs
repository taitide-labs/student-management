use proptest::prelude::*;
use student_management_contracts::EnrollmentStatus;
use student_management_primitives::{NonEmptyText, PositiveVersion, ReasonText, StudentNo};

use crate::{EnrollmentDomainError, EnrollmentTransition, Student};

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
