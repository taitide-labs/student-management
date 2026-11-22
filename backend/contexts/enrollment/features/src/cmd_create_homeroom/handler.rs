use std::num::NonZeroU32;

use student_management_contracts::{
    CreateHomeroomRequest, HomeroomSummary, OperatorPrincipal, ProblemDetails, Role,
};
use student_management_infra::{
    AppState, IdempotencyOutcome, InfraError, check_idempotency, ensure_role, hash_payload,
    infra_error_to_problem, save_idempotency,
};

use super::{command::CreateHomeroomCommand, workflow};

pub async fn execute(
    state: &AppState,
    operator: &OperatorPrincipal,
    req: CreateHomeroomRequest,
) -> Result<HomeroomSummary, ProblemDetails> {
    ensure_role(operator, &[Role::Admin])?;
    let payload_hash = hash_payload(&req).map_err(|err| infra_error_to_problem(&err))?;
    let cmd = CreateHomeroomCommand::try_from(req)?;

    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(InfraError::Database)
        .map_err(|err| infra_error_to_problem(&err))?;

    if let IdempotencyOutcome::Replay(cached) = check_idempotency::<HomeroomSummary>(
        &mut tx,
        "enrollment::cmd_create_homeroom",
        cmd.idempotency_key.as_str(),
        &payload_hash,
    )
    .await
    .map_err(|err| infra_error_to_problem(&err))?
    {
        return Ok(cached);
    }

    let homeroom = workflow::prepare(state.ids.next_id("hr"), &cmd);
    let now = state.clock.now_rfc3339();
    let capacity_db = homeroom.capacity.map(|cap| i64::from(cap.get()));

    if let Err(err) = sqlx::query(
        "INSERT INTO homerooms (id, name, grade_year, capacity, version, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&homeroom.id)
    .bind(homeroom.name.as_str())
    .bind(i64::from(homeroom.grade_year.get()))
    .bind(capacity_db)
    .bind(homeroom.version.get() as i64)
    .bind(&now)
    .bind(&now)
    .execute(&mut *tx)
    .await
    {
        return Err(map_insert_error(err));
    }

    let summary = HomeroomSummary {
        id: homeroom.id,
        name: homeroom.name.as_str().to_owned(),
        grade_year: homeroom.grade_year.get(),
        capacity: homeroom.capacity.map(NonZeroU32::get),
        active_count: 0,
        version: homeroom.version.get(),
    };

    save_idempotency(
        &mut tx,
        "enrollment::cmd_create_homeroom",
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

fn map_insert_error(err: sqlx::Error) -> ProblemDetails {
    if let sqlx::Error::Database(db_err) = &err
        && db_err.is_unique_violation()
    {
        return ProblemDetails::conflict("homeroom_duplicate", "同年级的行政班名称已存在");
    }
    infra_error_to_problem(&InfraError::Database(err))
}
