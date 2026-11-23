use crate::Username;

#[test]
fn username_normalizes_case_and_rejects_invalid_length_or_chars() {
    let parsed = Username::try_new("Admin_01").expect("valid username");
    assert_eq!(parsed.as_str(), "admin_01");
    assert_eq!(format!("{parsed}"), "admin_01");
    assert_eq!(Username::try_new("ab").expect("min len").as_str(), "ab");
    assert!(Username::try_new(&"u".repeat(32)).is_ok());
    assert!(Username::try_new(&"u".repeat(33)).is_err());
    assert!(Username::try_new("a").is_err());
    assert!(Username::try_new("bad name").is_err());
}
