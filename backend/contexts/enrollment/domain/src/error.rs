use student_management_contracts::EnrollmentStatus;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum EnrollmentDomainError {
    #[error("行政班额定容量已满（{active_count}/{capacity}）")]
    HomeroomCapacityExceeded { capacity: u32, active_count: u32 },
    #[error("目标额定容量（{capacity}）不能低于当前在籍人数（{active_count}）")]
    CapacityBelowActiveCount { capacity: u32, active_count: u32 },
    #[error("学籍已处于不可逆终态（{0:?}），常规流转已被禁止")]
    TerminalStatusLocked(EnrollmentStatus),
    #[error("仅在籍学生允许执行转班操作")]
    TransferRequiresActiveStatus,
    #[error("当前状态（{from:?}）不支持该学籍流转操作")]
    InvalidTransition { from: EnrollmentStatus },
    #[error("仅处于退学或毕业终态的档案才允许执行误操作冲正")]
    ReversalRequiresTerminalStatus,
    #[error("冲正目标状态必须为非终态（在籍或休学）")]
    ReversalTargetMustBeNonTerminal,
    #[error("学生已归属目标行政班")]
    SameHomeroomTransfer,
    #[error("乐观锁版本冲突：期望 {expected}，实际 {actual}")]
    VersionConflict { expected: u64, actual: u64 },
}
