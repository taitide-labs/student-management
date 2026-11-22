use student_management_gateway::contract::Role;

use super::command::build_request;

#[test]
fn provision_command_requires_student_no_only_for_student_role() {
    assert!(build_request(" ", "王老师", Role::Teacher, "", "k1".to_owned()).is_err());
    assert!(build_request("stu2", "李四", Role::Student, "", "k2".to_owned()).is_err());
    let req = build_request("wanglaoshi", "王老师", Role::Teacher, "", "k3".to_owned())
        .expect("valid teacher");
    assert_eq!(req.bound_student_no, None);
}
