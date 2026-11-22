use student_management_contracts::{
    CourseSummary, CreateCourseRequest, OperatorPrincipal, ProblemDetails, Role,
};
use student_management_infra::{
    AppState, IdempotencyOutcome, InfraError, check_idempotency, ensure_role, hash_payload,
    infra_error_to_problem, save_idempotency,
};

use super::{command::CreateCourseCommand, workflow};

pub async fn execute(
    state: &AppState,
    operator: &OperatorPrincipal,
    req: CreateCourseRequest,
) -> Result<CourseSummary, ProblemDetails> {
    ensure_role(operator, &[Role::Admin])?;
    let payload_hash = hash_payload(&req).map_err(|err| infra_error_to_problem(&err))?;
    let cmd = CreateCourseCommand::parse(&req)?;

    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(InfraError::Database)
        .map_err(|err| infra_error_to_problem(&err))?;

    if let IdempotencyOutcome::Replay(cached) = check_idempotency::<CourseSummary>(
        &mut tx,
        "grading::cmd_create_course",
        cmd.idempotency_key.as_str(),
        &payload_hash,
    )
    .await
    .map_err(|err| infra_error_to_problem(&err))?
    {
        return Ok(cached);
    }

    let course = workflow::apply(state.ids.next_id("crs"), &cmd)?;
    let now = state.clock.now_rfc3339();
    let insert_res = sqlx::query(
        "INSERT INTO courses (id, code, name, max_score, pass_score, version, created_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&course.id)
    .bind(course.code.as_str())
    .bind(course.name.as_str())
    .bind(course.max_score.get().to_string())
    .bind(course.pass_score.get().to_string())
    .bind(course.version.get() as i64)
    .bind(&now)
    .execute(&mut *tx)
    .await;

    if let Err(err) = insert_res {
        if matches!(&err, sqlx::Error::Database(db_err) if db_err.is_unique_violation()) {
            return Err(ProblemDetails::conflict(
                "course_code_conflict",
                "课程编码已存在",
            ));
        }
        return Err(infra_error_to_problem(&InfraError::Database(err)));
    }

    let summary = CourseSummary {
        id: course.id,
        code: course.code.as_str().to_owned(),
        name: course.name.as_str().to_owned(),
        max_score: course.max_score.get(),
        pass_score: course.pass_score.get(),
        version: course.version.get(),
    };

    save_idempotency(
        &mut tx,
        "grading::cmd_create_course",
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
