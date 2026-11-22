use std::collections::HashMap;

use sqlx::Row;
use student_management_contracts::{OperatorPrincipal, ProblemDetails, Role, TeachingSummary};
use student_management_infra::{AppState, InfraError, ensure_role, infra_error_to_problem};

pub async fn execute(
    state: &AppState,
    operator: &OperatorPrincipal,
) -> Result<Vec<TeachingSummary>, ProblemDetails> {
    ensure_role(operator, &[Role::Admin, Role::Teacher])?;

    let rows = match operator.role {
        Role::Teacher => {
            sqlx::query(
                "SELECT ta.id, ta.course_id, c.code AS course_code, c.name AS course_name, \
                 ta.homeroom_id, ta.term, ta.teacher_account_id, ta.teacher_name, ta.version \
                 FROM teaching_assignments ta \
                 INNER JOIN courses c ON c.id = ta.course_id \
                 WHERE ta.teacher_account_id = ? \
                 ORDER BY ta.term DESC, c.code ASC",
            )
            .bind(&operator.account_id)
            .fetch_all(&state.pool)
            .await
        }
        Role::Admin | Role::Student => {
            sqlx::query(
                "SELECT ta.id, ta.course_id, c.code AS course_code, c.name AS course_name, \
                 ta.homeroom_id, ta.term, ta.teacher_account_id, ta.teacher_name, ta.version \
                 FROM teaching_assignments ta \
                 INNER JOIN courses c ON c.id = ta.course_id \
                 ORDER BY ta.term DESC, c.code ASC",
            )
            .fetch_all(&state.pool)
            .await
        }
    }
    .map_err(InfraError::Database)
    .map_err(|err| infra_error_to_problem(&err))?;

    let mut homeroom_cache: HashMap<String, String> = HashMap::new();
    let mut items = Vec::with_capacity(rows.len());

    for row in rows {
        let homeroom_id: String = row
            .try_get("homeroom_id")
            .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?;
        let homeroom_name = if let Some(cached) = homeroom_cache.get(&homeroom_id) {
            cached.clone()
        } else {
            let resolved = match state.enrollment_query.as_ref() {
                Some(port) => match port.verify_homeroom_exists(&homeroom_id).await {
                    Ok(snap) => snap.name,
                    Err(_) => homeroom_id.clone(),
                },
                None => homeroom_id.clone(),
            };
            homeroom_cache.insert(homeroom_id.clone(), resolved.clone());
            resolved
        };
        let version_raw: i64 = row
            .try_get("version")
            .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?;

        items.push(TeachingSummary {
            id: row
                .try_get("id")
                .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?,
            course_id: row
                .try_get("course_id")
                .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?,
            course_code: row
                .try_get("course_code")
                .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?,
            course_name: row
                .try_get("course_name")
                .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?,
            homeroom_id,
            homeroom_name,
            term: row
                .try_get("term")
                .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?,
            teacher_account_id: row
                .try_get("teacher_account_id")
                .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?,
            teacher_name: row
                .try_get("teacher_name")
                .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?,
            version: u64::try_from(version_raw).unwrap_or(1),
        });
    }

    Ok(items)
}
