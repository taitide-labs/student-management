use rust_decimal::Decimal;
use sqlx::{Row, SqlitePool};
use student_management_contracts::{GradeStatus, events::IntegrationEvent};
use student_management_grading_domain::GradeEntry;
use student_management_infra::{
    InfraError,
    outbox::{DispatchFuture, OutboxSubscription},
};
use student_management_primitives::{PositiveVersion, Score, StudentNo};

async fn void_drafts_for_student(
    pool: &SqlitePool,
    student_no_raw: &str,
    occurred_at: &str,
) -> Result<(), InfraError> {
    let mut tx = pool.begin().await?;
    let rows = sqlx::query(
        "SELECT id, teaching_id, score, version FROM grade_entries \
         WHERE student_no = ? AND status = 'draft'",
    )
    .bind(student_no_raw)
    .fetch_all(&mut *tx)
    .await?;

    let student_no = StudentNo::try_new(student_no_raw)
        .map_err(|err| InfraError::EventDispatch(err.to_string()))?;

    for row in rows {
        let id: String = row.try_get("id")?;
        let teaching_id: String = row.try_get("teaching_id")?;
        let score_raw: String = row.try_get("score")?;
        let version_raw: i64 = row.try_get("version")?;

        let score = score_raw
            .parse::<Decimal>()
            .ok()
            .and_then(|val| Score::try_new(val).ok())
            .ok_or_else(|| InfraError::EventDispatch("成绩分值异常".to_owned()))?;
        let version = u64::try_from(version_raw)
            .ok()
            .and_then(|val| PositiveVersion::try_new(val).ok())
            .ok_or_else(|| InfraError::EventDispatch("成绩版本号异常".to_owned()))?;

        let mut entry = GradeEntry {
            id: id.clone(),
            teaching_id,
            student_no: student_no.clone(),
            score,
            status: GradeStatus::Draft,
            version,
        };
        if entry.void_if_draft() {
            sqlx::query(
                "UPDATE grade_entries SET status = 'voided', version = ?, updated_at = ? \
                 WHERE id = ? AND status = 'draft' AND version = ?",
            )
            .bind(entry.version.get() as i64)
            .bind(occurred_at)
            .bind(&id)
            .bind(version_raw)
            .execute(&mut *tx)
            .await?;
        }
    }

    tx.commit().await?;
    Ok(())
}

fn on_event<'a>(pool: &'a SqlitePool, event: &'a IntegrationEvent) -> DispatchFuture<'a> {
    Box::pin(async move {
        if let IntegrationEvent::StudentWithdrawn {
            student_no,
            occurred_at,
            ..
        } = event
        {
            void_drafts_for_student(pool, student_no, occurred_at).await?;
        }
        Ok(())
    })
}

inventory::submit!(OutboxSubscription {
    name: "grading::evt_void_draft_grades",
    handler: on_event,
});
