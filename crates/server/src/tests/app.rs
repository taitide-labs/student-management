use crate::app::service_name;

#[test]
fn service_name_matches_package() {
    assert_eq!(service_name(), "student-management-server");
}
