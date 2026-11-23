use crate::{IdempotencyKey, NonEmptyText, PositiveVersion, ReasonText};

#[test]
fn non_empty_text_enforces_exact_bounds_and_displays_value() {
    let exact = NonEmptyText::try_new("字段", "abcd", 4).expect("exact length");
    assert_eq!(exact.as_str(), "abcd");
    assert_eq!(format!("{exact}"), "abcd");
    assert!(NonEmptyText::try_new("字段", "abcde", 4).is_err());
    assert!(NonEmptyText::try_new("字段", "   ", 4).is_err());
}

#[test]
fn reason_and_idempotency_key_enforce_boundaries() {
    let reason = ReasonText::try_new("复核成绩").expect("valid reason");
    assert_eq!(reason.as_str(), "复核成绩");
    assert_eq!(format!("{reason}"), "复核成绩");
    assert!(ReasonText::try_new(&"文".repeat(ReasonText::MAX_CHARS)).is_ok());
    assert!(ReasonText::try_new(&"文".repeat(ReasonText::MAX_CHARS + 1)).is_err());

    let key = IdempotencyKey::try_new("req-1").expect("valid key");
    assert_eq!(key.as_str(), "req-1");
    assert!(IdempotencyKey::try_new(&"k".repeat(IdempotencyKey::MAX_CHARS)).is_ok());
    assert!(IdempotencyKey::try_new(&"k".repeat(IdempotencyKey::MAX_CHARS + 1)).is_err());
    assert!(IdempotencyKey::try_new("bad key").is_err());
}

#[test]
fn positive_version_rejects_zero_and_returns_exact_value() {
    assert!(PositiveVersion::try_new(0).is_err());
    let v2 = PositiveVersion::try_new(2).expect("version 2");
    assert_eq!(v2.get(), 2);
    assert_eq!(v2.next().get(), 3);
}
