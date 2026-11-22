use super::command::build_request;

#[test]
fn amend_grade_requires_numeric_score_and_reason() {
    assert!(build_request("bad", "复核", 1, "k1".to_owned()).is_err());
    assert!(build_request("95", "   ", 1, "k2".to_owned()).is_err());
    let req = build_request("95", " 试卷复核加分 ", 1, "k3".to_owned()).expect("valid amend");
    assert_eq!(req.reason, "试卷复核加分");
}
