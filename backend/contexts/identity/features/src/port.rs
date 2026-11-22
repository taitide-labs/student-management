use std::{future::Future, pin::Pin};

use sqlx::{Row, SqlitePool};
use student_management_contracts::{IdentityLookupPort, PortError, TeacherIdentity};

#[derive(Debug, Clone)]
pub struct SqliteIdentityLookupPort {
    pool: SqlitePool,
}

impl SqliteIdentityLookupPort {
    #[must_use]
    pub const fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

impl IdentityLookupPort for SqliteIdentityLookupPort {
    fn verify_teacher(
        &self,
        account_id: &str,
    ) -> Pin<Box<dyn Future<Output = Result<TeacherIdentity, PortError>> + Send + '_>> {
        let id = account_id.to_owned();
        Box::pin(async move {
            let row = sqlx::query(
                "SELECT account_id, display_name, role, status FROM accounts WHERE account_id = ?",
            )
            .bind(&id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|err| PortError::Conflict(err.to_string()))?;

            let Some(row) = row else {
                return Err(PortError::NotFound(format!("教师账号 {id} 不存在")));
            };

            let status: String = row
                .try_get("status")
                .map_err(|err| PortError::Conflict(err.to_string()))?;
            if status != "active" {
                return Err(PortError::Conflict(format!("教师账号 {id} 已停用")));
            }

            let role: String = row
                .try_get("role")
                .map_err(|err| PortError::Conflict(err.to_string()))?;
            if role != "teacher" {
                return Err(PortError::Forbidden(format!("账号 {id} 不是教师角色")));
            }

            let display_name: String = row
                .try_get("display_name")
                .map_err(|err| PortError::Conflict(err.to_string()))?;

            Ok(TeacherIdentity {
                account_id: id,
                display_name,
            })
        })
    }
}
