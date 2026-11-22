use std::num::NonZeroU32;

use sqlx::{Row, SqliteConnection};
use student_management_contracts::{
    HomeroomSummary, OperatorPrincipal, ProblemDetails, Role, UpdateHomeroomCapacityRequest,
};
use student_management_enrollment_domain::Homeroom;
use student_management_infra::{
    AppState, IdempotencyOutcome, InfraError, check_idempotency, ensure_role, hash_payload,
    infra_error_to_problem, save_idempotency,
};
use student_management_primitives::{GradeYear, HomeroomName, PositiveVersion};

use super::{command::UpdateHomeroomCapacityCommand, workflow};

pub async fn execute(
    state: &AppState,
    operator: &OperatorPrincipal,
    homeroom_id: &str,
    req: UpdateHomeroomCapacityRequest,
) -> Result<HomeroomSummary, ProblemDetails> {
    ensure_role(operator, &[Role::Admin])?;
    let payload_hash =
        hash_payload(&(homeroom_id, &req)).map_err(|err| infra_error_to_problem(&err))?;
    let cmd = UpdateHomeroomCapacityCommand::parse(homeroom_id, &req)?;

    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(InfraError::Database)
        .map_err(|err| infra_error_to_problem(&err))?;

    if let IdempotencyOutcome::Replay(cached) = check_idempotency::<HomeroomSummary>(
        &mut tx,
        "enrollment::cmd_update_homeroom_capacity",
        cmd.idempotency_key.as_str(),
        &payload_hash,
    )
    .await
    .map_err(|err| infra_error_to_problem(&err))?
    {
        return Ok(cached);
    }

    let expected_ver = cmd.expected_version.get() as i64;
    let (mut homeroom, active_count) =
        load_homeroom_with_occupancy(&mut tx, cmd.homeroom_id.as_str()).await?;
    workflow::apply(&mut homeroom, active_count, &cmd)?;

    let now = state.clock.now_rfc3339();
    let cap_db = homeroom.capacity.map(|cap| i64::from(cap.get()));
    let res = sqlx::query(
        "UPDATE homerooms SET capacity = ?, version = ?, updated_at = ? WHERE id = ? AND version = ?",
    )
    .bind(cap_db)
    .bind(homeroom.version.get() as i64)
    .bind(&now)
    .bind(&homeroom.id)
    .bind(expected_ver)
    .execute(&mut *tx)
    .await
    .map_err(InfraError::Database)
    .map_err(|err| infra_error_to_problem(&err))?;

    if res.rows_affected() == 0 {
        return Err(ProblemDetails::conflict(
            "version_conflict",
            "行政班版本号已变更",
        ));
    }

    let summary = HomeroomSummary {
        id: homeroom.id,
        name: homeroom.name.as_str().to_owned(),
        grade_year: homeroom.grade_year.get(),
        capacity: homeroom.capacity.map(NonZeroU32::get),
        active_count,
        version: homeroom.version.get(),
    };

    save_idempotency(
        &mut tx,
        "enrollment::cmd_update_homeroom_capacity",
        cmd.idempotency_key.as_str(),
        &payload_hash,
        &summary,
        &now,
    )
    .await
    .map_err(|err| infra_error_to_problem(&err))?;

    tx.commit()
        .await
        .map_err(InfraError::Database)
        .map_err(|err| infra_error_to_problem(&err))?;
    Ok(summary)
}

async fn load_homeroom_with_occupancy(
    conn: &mut SqliteConnection,
    homeroom_id: &str,
) -> Result<(Homeroom, u32), ProblemDetails> {
    let row = sqlx::query(
        "SELECT h.id, h.name, h.grade_year, h.capacity, h.version, \
         (SELECT COUNT(*) FROM students s WHERE s.homeroom_id = h.id AND s.status = 'active') AS active_count \
         FROM homerooms h WHERE h.id = ?",
    )
    .bind(homeroom_id)
    .fetch_optional(conn)
    .await
    .map_err(InfraError::Database)
    .map_err(|err| infra_error_to_problem(&err))?
    .ok_or_else(|| {
        ProblemDetails::not_found("homeroom_not_found", format!("行政班 {homeroom_id} 不存在"))
    })?;

    let grade_year_raw: i64 = row
        .try_get("grade_year")
        .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?;
    let capacity_raw: Option<i64> = row
        .try_get("capacity")
        .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?;
    let version_raw: i64 = row
        .try_get("version")
        .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?;
    let active_raw: i64 = row
        .try_get("active_count")
        .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?;
    let name_raw: String = row
        .try_get("name")
        .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?;
    let id: String = row
        .try_get("id")
        .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?;

    let grade_year = u16::try_from(grade_year_raw)
        .ok()
        .and_then(|val| GradeYear::try_new(val).ok())
        .ok_or_else(|| ProblemDetails::bad_request("invalid_grade_year", "行政班年级异常"))?;
    let name = HomeroomName::try_new(&name_raw)
        .map_err(|err| ProblemDetails::bad_request("invalid_homeroom_name", err.to_string()))?;
    let capacity = capacity_raw
        .map(|cap| {
            u32::try_from(cap)
                .ok()
                .and_then(NonZeroU32::new)
                .ok_or_else(|| ProblemDetails::bad_request("invalid_capacity", "容量数据异常"))
        })
        .transpose()?;
    let version = u64::try_from(version_raw)
        .ok()
        .and_then(|val| PositiveVersion::try_new(val).ok())
        .ok_or_else(|| ProblemDetails::bad_request("invalid_version", "行政班版本号异常"))?;
    let active_count = u32::try_from(active_raw)
        .map_err(|_| ProblemDetails::bad_request("invalid_active_count", "在籍人数计数异常"))?;

    Ok((
        Homeroom {
            id,
            name,
            grade_year,
            capacity,
            version,
        },
        active_count,
    ))
}
