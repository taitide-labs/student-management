use super::command::build_request;

#[test]
fn disable_account_requires_reason() {
    assert!(build_request("   ", 1, "k1".to_owned()).is_err());
    let req = build_request(" 岗位调整 ", 1, "k2".to_owned()).expect("valid disable");
    assert_eq!(req.reason, "岗位调整");
}
