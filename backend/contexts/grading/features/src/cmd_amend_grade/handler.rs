use rust_decimal::Decimal;
use sqlx::{Row, SqliteConnection};
use student_management_contracts::{
    AmendGradeRequest, GradeEntrySummary, OperatorPrincipal, ProblemDetails, Role,
};
use student_management_grading_domain::GradeEntry;
use student_management_infra::{
    AppState, IdempotencyOutcome, InfraError, check_idempotency, ensure_role, hash_payload,
    infra_error_to_problem, save_idempotency,
};
use student_management_primitives::{MaxScore, PositiveVersion, Score, StudentNo};

use super::{command::AmendGradeCommand, workflow};
use crate::grading_repo::{build_grade_summary, parse_grade_status, resolve_student_name};

async fn lookup_student_name_for_grade(state: &AppState, grade_id: &str) -> String {
    let maybe_no: Option<String> = sqlx::query("SELECT student_no FROM grade_entries WHERE id = ?")
        .bind(grade_id)
        .fetch_optional(&state.pool)
        .await
        .ok()
        .flatten()
        .and_then(|row| row.try_get("student_no").ok());
    match maybe_no {
        Some(no) => resolve_student_name(state, &no).await,
        None => String::new(),
    }
}

async fn load_grade_and_max(
    tx: &mut SqliteConnection,
    grade_id: &str,
) -> Result<(GradeEntry, MaxScore), ProblemDetails> {
    let row = sqlx::query(
        "SELECT ge.id, ge.teaching_id, ge.student_no, ge.score, ge.status, ge.version, c.max_score \
         FROM grade_entries ge \
         INNER JOIN teaching_assignments ta ON ta.id = ge.teaching_id \
         INNER JOIN courses c ON c.id = ta.course_id \
         WHERE ge.id = ?",
    )
    .bind(grade_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(InfraError::Database)
    .map_err(|err| infra_error_to_problem(&err))?
    .ok_or_else(|| ProblemDetails::not_found("grade_not_found", "成绩记录不存在"))?;

    let teaching_id: String = row
        .try_get("teaching_id")
        .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?;
    let stu_raw: String = row
        .try_get("student_no")
        .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?;
    let score_raw: String = row
        .try_get("score")
        .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?;
    let status_raw: String = row
        .try_get("status")
        .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?;
    let version_raw: i64 = row
        .try_get("version")
        .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?;
    let max_raw: String = row
        .try_get("max_score")
        .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?;

    let student_no = StudentNo::try_new(&stu_raw)
        .map_err(|err| ProblemDetails::bad_request("invalid_student_no", err.to_string()))?;
    let score = score_raw
        .parse::<Decimal>()
        .ok()
        .and_then(|val| Score::try_new(val).ok())
        .ok_or_else(|| ProblemDetails::bad_request("invalid_score", "成绩分值数据异常"))?;
    let max_score = max_raw
        .parse::<Decimal>()
        .ok()
        .and_then(|val| MaxScore::try_new(val).ok())
        .ok_or_else(|| ProblemDetails::bad_request("invalid_max_score", "课程满分数据异常"))?;
    let version = u64::try_from(version_raw)
        .ok()
        .and_then(|val| PositiveVersion::try_new(val).ok())
        .ok_or_else(|| ProblemDetails::bad_request("invalid_version", "成绩版本号异常"))?;

    Ok((
        GradeEntry {
            id: grade_id.to_owned(),
            teaching_id,
            student_no,
            score,
            status: parse_grade_status(&status_raw)?,
            version,
        },
        max_score,
    ))
}

async fn persist_amendment(
    tx: &mut SqliteConnection,
    state: &AppState,
    operator: &OperatorPrincipal,
    cmd: &AmendGradeCommand,
    now: &str,
) -> Result<(), ProblemDetails> {
    let (mut entry, max_score) = load_grade_and_max(tx, cmd.grade_id.as_str()).await?;
    let draft = workflow::apply(&mut entry, operator.role == Role::Admin, cmd, max_score)?;

    let res = sqlx::query(
        "UPDATE grade_entries SET score = ?, version = ?, updated_at = ? WHERE id = ? AND version = ?",
    )
    .bind(entry.score.get().to_string())
    .bind(entry.version.get() as i64)
    .bind(now)
    .bind(&entry.id)
    .bind(cmd.expected_version.get() as i64)
    .execute(&mut *tx)
    .await
    .map_err(InfraError::Database)
    .map_err(|err| infra_error_to_problem(&err))?;

    if res.rows_affected() == 0 {
        return Err(ProblemDetails::conflict(
            "version_conflict",
            "成绩记录版本号已变更",
        ));
    }

    sqlx::query(
        "INSERT INTO grade_amendments \
         (id, grade_id, previous_score, new_score, reason, operator_name, occurred_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(state.ids.next_id("amd"))
    .bind(&entry.id)
    .bind(draft.previous_score.to_string())
    .bind(draft.new_score.to_string())
    .bind(draft.reason)
    .bind(&operator.display_name)
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(InfraError::Database)
    .map_err(|err| infra_error_to_problem(&err))?;

    Ok(())
}

pub async fn execute(
    state: &AppState,
    operator: &OperatorPrincipal,
    grade_id: &str,
    req: AmendGradeRequest,
) -> Result<GradeEntrySummary, ProblemDetails> {
    ensure_role(operator, &[Role::Admin])?;
    let payload_hash =
        hash_payload(&(grade_id, &req)).map_err(|err| infra_error_to_problem(&err))?;
    let cmd = AmendGradeCommand::parse(grade_id, &req)?;

    let mut conn = state
        .pool
        .acquire()
        .await
        .map_err(InfraError::Database)
        .map_err(|err| infra_error_to_problem(&err))?;
    if let IdempotencyOutcome::Replay(cached) = check_idempotency::<GradeEntrySummary>(
        &mut conn,
        "grading::cmd_amend_grade",
        cmd.idempotency_key.as_str(),
        &payload_hash,
    )
    .await
    .map_err(|err| infra_error_to_problem(&err))?
    {
        return Ok(cached);
    }
    drop(conn);

    let student_name = lookup_student_name_for_grade(state, cmd.grade_id.as_str()).await;

    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(InfraError::Database)
        .map_err(|err| infra_error_to_problem(&err))?;

    if let IdempotencyOutcome::Replay(cached) = check_idempotency::<GradeEntrySummary>(
        &mut tx,
        "grading::cmd_amend_grade",
        cmd.idempotency_key.as_str(),
        &payload_hash,
    )
    .await
    .map_err(|err| infra_error_to_problem(&err))?
    {
        return Ok(cached);
    }

    let now = state.clock.now_rfc3339();
    persist_amendment(&mut tx, state, operator, &cmd, &now).await?;
    let summary = build_grade_summary(&mut tx, cmd.grade_id.as_str(), student_name).await?;

    save_idempotency(
        &mut tx,
        "grading::cmd_amend_grade",
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
