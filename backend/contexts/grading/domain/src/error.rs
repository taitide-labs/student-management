use rust_decimal::Decimal;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum GradingDomainError {
    #[error("及格线（{pass_score}）不能大于满分（{max_score}）")]
    PassScoreExceedsMax {
        pass_score: Decimal,
        max_score: Decimal,
    },
    #[error("成绩分值（{score}）超出课程满分上限（{max_score}）")]
    ScoreExceedsMax { score: Decimal, max_score: Decimal },
    #[error("成绩已锁定发布，仅允许管理员走修订流程")]
    PublishedGradeLocked,
    #[error("已作废成绩不允许修改或发布")]
    VoidedGradeLocked,
    #[error("仅已锁定发布的成绩才允许执行合规修订")]
    AmendmentRequiresPublishedGrade,
    #[error("仅教务管理员有权修订已发布成绩")]
    AmendmentRequiresAdmin,
    #[error("新分值与当前分值相同，无需重复修订")]
    AmendmentScoreUnchanged,
    #[error("乐观锁版本冲突：期望 {expected}，实际 {actual}")]
    VersionConflict { expected: u64, actual: u64 },
}
