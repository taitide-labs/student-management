use rust_decimal::Decimal;
use sqlx::{Row, SqliteConnection};
use student_management_contracts::{
    GradeEntrySummary, OperatorPrincipal, ProblemDetails, Role, SaveDraftGradeRequest,
    StudentEnrollmentSnapshot,
};
use student_management_grading_domain::GradeEntry;
use student_management_infra::{
    AppState, IdempotencyOutcome, InfraError, check_idempotency, ensure_role, hash_payload,
    infra_error_to_problem, save_idempotency,
};
use student_management_primitives::{MaxScore, PositiveVersion, Score};

use super::{command::SaveDraftGradeCommand, workflow};
use crate::grading_repo::{build_grade_summary, parse_grade_status, port_error_to_problem};

async fn load_teaching_max_score(
    tx: &mut SqliteConnection,
    operator: &OperatorPrincipal,
    teaching_id: &str,
    student_homeroom_id: &str,
) -> Result<MaxScore, ProblemDetails> {
    let row = sqlx::query(
        "SELECT ta.homeroom_id, ta.teacher_account_id, c.max_score \
         FROM teaching_assignments ta \
         INNER JOIN courses c ON c.id = ta.course_id \
         WHERE ta.id = ?",
    )
    .bind(teaching_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(InfraError::Database)
    .map_err(|err| infra_error_to_problem(&err))?
    .ok_or_else(|| ProblemDetails::not_found("teaching_not_found", "教学任务不存在"))?;

    let homeroom_id: String = row
        .try_get("homeroom_id")
        .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?;
    let teacher_id: String = row
        .try_get("teacher_account_id")
        .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?;
    let max_raw: String = row
        .try_get("max_score")
        .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?;

    if operator.role == Role::Teacher && teacher_id != operator.account_id {
        return Err(ProblemDetails::forbidden(
            "teaching_ownership_forbidden",
            "教师仅可录入本人任教课程成绩",
        ));
    }
    if homeroom_id != student_homeroom_id {
        return Err(ProblemDetails::conflict(
            "student_homeroom_mismatch",
            "该学生不属于此教学任务对应的行政班",
        ));
    }

    let max_dec = max_raw
        .parse::<Decimal>()
        .map_err(|_| ProblemDetails::bad_request("invalid_score", "课程满分数据异常"))?;
    MaxScore::try_new(max_dec)
        .map_err(|err| ProblemDetails::bad_request("invalid_max_score", err.to_string()))
}

async fn upsert_draft_entry(
    tx: &mut SqliteConnection,
    state: &AppState,
    cmd: &SaveDraftGradeCommand,
    max_score: MaxScore,
    now: &str,
) -> Result<String, ProblemDetails> {
    let existing = sqlx::query(
        "SELECT id, score, status, version FROM grade_entries WHERE teaching_id = ? AND student_no = ?",
    )
    .bind(cmd.teaching_id.as_str())
    .bind(cmd.student_no.as_str())
    .fetch_optional(&mut *tx)
    .await
    .map_err(InfraError::Database)
    .map_err(|err| infra_error_to_problem(&err))?;

    let Some(row) = existing else {
        let entry = workflow::create_draft(state.ids.next_id("grd"), cmd, max_score)?;
        sqlx::query(
            "INSERT INTO grade_entries \
             (id, teaching_id, student_no, score, status, version, updated_at) \
             VALUES (?, ?, ?, ?, 'draft', 1, ?)",
        )
        .bind(&entry.id)
        .bind(&entry.teaching_id)
        .bind(entry.student_no.as_str())
        .bind(entry.score.get().to_string())
        .bind(now)
        .execute(&mut *tx)
        .await
        .map_err(InfraError::Database)
        .map_err(|err| infra_error_to_problem(&err))?;
        return Ok(entry.id);
    };

    let id: String = row
        .try_get("id")
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

    let cur_score = score_raw
        .parse::<Decimal>()
        .ok()
        .and_then(|val| Score::try_new(val).ok())
        .ok_or_else(|| ProblemDetails::bad_request("invalid_score", "已有成绩分值异常"))?;
    let version = u64::try_from(version_raw)
        .ok()
        .and_then(|val| PositiveVersion::try_new(val).ok())
        .ok_or_else(|| ProblemDetails::bad_request("invalid_version", "成绩版本号异常"))?;

    let mut entry = GradeEntry {
        id: id.clone(),
        teaching_id: cmd.teaching_id.as_str().to_owned(),
        student_no: cmd.student_no.clone(),
        score: cur_score,
        status: parse_grade_status(&status_raw)?,
        version,
    };
    workflow::update_draft(&mut entry, cmd.score, max_score)?;

    let res = sqlx::query(
        "UPDATE grade_entries SET score = ?, version = ?, updated_at = ? WHERE id = ? AND version = ?",
    )
    .bind(entry.score.get().to_string())
    .bind(entry.version.get() as i64)
    .bind(now)
    .bind(&id)
    .bind(version_raw)
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
    Ok(id)
}

async fn verify_active_student(
    state: &AppState,
    student_no: &str,
) -> Result<StudentEnrollmentSnapshot, ProblemDetails> {
    let port = state.enrollment_query.as_ref().ok_or_else(|| {
        ProblemDetails::bad_request("missing_enrollment_port", "缺少学籍查询端口")
    })?;
    port.check_student_active(student_no)
        .await
        .map_err(|err| port_error_to_problem(&err))
}

pub async fn execute(
    state: &AppState,
    operator: &OperatorPrincipal,
    req: SaveDraftGradeRequest,
) -> Result<GradeEntrySummary, ProblemDetails> {
    ensure_role(operator, &[Role::Teacher, Role::Admin])?;
    let payload_hash = hash_payload(&req).map_err(|err| infra_error_to_problem(&err))?;
    let cmd = SaveDraftGradeCommand::parse(&req)?;

    let mut conn = state
        .pool
        .acquire()
        .await
        .map_err(InfraError::Database)
        .map_err(|err| infra_error_to_problem(&err))?;
    if let IdempotencyOutcome::Replay(cached) = check_idempotency::<GradeEntrySummary>(
        &mut conn,
        "grading::cmd_save_draft_grade",
        cmd.idempotency_key.as_str(),
        &payload_hash,
    )
    .await
    .map_err(|err| infra_error_to_problem(&err))?
    {
        return Ok(cached);
    }
    drop(conn);

    let student = verify_active_student(state, cmd.student_no.as_str()).await?;

    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(InfraError::Database)
        .map_err(|err| infra_error_to_problem(&err))?;

    if let IdempotencyOutcome::Replay(cached) = check_idempotency::<GradeEntrySummary>(
        &mut tx,
        "grading::cmd_save_draft_grade",
        cmd.idempotency_key.as_str(),
        &payload_hash,
    )
    .await
    .map_err(|err| infra_error_to_problem(&err))?
    {
        return Ok(cached);
    }

    let max_score = load_teaching_max_score(
        &mut tx,
        operator,
        cmd.teaching_id.as_str(),
        &student.homeroom_id,
    )
    .await?;
    let now = state.clock.now_rfc3339();
    let grade_id = upsert_draft_entry(&mut tx, state, &cmd, max_score, &now).await?;
    let summary = build_grade_summary(&mut tx, &grade_id, student.full_name).await?;

    save_idempotency(
        &mut tx,
        "grading::cmd_save_draft_grade",
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
