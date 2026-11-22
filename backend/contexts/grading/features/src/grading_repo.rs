use rust_decimal::Decimal;
use sqlx::Row;
use student_management_contracts::{GradeEntrySummary, GradeStatus, PortError, ProblemDetails};
use student_management_infra::{AppState, InfraError, infra_error_to_problem};

#[must_use]
pub fn port_error_to_problem(err: &PortError) -> ProblemDetails {
    match err {
        PortError::Forbidden(msg) => ProblemDetails::forbidden("port_forbidden", msg.clone()),
        PortError::NotFound(msg) => ProblemDetails::not_found("port_not_found", msg.clone()),
        PortError::Conflict(msg) => ProblemDetails::conflict("port_conflict", msg.clone()),
    }
}

pub fn parse_grade_status(raw: &str) -> Result<GradeStatus, ProblemDetails> {
    match raw {
        "draft" => Ok(GradeStatus::Draft),
        "published" => Ok(GradeStatus::Published),
        "voided" => Ok(GradeStatus::Voided),
        _ => Err(ProblemDetails::bad_request(
            "invalid_grade_status",
            format!("未知成绩状态：{raw}"),
        )),
    }
}

pub async fn resolve_student_name(state: &AppState, student_no: &str) -> String {
    match state.enrollment_query.as_ref() {
        Some(port) => match port.check_student_active(student_no).await {
            Ok(snap) => snap.full_name,
            Err(_) => student_no.to_owned(),
        },
        None => student_no.to_owned(),
    }
}

pub fn grade_summary_from_row(
    row: &sqlx::sqlite::SqliteRow,
    student_name: String,
) -> Result<GradeEntrySummary, ProblemDetails> {
    let score_raw: String = row
        .try_get("score")
        .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?;
    let max_raw: String = row
        .try_get("max_score")
        .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?;
    let status_raw: String = row
        .try_get("status")
        .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?;
    let version_raw: i64 = row
        .try_get("version")
        .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?;

    let score = score_raw
        .parse::<Decimal>()
        .map_err(|_| ProblemDetails::bad_request("invalid_score", "成绩分值数据异常"))?;
    let max_score = max_raw
        .parse::<Decimal>()
        .map_err(|_| ProblemDetails::bad_request("invalid_score", "课程满分数据异常"))?;

    Ok(GradeEntrySummary {
        id: row
            .try_get("id")
            .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?,
        teaching_id: row
            .try_get("teaching_id")
            .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?,
        course_code: row
            .try_get("course_code")
            .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?,
        course_name: row
            .try_get("course_name")
            .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?,
        term: row
            .try_get("term")
            .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?,
        student_no: row
            .try_get("student_no")
            .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?,
        student_name,
        score,
        max_score,
        status: parse_grade_status(&status_raw)?,
        version: u64::try_from(version_raw).unwrap_or(1),
    })
}
