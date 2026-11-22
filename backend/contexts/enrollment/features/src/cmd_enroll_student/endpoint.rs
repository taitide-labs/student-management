use axum::{Json, Router, http::StatusCode, routing::post};
use student_management_contracts::{EnrollStudentRequest, ProblemDetails, StudentSummary};
use student_management_infra::{HttpMethod, SliceDescriptor, SliceStatus, state::AppState};

#[utoipa::path(
    post,
    path = "/api/v1/students",
    tag = "enrollment",
    request_body = EnrollStudentRequest,
    responses(
        (status = 201, description = "入学建档成功", body = StudentSummary),
        (status = 409, description = "班级满员或幂等冲突", body = ProblemDetails)
    )
)]
pub async fn enroll_student(
    Json(_req): Json<EnrollStudentRequest>,
) -> (StatusCode, Json<ProblemDetails>) {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(ProblemDetails::not_implemented(
            "enrollment::cmd_enroll_student",
        )),
    )
}

fn route() -> Router<AppState> {
    Router::new().route("/api/v1/students", post(enroll_student))
}

inventory::submit!(SliceDescriptor {
    context: "enrollment",
    slice: "cmd_enroll_student",
    method: HttpMethod::Post,
    path: "/api/v1/students",
    status: SliceStatus::Planned,
    route,
});
