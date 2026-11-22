use crate::Username;

#[test]
fn username_normalizes_case_and_rejects_invalid_length_or_chars() {
    let parsed = Username::try_new("Admin_01").expect("valid username");
    assert_eq!(parsed.as_str(), "admin_01");
    assert!(Username::try_new("a").is_err());
    assert!(Username::try_new("bad name").is_err());
}
