use super::command::{build_publish, build_save_draft};

#[test]
fn draft_grade_commands_validate_inputs() {
    assert!(build_save_draft("", "20261001", "88", "k1".to_owned()).is_err());
    assert!(build_save_draft("teach-2", "20261001", "abc", "k2".to_owned()).is_err());
    let draft =
        build_save_draft("teach-2", " 20261001 ", "91.5", "k3".to_owned()).expect("valid draft");
    assert_eq!(draft.student_no, "20261001");
    assert!(build_publish(" ", "k4".to_owned()).is_err());
}
