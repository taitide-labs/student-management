use sqlx::{Row, SqliteConnection};
use student_management_contracts::{
    AssignTeachingRequest, HomeroomSnapshot, OperatorPrincipal, ProblemDetails, Role,
    TeacherIdentity, TeachingSummary,
};
use student_management_infra::{
    AppState, IdempotencyOutcome, InfraError, check_idempotency, ensure_role, hash_payload,
    infra_error_to_problem, save_idempotency,
};

use super::command::AssignTeachingCommand;
use crate::grading_repo::port_error_to_problem;

async fn verify_targets(
    state: &AppState,
    cmd: &AssignTeachingCommand,
) -> Result<(TeacherIdentity, HomeroomSnapshot), ProblemDetails> {
    let identity_port = state
        .identity_lookup
        .as_ref()
        .ok_or_else(|| ProblemDetails::bad_request("missing_identity_port", "缺少身份查询端口"))?;
    let teacher = identity_port
        .verify_teacher(cmd.teacher_account_id.as_str())
        .await
        .map_err(|err| port_error_to_problem(&err))?;

    let enrollment_port = state.enrollment_query.as_ref().ok_or_else(|| {
        ProblemDetails::bad_request("missing_enrollment_port", "缺少学籍查询端口")
    })?;
    let homeroom = enrollment_port
        .verify_homeroom_exists(cmd.homeroom_id.as_str())
        .await
        .map_err(|err| port_error_to_problem(&err))?;
    Ok((teacher, homeroom))
}

async fn persist_teaching(
    tx: &mut SqliteConnection,
    state: &AppState,
    cmd: &AssignTeachingCommand,
    teacher: TeacherIdentity,
    homeroom: HomeroomSnapshot,
    now: &str,
) -> Result<TeachingSummary, ProblemDetails> {
    let course_row = sqlx::query("SELECT code, name FROM courses WHERE id = ?")
        .bind(cmd.course_id.as_str())
        .fetch_optional(&mut *tx)
        .await
        .map_err(InfraError::Database)
        .map_err(|err| infra_error_to_problem(&err))?
        .ok_or_else(|| ProblemDetails::not_found("course_not_found", "课程不存在"))?;
    let course_code: String = course_row
        .try_get("code")
        .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?;
    let course_name: String = course_row
        .try_get("name")
        .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?;

    let id = state.ids.next_id("tch");
    let insert_res = sqlx::query(
        "INSERT INTO teaching_assignments \
         (id, course_id, homeroom_id, term, teacher_account_id, teacher_name, version, created_at) \
         VALUES (?, ?, ?, ?, ?, ?, 1, ?)",
    )
    .bind(&id)
    .bind(cmd.course_id.as_str())
    .bind(cmd.homeroom_id.as_str())
    .bind(cmd.term.as_str())
    .bind(&teacher.account_id)
    .bind(&teacher.display_name)
    .bind(now)
    .execute(&mut *tx)
    .await;

    if let Err(err) = insert_res {
        if matches!(&err, sqlx::Error::Database(db_err) if db_err.is_unique_violation()) {
            return Err(ProblemDetails::conflict(
                "teaching_assignment_conflict",
                "同一班级、学期与课程已分派教学任务",
            ));
        }
        return Err(infra_error_to_problem(&InfraError::Database(err)));
    }

    Ok(TeachingSummary {
        id,
        course_id: cmd.course_id.as_str().to_owned(),
        course_code,
        course_name,
        homeroom_id: homeroom.id,
        homeroom_name: homeroom.name,
        term: cmd.term.as_str().to_owned(),
        teacher_account_id: teacher.account_id,
        teacher_name: teacher.display_name,
        version: 1,
    })
}

pub async fn execute(
    state: &AppState,
    operator: &OperatorPrincipal,
    req: AssignTeachingRequest,
) -> Result<TeachingSummary, ProblemDetails> {
    ensure_role(operator, &[Role::Admin])?;
    let payload_hash = hash_payload(&req).map_err(|err| infra_error_to_problem(&err))?;
    let cmd = AssignTeachingCommand::parse(&req)?;

    let mut conn = state
        .pool
        .acquire()
        .await
        .map_err(InfraError::Database)
        .map_err(|err| infra_error_to_problem(&err))?;
    if let IdempotencyOutcome::Replay(cached) = check_idempotency::<TeachingSummary>(
        &mut conn,
        "grading::cmd_assign_teaching",
        cmd.idempotency_key.as_str(),
        &payload_hash,
    )
    .await
    .map_err(|err| infra_error_to_problem(&err))?
    {
        return Ok(cached);
    }
    drop(conn);

    let (teacher, homeroom) = verify_targets(state, &cmd).await?;

    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(InfraError::Database)
        .map_err(|err| infra_error_to_problem(&err))?;

    if let IdempotencyOutcome::Replay(cached) = check_idempotency::<TeachingSummary>(
        &mut tx,
        "grading::cmd_assign_teaching",
        cmd.idempotency_key.as_str(),
        &payload_hash,
    )
    .await
    .map_err(|err| infra_error_to_problem(&err))?
    {
        return Ok(cached);
    }

    let now = state.clock.now_rfc3339();
    let summary = persist_teaching(&mut tx, state, &cmd, teacher, homeroom, &now).await?;

    save_idempotency(
        &mut tx,
        "grading::cmd_assign_teaching",
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
