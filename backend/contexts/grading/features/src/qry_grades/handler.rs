use std::collections::HashMap;

use sqlx::{QueryBuilder, Row, Sqlite};
use student_management_contracts::{GradeEntrySummary, OperatorPrincipal, ProblemDetails, Role};
use student_management_infra::{AppState, InfraError, ensure_role, infra_error_to_problem};

use crate::grading_repo::{grade_summary_from_row, resolve_student_name};

pub async fn execute(
    state: &AppState,
    operator: &OperatorPrincipal,
    student_no: Option<&str>,
) -> Result<Vec<GradeEntrySummary>, ProblemDetails> {
    ensure_role(operator, &[Role::Admin, Role::Teacher, Role::Student])?;

    let mut qb: QueryBuilder<Sqlite> = QueryBuilder::new(
        "SELECT ge.id, ge.teaching_id, c.code AS course_code, c.name AS course_name, \
         ta.term, ge.student_no, ge.score, c.max_score, ge.status, ge.version \
         FROM grade_entries ge \
         INNER JOIN teaching_assignments ta ON ta.id = ge.teaching_id \
         INNER JOIN courses c ON c.id = ta.course_id \
         WHERE 1 = 1",
    );

    match operator.role {
        Role::Admin => {}
        Role::Teacher => {
            qb.push(" AND ta.teacher_account_id = ")
                .push_bind(operator.account_id.clone());
        }
        Role::Student => {
            let bound = operator.bound_student_no.as_deref().ok_or_else(|| {
                ProblemDetails::forbidden("student_profile_unbound", "当前学生账号未绑定学籍")
            })?;
            if let Some(requested) = student_no.map(str::trim).filter(|val| !val.is_empty())
                && requested != bound
            {
                return Err(ProblemDetails::forbidden(
                    "student_scope_forbidden",
                    "学生仅可查询本人成绩",
                ));
            }
            qb.push(" AND ge.student_no = ")
                .push_bind(bound.to_owned())
                .push(" AND ge.status = 'published'");
        }
    }

    if let Some(filter) = student_no.map(str::trim).filter(|val| !val.is_empty())
        && operator.role != Role::Student
    {
        qb.push(" AND ge.student_no = ")
            .push_bind(filter.to_owned());
    }

    qb.push(" ORDER BY ta.term DESC, c.code ASC, ge.student_no ASC");

    let rows = qb
        .build()
        .fetch_all(&state.pool)
        .await
        .map_err(InfraError::Database)
        .map_err(|err| infra_error_to_problem(&err))?;

    let mut names: HashMap<String, String> = HashMap::new();
    let mut items = Vec::with_capacity(rows.len());
    for row in rows {
        let stu_no: String = row
            .try_get("student_no")
            .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?;
        let student_name = if let Some(cached) = names.get(&stu_no) {
            cached.clone()
        } else {
            let resolved = resolve_student_name(state, &stu_no).await;
            names.insert(stu_no, resolved.clone());
            resolved
        };
        items.push(grade_summary_from_row(&row, student_name)?);
    }

    Ok(items)
}
