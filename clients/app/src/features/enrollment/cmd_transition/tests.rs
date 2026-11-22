use student_management_gateway::contract::EnrollmentStatus;

use super::command::{ActionKind, allowed_actions, build_reverse, build_transfer, validate_reason};

#[test]
fn transition_reason_validation_requires_non_empty_reason() {
    assert!(validate_reason("  ").is_err());
    assert_eq!(validate_reason(" 因病休学 ").as_deref(), Ok("因病休学"));
    assert!(build_transfer(None, "转班", 1, "k1".to_owned()).is_err());
    assert!(build_reverse(EnrollmentStatus::Withdrawn, "冲正", 1, "k2".to_owned()).is_err());
}

#[test]
fn allowed_actions_follow_enrollment_state_machine() {
    assert_eq!(
        allowed_actions(EnrollmentStatus::Active),
        vec![
            ActionKind::Transfer,
            ActionKind::Suspend,
            ActionKind::Withdraw,
            ActionKind::Graduate,
        ]
    );
    assert_eq!(
        allowed_actions(EnrollmentStatus::Suspended),
        vec![ActionKind::Resume, ActionKind::Withdraw]
    );
    assert_eq!(
        allowed_actions(EnrollmentStatus::Withdrawn),
        vec![ActionKind::Reverse]
    );
    assert_eq!(
        allowed_actions(EnrollmentStatus::Graduated),
        vec![ActionKind::Reverse]
    );
}
