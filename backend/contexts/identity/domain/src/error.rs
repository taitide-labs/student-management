use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum IdentityDomainError {
    #[error("仅学生角色允许绑定学号")]
    StudentBindingRequiresStudentRole,
    #[error("已停用账号禁止修改属性或绑定档案")]
    DisabledAccountImmutable,
    #[error("禁止停用当前正在执行操作的本人账号")]
    SelfDisableForbidden,
    #[error("账号已处于停用状态")]
    AlreadyDisabled,
    #[error("乐观锁版本冲突：期望 {expected}，实际 {actual}")]
    VersionConflict { expected: u64, actual: u64 },
}
