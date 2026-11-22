use student_management_gateway::contract::EnrollmentStatus;

use super::super::store::StatusFilter;

#[test]
fn status_filter_maps_all_variants() {
    assert_eq!(StatusFilter::All.as_status(), None);
    assert_eq!(
        StatusFilter::Active.as_status(),
        Some(EnrollmentStatus::Active)
    );
    assert_eq!(
        StatusFilter::Suspended.as_status(),
        Some(EnrollmentStatus::Suspended)
    );
    assert_eq!(
        StatusFilter::Withdrawn.as_status(),
        Some(EnrollmentStatus::Withdrawn)
    );
    assert_eq!(
        StatusFilter::Graduated.as_status(),
        Some(EnrollmentStatus::Graduated)
    );
}
