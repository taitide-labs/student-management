use std::collections::HashMap;

use rust_decimal::Decimal;
use sqlx::{QueryBuilder, Row, Sqlite};
use student_management_contracts::{GradeAmendmentRecord, OperatorPrincipal, ProblemDetails, Role};
use student_management_infra::{AppState, InfraError, ensure_role, infra_error_to_problem};

use crate::grading_repo::resolve_student_name;

pub async fn execute(
    state: &AppState,
    operator: &OperatorPrincipal,
    grade_id: Option<&str>,
) -> Result<Vec<GradeAmendmentRecord>, ProblemDetails> {
    ensure_role(operator, &[Role::Admin, Role::Teacher])?;

    let mut qb: QueryBuilder<Sqlite> = QueryBuilder::new(
        "SELECT ga.id, ga.grade_id, c.name AS course_name, ge.student_no, \
         ga.previous_score, ga.new_score, ga.reason, ga.operator_name, ga.occurred_at \
         FROM grade_amendments ga \
         INNER JOIN grade_entries ge ON ge.id = ga.grade_id \
         INNER JOIN teaching_assignments ta ON ta.id = ge.teaching_id \
         INNER JOIN courses c ON c.id = ta.course_id \
         WHERE 1 = 1",
    );

    if operator.role == Role::Teacher {
        qb.push(" AND ta.teacher_account_id = ")
            .push_bind(operator.account_id.clone());
    }
    if let Some(filter) = grade_id.map(str::trim).filter(|val| !val.is_empty()) {
        qb.push(" AND ga.grade_id = ").push_bind(filter.to_owned());
    }
    qb.push(" ORDER BY ga.occurred_at DESC, ga.id DESC");

    let rows = qb
        .build()
        .fetch_all(&state.pool)
        .await
        .map_err(InfraError::Database)
        .map_err(|err| infra_error_to_problem(&err))?;

    let mut names: HashMap<String, String> = HashMap::new();
    let mut records = Vec::with_capacity(rows.len());

    for row in rows {
        let student_no: String = row
            .try_get("student_no")
            .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?;
        let student_name = if let Some(cached) = names.get(&student_no) {
            cached.clone()
        } else {
            let resolved = resolve_student_name(state, &student_no).await;
            names.insert(student_no.clone(), resolved.clone());
            resolved
        };
        let prev_raw: String = row
            .try_get("previous_score")
            .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?;
        let new_raw: String = row
            .try_get("new_score")
            .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?;

        let previous_score = prev_raw
            .parse::<Decimal>()
            .map_err(|_| ProblemDetails::bad_request("invalid_score", "原成绩分值异常"))?;
        let new_score = new_raw
            .parse::<Decimal>()
            .map_err(|_| ProblemDetails::bad_request("invalid_score", "新成绩分值异常"))?;

        records.push(GradeAmendmentRecord {
            id: row
                .try_get("id")
                .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?,
            grade_id: row
                .try_get("grade_id")
                .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?,
            course_name: row
                .try_get("course_name")
                .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?,
            student_no,
            student_name,
            previous_score,
            new_score,
            reason: row
                .try_get("reason")
                .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?,
            operator_name: row
                .try_get("operator_name")
                .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?,
            occurred_at: row
                .try_get("occurred_at")
                .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?,
        });
    }

    Ok(records)
}
