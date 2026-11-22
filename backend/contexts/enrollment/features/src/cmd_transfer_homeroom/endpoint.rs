use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode, header::AUTHORIZATION},
    routing::post,
};
use student_management_contracts::{ProblemDetails, StudentSummary, TransferHomeroomRequest};
use student_management_infra::{
    AppState, HttpMethod, SliceDescriptor, SliceStatus, authenticate_bearer,
};

use super::handler;

#[utoipa::path(
    post,
    path = "/api/v1/students/{student_no}/transfer",
    tag = "enrollment",
    params(("student_no" = String, Path, description = "学号")),
    request_body = TransferHomeroomRequest,
    responses(
        (status = 200, description = "转班成功", body = StudentSummary),
        (status = 409, description = "状态锁定、容量已满或版本冲突", body = ProblemDetails)
    )
)]
pub async fn transfer_homeroom(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(student_no): Path<String>,
    Json(req): Json<TransferHomeroomRequest>,
) -> Result<(StatusCode, Json<StudentSummary>), (StatusCode, Json<ProblemDetails>)> {
    let auth_header = headers.get(AUTHORIZATION).and_then(|v| v.to_str().ok());
    let operator = authenticate_bearer(&state.pool, auth_header)
        .await
        .map_err(problem_response)?;
    let summary = handler::execute(&state, &operator, &student_no, req)
        .await
        .map_err(problem_response)?;
    Ok((StatusCode::OK, Json(summary)))
}

fn problem_response(problem: ProblemDetails) -> (StatusCode, Json<ProblemDetails>) {
    let status = StatusCode::from_u16(problem.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    (status, Json(problem))
}

fn route() -> Router<AppState> {
    Router::new().route(
        "/api/v1/students/{student_no}/transfer",
        post(transfer_homeroom),
    )
}

inventory::submit!(SliceDescriptor {
    context: "enrollment",
    slice: "cmd_transfer_homeroom",
    method: HttpMethod::Post,
    path: "/api/v1/students/{student_no}/transfer",
    status: SliceStatus::Active,
    route,
});
