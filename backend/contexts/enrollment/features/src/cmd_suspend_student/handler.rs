use student_management_contracts::{
    OperatorPrincipal, ProblemDetails, Role, StudentSummary, TransitionStudentRequest,
};
use student_management_infra::{
    AppState, IdempotencyOutcome, InfraError, check_idempotency, ensure_role, hash_payload,
    infra_error_to_problem,
};

use super::{command::SuspendStudentCommand, workflow};
use crate::student_repo::{StatusPersistCtx, load_student, persist_status_change};

pub async fn execute(
    state: &AppState,
    operator: &OperatorPrincipal,
    student_no: &str,
    req: TransitionStudentRequest,
) -> Result<StudentSummary, ProblemDetails> {
    ensure_role(operator, &[Role::Admin])?;
    let payload_hash =
        hash_payload(&(student_no, &req)).map_err(|err| infra_error_to_problem(&err))?;
    let cmd = SuspendStudentCommand::parse(student_no, &req)?;

    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(InfraError::Database)
        .map_err(|err| infra_error_to_problem(&err))?;

    if let IdempotencyOutcome::Replay(cached) = check_idempotency::<StudentSummary>(
        &mut tx,
        "enrollment::cmd_suspend_student",
        cmd.idempotency_key.as_str(),
        &payload_hash,
    )
    .await
    .map_err(|err| infra_error_to_problem(&err))?
    {
        return Ok(cached);
    }

    let expected_ver = cmd.expected_version.get() as i64;
    let (mut student, homeroom_name) = load_student(&mut tx, cmd.student_no.as_str()).await?;
    let change = workflow::apply(&mut student, &cmd)?;
    let now = state.clock.now_rfc3339();

    let summary = persist_status_change(
        &mut tx,
        StatusPersistCtx {
            state,
            operator,
            student: &student,
            homeroom_name: &homeroom_name,
            expected_version: expected_ver,
            change,
            event: None,
            scope: "enrollment::cmd_suspend_student",
            idempotency_key: cmd.idempotency_key.as_str(),
            payload_hash: &payload_hash,
            now: &now,
        },
    )
    .await?;

    tx.commit()
        .await
        .map_err(InfraError::Database)
        .map_err(|err| infra_error_to_problem(&err))?;
    Ok(summary)
}
