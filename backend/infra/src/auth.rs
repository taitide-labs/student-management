use sqlx::{Row, SqlitePool};
use student_management_contracts::{OperatorPrincipal, ProblemDetails, Role};

use crate::error::InfraError;

pub async fn authenticate_bearer(
    pool: &SqlitePool,
    auth_header: Option<&str>,
) -> Result<OperatorPrincipal, ProblemDetails> {
    let token = auth_header
        .and_then(|raw| raw.strip_prefix("Bearer "))
        .map(str::trim)
        .filter(|tok| !tok.is_empty())
        .ok_or_else(|| ProblemDetails::unauthorized("missing_token", "缺少有效 Bearer 会话令牌"))?;

    let row = sqlx::query(
        "SELECT a.account_id, a.display_name, a.role, a.bound_student_no \
         FROM sessions s \
         JOIN accounts a ON a.account_id = s.account_id \
         WHERE s.token = ? AND s.revoked_at IS NULL AND a.status = 'active'",
    )
    .bind(token)
    .fetch_optional(pool)
    .await
    .map_err(InfraError::Database)
    .map_err(|err| infra_error_to_problem(&err))?;

    let Some(row) = row else {
        return Err(ProblemDetails::unauthorized(
            "invalid_session",
            "会话令牌无效、已吊销或账号已停用",
        ));
    };

    let role_raw: String = row
        .try_get("role")
        .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?;
    let role = parse_role(&role_raw)?;

    Ok(OperatorPrincipal {
        account_id: row
            .try_get("account_id")
            .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?,
        display_name: row
            .try_get("display_name")
            .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?,
        role,
        bound_student_no: row
            .try_get("bound_student_no")
            .map_err(|err| infra_error_to_problem(&InfraError::Database(err)))?,
    })
}

pub fn ensure_role(operator: &OperatorPrincipal, allowed: &[Role]) -> Result<(), ProblemDetails> {
    if allowed.contains(&operator.role) {
        return Ok(());
    }
    Err(ProblemDetails::forbidden(
        "role_forbidden",
        format!("当前角色「{}」无权执行此操作", operator.role.label()),
    ))
}

pub fn parse_role(raw: &str) -> Result<Role, ProblemDetails> {
    match raw {
        "admin" => Ok(Role::Admin),
        "teacher" => Ok(Role::Teacher),
        "student" => Ok(Role::Student),
        _ => Err(ProblemDetails::bad_request(
            "invalid_role",
            format!("未知角色标识：{raw}"),
        )),
    }
}

#[must_use]
pub fn infra_error_to_problem(err: &InfraError) -> ProblemDetails {
    match err {
        InfraError::IdempotencyConflict => {
            ProblemDetails::conflict("idempotency_conflict", "同一幂等键被用于不同的请求内容")
        }
        InfraError::Database(_)
        | InfraError::Migrate(_)
        | InfraError::Serialization(_)
        | InfraError::PasswordHash(_)
        | InfraError::EventDispatch(_) => ProblemDetails {
            problem_type: "https://student-management.local/problems/internal".to_owned(),
            title: "服务端基础设施异常".to_owned(),
            status: 500,
            detail: err.to_string(),
            code: "internal_error".to_owned(),
        },
    }
}
