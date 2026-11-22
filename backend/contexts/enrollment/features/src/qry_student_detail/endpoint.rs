use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode, header::AUTHORIZATION},
    routing::get,
};
use student_management_contracts::{ProblemDetails, StudentDetailPayload};
use student_management_infra::{
    AppState, HttpMethod, SliceDescriptor, SliceStatus, authenticate_bearer,
};

use super::handler;

#[utoipa::path(
    get,
    path = "/api/v1/students/{student_no}",
    tag = "enrollment",
    params(("student_no" = String, Path, description = "学号")),
    responses(
        (status = 200, description = "学生档案与异动记录", body = StudentDetailPayload),
        (status = 404, description = "学生不存在", body = ProblemDetails)
    )
)]
pub async fn student_detail(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(student_no): Path<String>,
) -> Result<(StatusCode, Json<StudentDetailPayload>), (StatusCode, Json<ProblemDetails>)> {
    let auth_header = headers.get(AUTHORIZATION).and_then(|v| v.to_str().ok());
    let operator = authenticate_bearer(&state.pool, auth_header)
        .await
        .map_err(problem_response)?;
    let detail = handler::execute(&state, &operator, &student_no)
        .await
        .map_err(problem_response)?;
    Ok((StatusCode::OK, Json(detail)))
}

fn problem_response(problem: ProblemDetails) -> (StatusCode, Json<ProblemDetails>) {
    let status = StatusCode::from_u16(problem.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    (status, Json(problem))
}

fn route() -> Router<AppState> {
    Router::new().route("/api/v1/students/{student_no}", get(student_detail))
}

inventory::submit!(SliceDescriptor {
    context: "enrollment",
    slice: "qry_student_detail",
    method: HttpMethod::Get,
    path: "/api/v1/students/{student_no}",
    status: SliceStatus::Active,
    route,
});
