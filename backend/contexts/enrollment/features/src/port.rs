use std::{future::Future, pin::Pin};

use sqlx::{Row, SqlitePool};
use student_management_contracts::{
    EnrollmentQueryPort, EnrollmentStatus, HomeroomSnapshot, PortError, StudentEnrollmentSnapshot,
};

#[derive(Debug, Clone)]
pub struct SqliteEnrollmentQueryPort {
    pool: SqlitePool,
}

impl SqliteEnrollmentQueryPort {
    #[must_use]
    pub const fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

impl EnrollmentQueryPort for SqliteEnrollmentQueryPort {
    fn verify_homeroom_exists(
        &self,
        homeroom_id: &str,
    ) -> Pin<Box<dyn Future<Output = Result<HomeroomSnapshot, PortError>> + Send + '_>> {
        let id = homeroom_id.to_owned();
        Box::pin(async move {
            let row = sqlx::query("SELECT id, name FROM homerooms WHERE id = ?")
                .bind(&id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|err| PortError::Conflict(err.to_string()))?;

            let Some(row) = row else {
                return Err(PortError::NotFound(format!("行政班 {id} 不存在")));
            };

            Ok(HomeroomSnapshot {
                id,
                name: row
                    .try_get("name")
                    .map_err(|err| PortError::Conflict(err.to_string()))?,
            })
        })
    }

    fn check_student_active(
        &self,
        student_no: &str,
    ) -> Pin<Box<dyn Future<Output = Result<StudentEnrollmentSnapshot, PortError>> + Send + '_>>
    {
        let no = student_no.to_owned();
        Box::pin(async move {
            let row = sqlx::query(
                "SELECT student_no, full_name, homeroom_id, status FROM students WHERE student_no = ?",
            )
            .bind(&no)
            .fetch_optional(&self.pool)
            .await
            .map_err(|err| PortError::Conflict(err.to_string()))?;

            let Some(row) = row else {
                return Err(PortError::NotFound(format!("学生 {no} 不存在")));
            };

            let status_raw: String = row
                .try_get("status")
                .map_err(|err| PortError::Conflict(err.to_string()))?;
            let status = match status_raw.as_str() {
                "active" => EnrollmentStatus::Active,
                "suspended" => EnrollmentStatus::Suspended,
                "withdrawn" => EnrollmentStatus::Withdrawn,
                "graduated" => EnrollmentStatus::Graduated,
                _ => return Err(PortError::Conflict(format!("未知学籍状态：{status_raw}"))),
            };
            if status != EnrollmentStatus::Active {
                return Err(PortError::Conflict(format!(
                    "学生 {no} 当前为「{}」，非在籍状态",
                    status.label()
                )));
            }

            Ok(StudentEnrollmentSnapshot {
                student_no: no,
                full_name: row
                    .try_get("full_name")
                    .map_err(|err| PortError::Conflict(err.to_string()))?,
                homeroom_id: row
                    .try_get("homeroom_id")
                    .map_err(|err| PortError::Conflict(err.to_string()))?,
                status,
            })
        })
    }
}
