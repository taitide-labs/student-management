use proptest::prelude::*;

use crate::{GradeYear, HomeroomName, StudentNo};

proptest! {
    #[test]
    fn student_no_roundtrips_valid_years_and_sequences(
        year in 1900u16..=2999u16,
        seq in 1u32..=9999u32,
    ) {
        let grade_year = GradeYear::try_new(year).expect("valid year");
        let student_no = StudentNo::from_sequence(grade_year, seq).expect("valid sequence");
        let reparsed = StudentNo::try_new(student_no.as_str()).expect("8-digit code parses");
        prop_assert_eq!(reparsed, student_no);
    }
}

#[test]
fn enrollment_primitives_reject_invalid_values() {
    assert!(GradeYear::try_new(1899).is_err());
    assert!(GradeYear::try_new(3000).is_err());
    assert_eq!(GradeYear::try_new(2026).expect("2026").get(), 2026);
    let homeroom = HomeroomName::try_new("高一 (1) 班").expect("homeroom");
    assert_eq!(homeroom.as_str(), "高一 (1) 班");
    assert!(HomeroomName::try_new("   ").is_err());
    assert!(StudentNo::try_new("123").is_err());
    assert!(StudentNo::try_new("2026abcd").is_err());
    let sno = StudentNo::try_new("20260001").expect("student no");
    assert_eq!(format!("{sno}"), "20260001");
}
