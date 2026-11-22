use sqlx::Row;
use student_management_contracts::{HomeroomSummary, ProblemDetails};
use student_management_infra::{AppState, InfraError, infra_error_to_problem};

pub async fn execute(state: &AppState) -> Result<Vec<HomeroomSummary>, ProblemDetails> {
    let rows = sqlx::query(
        "SELECT h.id, h.name, h.grade_year, h.capacity, h.version, \
         (SELECT COUNT(*) FROM students s WHERE s.homeroom_id = h.id AND s.status = 'active') AS active_count \
         FROM homerooms h ORDER BY h.grade_year DESC, h.name ASC",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(InfraError::Database)
    .map_err(|err| infra_error_to_problem(&err))?;

    let mut items = Vec::with_capacity(rows.len());
    for row in rows {
        let grade_year_raw: i64 = row
            .try_get("grade_year")
            .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?;
        let capacity_raw: Option<i64> = row
            .try_get("capacity")
            .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?;
        let active_raw: i64 = row
            .try_get("active_count")
            .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?;
        let version_raw: i64 = row
            .try_get("version")
            .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?;

        items.push(HomeroomSummary {
            id: row
                .try_get("id")
                .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?,
            name: row
                .try_get("name")
                .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?,
            grade_year: u16::try_from(grade_year_raw).unwrap_or(0),
            capacity: capacity_raw.and_then(|val| u32::try_from(val).ok()),
            active_count: u32::try_from(active_raw).unwrap_or(0),
            version: u64::try_from(version_raw).unwrap_or(1),
        });
    }

    Ok(items)
}
