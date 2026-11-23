use proptest::prelude::*;
use rust_decimal::Decimal;

use crate::{CourseCode, MaxScore, Score, TermCode};

proptest! {
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
fn max_score_and_term_enforce_non_empty_positive_bounds() {
    assert!(CourseCode::try_new("a").is_err());
    assert!(CourseCode::try_new(&"a".repeat(25)).is_err());
    assert!(MaxScore::try_new(Decimal::ZERO).is_err());
    assert!(MaxScore::try_new(Decimal::new(1234, 3)).is_err());
    assert_eq!(
        MaxScore::try_new(Decimal::new(10025, 2))
            .expect("max")
            .get(),
        Decimal::new(10025, 2)
    );
    assert!(Score::try_new(Decimal::new(1234, 3)).is_err());
    let s = Score::try_new(Decimal::new(855, 1)).expect("score");
    assert_eq!(format!("{s}"), "85.5");
    assert!(TermCode::try_new("   ").is_err());
    let term = TermCode::try_new("2026-春").expect("term");
    assert_eq!(term.as_str(), "2026-春");
}
