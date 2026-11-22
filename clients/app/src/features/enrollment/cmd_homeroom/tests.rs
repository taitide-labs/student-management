use super::command::{build_create, build_update_capacity};

#[test]
fn homeroom_command_parses_year_and_optional_capacity() {
    assert!(build_create("  ", "2026", "40", "k1".to_owned()).is_err());
    assert!(build_create("高一 3 班", "abc", "40", "k2".to_owned()).is_err());
    let created = build_create("高一 3 班", "2026", "", "k3".to_owned()).expect("unbounded cap");
    assert_eq!(created.capacity, None);

    let cap = build_update_capacity("45", 2, "k4".to_owned()).expect("valid cap update");
    assert_eq!(cap.capacity, Some(45));
}
