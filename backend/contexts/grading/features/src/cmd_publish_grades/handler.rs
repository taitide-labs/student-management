use std::collections::HashMap;

use rust_decimal::Decimal;
use sqlx::{Row, SqliteConnection};
use student_management_contracts::{
    GradeEntrySummary, OperatorPrincipal, ProblemDetails, PublishGradesRequest, Role,
};
use student_management_grading_domain::GradeEntry;
use student_management_infra::{
    AppState, IdempotencyOutcome, InfraError, check_idempotency, ensure_role, hash_payload,
    infra_error_to_problem, save_idempotency,
};
use student_management_primitives::{PositiveVersion, Score, StudentNo};

use super::{command::PublishGradesCommand, workflow};
use crate::grading_repo::{grade_summary_from_row, parse_grade_status, resolve_student_name};

async fn prefetch_student_names(
    state: &AppState,
    teaching_id: &str,
) -> Result<HashMap<String, String>, ProblemDetails> {
    let rows = sqlx::query("SELECT DISTINCT student_no FROM grade_entries WHERE teaching_id = ?")
        .bind(teaching_id)
        .fetch_all(&state.pool)
        .await
        .map_err(InfraError::Database)
        .map_err(|err| infra_error_to_problem(&err))?;

    let mut names = HashMap::with_capacity(rows.len());
    for row in rows {
        let student_no: String = row
            .try_get("student_no")
            .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?;
        let name = resolve_student_name(state, &student_no).await;
        names.insert(student_no, name);
    }
    Ok(names)
}

async fn publish_drafts_in_tx(
    tx: &mut SqliteConnection,
    operator: &OperatorPrincipal,
    teaching_id: &str,
    now: &str,
) -> Result<(), ProblemDetails> {
    let teacher_id: String =
        sqlx::query("SELECT teacher_account_id FROM teaching_assignments WHERE id = ?")
            .bind(teaching_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(InfraError::Database)
            .map_err(|err| infra_error_to_problem(&err))?
            .ok_or_else(|| ProblemDetails::not_found("teaching_not_found", "教学任务不存在"))?
            .try_get("teacher_account_id")
            .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?;

    if operator.role == Role::Teacher && teacher_id != operator.account_id {
        return Err(ProblemDetails::forbidden(
            "teaching_ownership_forbidden",
            "教师仅可发布本人任教课程成绩",
        ));
    }

    let draft_rows = sqlx::query(
        "SELECT id, student_no, score, status, version FROM grade_entries \
         WHERE teaching_id = ? AND status = 'draft'",
    )
    .bind(teaching_id)
    .fetch_all(&mut *tx)
    .await
    .map_err(InfraError::Database)
    .map_err(|err| infra_error_to_problem(&err))?;

    for row in draft_rows {
        let id: String = row
            .try_get("id")
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

        let score = score_raw
            .parse::<Decimal>()
            .ok()
            .and_then(|val| Score::try_new(val).ok())
            .ok_or_else(|| ProblemDetails::bad_request("invalid_score", "成绩分值数据异常"))?;
        let version = u64::try_from(version_raw)
            .ok()
            .and_then(|val| PositiveVersion::try_new(val).ok())
            .ok_or_else(|| ProblemDetails::bad_request("invalid_version", "成绩版本号异常"))?;
        let student_no = StudentNo::try_new(&stu_raw)
            .map_err(|err| ProblemDetails::bad_request("invalid_student_no", err.to_string()))?;

        let mut entry = GradeEntry {
            id: id.clone(),
            teaching_id: teaching_id.to_owned(),
            student_no,
            score,
            status: parse_grade_status(&status_raw)?,
            version,
        };
        workflow::apply(&mut entry)?;

        sqlx::query(
            "UPDATE grade_entries SET status = 'published', version = ?, updated_at = ? \
             WHERE id = ? AND version = ?",
        )
        .bind(entry.version.get() as i64)
        .bind(now)
        .bind(&id)
        .bind(version_raw)
        .execute(&mut *tx)
        .await
        .map_err(InfraError::Database)
        .map_err(|err| infra_error_to_problem(&err))?;
    }

    Ok(())
}

pub async fn execute(
    state: &AppState,
    operator: &OperatorPrincipal,
    req: PublishGradesRequest,
) -> Result<Vec<GradeEntrySummary>, ProblemDetails> {
    ensure_role(operator, &[Role::Teacher, Role::Admin])?;
    let payload_hash = hash_payload(&req).map_err(|err| infra_error_to_problem(&err))?;
    let cmd = PublishGradesCommand::parse(&req)?;

    let mut conn = state
        .pool
        .acquire()
        .await
        .map_err(InfraError::Database)
        .map_err(|err| infra_error_to_problem(&err))?;
    if let IdempotencyOutcome::Replay(cached) = check_idempotency::<Vec<GradeEntrySummary>>(
        &mut conn,
        "grading::cmd_publish_grades",
        cmd.idempotency_key.as_str(),
        &payload_hash,
    )
    .await
    .map_err(|err| infra_error_to_problem(&err))?
    {
        return Ok(cached);
    }
    drop(conn);

    let names = prefetch_student_names(state, cmd.teaching_id.as_str()).await?;

    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(InfraError::Database)
        .map_err(|err| infra_error_to_problem(&err))?;

    if let IdempotencyOutcome::Replay(cached) = check_idempotency::<Vec<GradeEntrySummary>>(
        &mut tx,
        "grading::cmd_publish_grades",
        cmd.idempotency_key.as_str(),
        &payload_hash,
    )
    .await
    .map_err(|err| infra_error_to_problem(&err))?
    {
        return Ok(cached);
    }

    let now = state.clock.now_rfc3339();
    publish_drafts_in_tx(&mut tx, operator, cmd.teaching_id.as_str(), &now).await?;

    let rows = sqlx::query(
        "SELECT ge.id, ge.teaching_id, c.code AS course_code, c.name AS course_name, \
         ta.term, ge.student_no, ge.score, c.max_score, ge.status, ge.version \
         FROM grade_entries ge \
         INNER JOIN teaching_assignments ta ON ta.id = ge.teaching_id \
         INNER JOIN courses c ON c.id = ta.course_id \
         WHERE ge.teaching_id = ? \
         ORDER BY ge.student_no ASC",
    )
    .bind(cmd.teaching_id.as_str())
    .fetch_all(&mut *tx)
    .await
    .map_err(InfraError::Database)
    .map_err(|err| infra_error_to_problem(&err))?;

    let mut summaries = Vec::with_capacity(rows.len());
    for row in rows {
        let student_no: String = row
            .try_get("student_no")
            .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?;
        let student_name = names.get(&student_no).cloned().unwrap_or(student_no);
        summaries.push(grade_summary_from_row(&row, student_name)?);
    }

    save_idempotency(
        &mut tx,
        "grading::cmd_publish_grades",
        cmd.idempotency_key.as_str(),
        &payload_hash,
        &summaries,
        &now,
    )
    .await
    .map_err(|err| infra_error_to_problem(&err))?;

    tx.commit()
        .await
        .map_err(InfraError::Database)
        .map_err(|err| infra_error_to_problem(&err))?;
    Ok(summaries)
}
