use proptest::prelude::*;
use rust_decimal::Decimal;

use crate::{
    CourseCode, GradeYear, IdempotencyKey, MaxScore, PositiveVersion, ReasonText, Score, StudentNo,
    Username,
};

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

    #[test]
    fn course_code_normalizes_ascii_to_uppercase(raw in "[a-zA-Z0-9_-]{2,24}") {
        let code = CourseCode::try_new(&raw).expect("valid course code");
        prop_assert_eq!(code.as_str(), raw.to_ascii_uppercase());
    }

    #[test]
    fn score_rejects_negative_and_accepts_two_decimals(cents in 0u32..=100_000u32) {
        let dec = Decimal::new(i64::from(cents), 2);
        let score = Score::try_new(dec).expect("non-negative score");
        prop_assert_eq!(score.get(), dec.normalize());
        let negative = Score::try_new(-Decimal::ONE);
        prop_assert!(negative.is_err());
    }
}

#[test]
fn primitive_guards_reject_boundary_violations() {
    assert!(GradeYear::try_new(1899).is_err());
    assert!(GradeYear::try_new(3000).is_err());
    assert!(StudentNo::try_new("123").is_err());
    assert!(StudentNo::try_new("2026abcd").is_err());
    assert!(Username::try_new("Admin_01").is_ok());
    assert!(Username::try_new("a").is_err());
    assert!(ReasonText::try_new("   ").is_err());
    assert!(IdempotencyKey::try_new("key with space").is_err());
    assert!(MaxScore::try_new(Decimal::ZERO).is_err());
    assert!(PositiveVersion::try_new(0).is_err());
    assert_eq!(
        PositiveVersion::try_new(1).expect("version 1").next().get(),
        2
    );
}
