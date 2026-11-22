use thiserror::Error;

#[derive(Debug, Error)]
pub enum InfraError {
    #[error("数据库操作失败：{0}")]
    Database(#[from] sqlx::Error),
    #[error("数据库迁移失败：{0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),
    #[error("序列化失败：{0}")]
    Serialization(#[from] serde_json::Error),
    #[error("密码哈希处理失败：{0}")]
    PasswordHash(String),
    #[error("幂等键被用于不同请求载荷")]
    IdempotencyConflict,
    #[error("事件投递失败：{0}")]
    EventDispatch(String),
}
