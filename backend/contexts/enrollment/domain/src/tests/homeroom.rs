use std::num::NonZeroU32;

use student_management_primitives::{GradeYear, HomeroomName};

use crate::{EnrollmentDomainError, Homeroom};

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
