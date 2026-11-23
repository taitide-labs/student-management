use proptest::prelude::*;
use rust_decimal::Decimal;
use student_management_contracts::GradeStatus;
use student_management_primitives::{MaxScore, ReasonText, Score, StudentNo};

use crate::{GradeEntry, GradingDomainError};

#[test]
fn published_grade_locks_draft_edits_and_allows_admin_amendment() {
    let max = MaxScore::try_new(Decimal::from(100)).expect("max");
    let mut entry = GradeEntry::new_draft(
        "g-1".to_owned(),
        "t-1".to_owned(),
        StudentNo::try_new("20261001").expect("no"),
        Score::try_new(Decimal::from(88)).expect("score"),
        max,
    )
    .expect("draft");

    entry.publish().expect("publish");
    assert_eq!(entry.status, GradeStatus::Published);

    let overwrite = entry.save_draft(Score::try_new(Decimal::from(90)).expect("score"), max);
    assert_eq!(overwrite, Err(GradingDomainError::PublishedGradeLocked));

    let reason = ReasonText::try_new("试卷加分复核").expect("reason");
    let non_admin = entry.amend(
        false,
        entry.version,
        Score::try_new(Decimal::from(92)).expect("new score"),
        max,
        &reason,
    );
    assert_eq!(non_admin, Err(GradingDomainError::AmendmentRequiresAdmin));

    let amendment = entry
        .amend(
            true,
            entry.version,
            Score::try_new(Decimal::from(92)).expect("new score"),
            max,
            &reason,
        )
        .expect("admin amends");
    assert_eq!(amendment.previous_score, Decimal::from(88));
    assert_eq!(amendment.new_score, Decimal::from(92));
    assert_eq!(entry.score.get(), Decimal::from(92));

    let exact_max = GradeEntry::new_draft(
        "g-max".to_owned(),
        "t-1".to_owned(),
        StudentNo::try_new("20261001").expect("no"),
        Score::try_new(Decimal::from(100)).expect("score"),
        max,
    );
    assert!(exact_max.is_ok());

    let over_max = GradeEntry::new_draft(
        "g-over".to_owned(),
        "t-1".to_owned(),
        StudentNo::try_new("20261001").expect("no"),
        Score::try_new(Decimal::from(101)).expect("score"),
        max,
    );
    assert!(matches!(
        over_max,
        Err(GradingDomainError::ScoreExceedsMax { .. })
    ));
}

proptest! {
    #[test]
    fn withdrawal_voids_only_draft_grades_and_preserves_published(initial_published in any::<bool>()) {
        let max = MaxScore::try_new(Decimal::from(100)).expect("max");
        let mut entry = GradeEntry::new_draft(
            "g-prop".to_owned(),
            "t-prop".to_owned(),
            StudentNo::try_new("20261002").expect("no"),
            Score::try_new(Decimal::from(75)).expect("score"),
            max,
        ).expect("draft");

        if initial_published {
            entry.publish().expect("publish");
        }

        let voided = entry.void_if_draft();
        if initial_published {
            prop_assert!(!voided);
            prop_assert_eq!(entry.status, GradeStatus::Published);
        } else {
            prop_assert!(voided);
            prop_assert_eq!(entry.status, GradeStatus::Voided);
        }
    }
}
