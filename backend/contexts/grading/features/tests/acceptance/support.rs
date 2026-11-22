use std::{collections::HashMap, pin::Pin, sync::Arc};

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Method, Request, StatusCode},
};
use serde::Serialize;
use serde_json::Value;
use sqlx::SqlitePool;
use student_management_contracts::{
    EnrollmentQueryPort, EnrollmentStatus, HomeroomSnapshot, IdentityLookupPort, PortError,
    StudentEnrollmentSnapshot, TeacherIdentity,
};
use student_management_infra::{
    AppState, FixedClock, SequentialIdGenerator, build_context_router, hash_password,
};
use tower::ServiceExt;

pub struct FakeIdentityPort;

impl IdentityLookupPort for FakeIdentityPort {
    fn verify_teacher(
        &self,
        account_id: &str,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<TeacherIdentity, PortError>> + Send + '_>>
    {
        let id = account_id.to_owned();
        Box::pin(async move {
            if id == "acc-teacher" {
                Ok(TeacherIdentity {
                    account_id: id,
                    display_name: "林老师".to_owned(),
                })
            } else {
                Err(PortError::Forbidden("目标账号不是有效教师".to_owned()))
            }
        })
    }
}

pub struct FakeEnrollmentPort {
    students: HashMap<String, StudentEnrollmentSnapshot>,
}

impl FakeEnrollmentPort {
    pub fn with_active_student(student_no: &str, homeroom_id: &str) -> Self {
        let mut students = HashMap::new();
        students.insert(
            student_no.to_owned(),
            StudentEnrollmentSnapshot {
                student_no: student_no.to_owned(),
                full_name: "张晓明".to_owned(),
                homeroom_id: homeroom_id.to_owned(),
                status: EnrollmentStatus::Active,
            },
        );
        Self { students }
    }

    pub fn with_withdrawn_student(student_no: &str, homeroom_id: &str) -> Self {
        let mut students = HashMap::new();
        students.insert(
            student_no.to_owned(),
            StudentEnrollmentSnapshot {
                student_no: student_no.to_owned(),
                full_name: "已退学学生".to_owned(),
                homeroom_id: homeroom_id.to_owned(),
                status: EnrollmentStatus::Withdrawn,
            },
        );
        Self { students }
    }
}

impl EnrollmentQueryPort for FakeEnrollmentPort {
    fn verify_homeroom_exists(
        &self,
        homeroom_id: &str,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<HomeroomSnapshot, PortError>> + Send + '_>>
    {
        let id = homeroom_id.to_owned();
        Box::pin(async move {
            if id == "hr-1" {
                Ok(HomeroomSnapshot {
                    id,
                    name: "高一 (1) 班".to_owned(),
                })
            } else {
                Err(PortError::NotFound("行政班不存在".to_owned()))
            }
        })
    }

    fn check_student_active(
        &self,
        student_no: &str,
    ) -> Pin<
        Box<
            dyn std::future::Future<Output = Result<StudentEnrollmentSnapshot, PortError>>
                + Send
                + '_,
        >,
    > {
        let found = self.students.get(student_no).cloned();
        Box::pin(async move {
            let snap = found.ok_or_else(|| PortError::NotFound("学生不存在".to_owned()))?;
            if snap.status != EnrollmentStatus::Active {
                return Err(PortError::Conflict("学生非在籍状态".to_owned()));
            }
            Ok(snap)
        })
    }
}

pub fn router(pool: SqlitePool, enrollment_port: FakeEnrollmentPort) -> Router {
    student_management_grading_features::link();
    let state = AppState::new(
        pool,
        Arc::new(FixedClock::new("2026-11-22T12:00:00Z", "2026-11-22")),
        Arc::new(SequentialIdGenerator::new(0)),
    )
    .with_ports(Arc::new(FakeIdentityPort), Arc::new(enrollment_port));
    build_context_router("grading", state)
}

pub async fn seed_operator(pool: &SqlitePool, account_id: &str, role: &str, token: &str) {
    let hash = hash_password("pw").expect("hash");
    sqlx::query(
        "INSERT INTO accounts (account_id, username, display_name, password_hash, role, status, version, created_at, updated_at) VALUES (?, ?, ?, ?, ?, 'active', 1, '2026-11-22T12:00:00Z', '2026-11-22T12:00:00Z')",
    )
    .bind(account_id)
    .bind(account_id)
    .bind(account_id)
    .bind(hash)
    .bind(role)
    .execute(pool)
    .await
    .expect("seed operator");

    sqlx::query(
        "INSERT INTO sessions (token, account_id, created_at) VALUES (?, ?, '2026-11-22T12:00:00Z')",
    )
    .bind(token)
    .bind(account_id)
    .execute(pool)
    .await
    .expect("seed session");
}

pub async fn send_json<T: Serialize>(
    app: &Router,
    method: Method,
    uri: &str,
    token: &str,
    body: Option<&T>,
) -> (StatusCode, Value) {
    let builder = Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"));
    let bytes = match body {
        Some(payload) => serde_json::to_vec(payload).expect("serialize request"),
        None => Vec::new(),
    };
    let req = builder.body(Body::from(bytes)).expect("build request");
    let resp = app.clone().oneshot(req).await.expect("execute request");
    let status = resp.status();
    let raw = to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .expect("read body");
    let parsed = if raw.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&raw).expect("parse json response")
    };
    (status, parsed)
}
