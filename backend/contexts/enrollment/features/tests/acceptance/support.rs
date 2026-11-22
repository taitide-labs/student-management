use std::sync::Arc;

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Method, Request, StatusCode},
};
use serde::Serialize;
use serde_json::Value;
use sqlx::SqlitePool;
use student_management_infra::{
    AppState, FixedClock, SequentialIdGenerator, build_context_router, hash_password,
};
use tower::ServiceExt;

pub fn router(pool: SqlitePool) -> Router {
    let state = AppState::new(
        pool,
        Arc::new(FixedClock::new("2026-11-22T12:00:00Z", "2026-11-22")),
        Arc::new(SequentialIdGenerator::new(0)),
    );
    build_context_router("enrollment", state)
}

pub async fn seed_operator(pool: &SqlitePool, account_id: &str, role: &str, token: &str) {
    let hash = hash_password("pw").expect("hash password");
    sqlx::query(
        "INSERT INTO accounts (account_id, username, display_name, password_hash, role, status, version, created_at, updated_at) VALUES (?, ?, ?, ?, ?, 'active', 1, '2026-11-22T12:00:00Z', '2026-11-22T12:00:00Z')",
    )
    .bind(account_id)
    .bind(account_id)
    .bind("测试操作员")
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
