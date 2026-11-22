use rust_decimal::Decimal;
use student_management_primitives::{CourseCode, MaxScore, NonEmptyText, Score};

use crate::{Course, GradingDomainError};

#[test]
fn course_rejects_pass_score_above_max() {
    let res = Course::create(
        "c-1".to_owned(),
        CourseCode::try_new("math101").expect("code"),
        NonEmptyText::try_new("课程名称", "高等数学", 64).expect("name"),
        MaxScore::try_new(Decimal::from(100)).expect("max"),
        Score::try_new(Decimal::from(101)).expect("pass"),
    );
    assert!(matches!(
        res,
        Err(GradingDomainError::PassScoreExceedsMax { .. })
    ));
}
