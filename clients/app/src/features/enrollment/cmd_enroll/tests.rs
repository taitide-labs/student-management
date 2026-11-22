use super::command::build_request;

#[test]
fn enroll_request_rejects_empty_name_or_missing_homeroom() {
    assert!(build_request("   ", Some("hr-2026-1"), "k1".to_owned()).is_err());
    assert!(build_request("周子昂", None, "k2".to_owned()).is_err());
    let req = build_request(" 周子昂 ", Some("hr-2026-1"), "k3".to_owned()).expect("valid enroll");
    assert_eq!(req.full_name, "周子昂");
    assert_eq!(req.homeroom_id, "hr-2026-1");
}
