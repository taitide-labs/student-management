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
    student_management_identity_features::link();
    let state = AppState::new(
        pool,
        Arc::new(FixedClock::new("2026-11-22T12:00:00Z", "2026-11-22")),
        Arc::new(SequentialIdGenerator::new(0)),
    );
    build_context_router("identity", state)
}

pub async fn seed_account(
    pool: &SqlitePool,
    account_id: &str,
    username: &str,
    role: &str,
    password: &str,
    token: &str,
) {
    let hash = hash_password(password).expect("hash password");
    sqlx::query(
        "INSERT INTO accounts (account_id, username, display_name, password_hash, role, status, version, created_at, updated_at) VALUES (?, ?, ?, ?, ?, 'active', 1, '2026-11-22T12:00:00Z', '2026-11-22T12:00:00Z')",
    )
    .bind(account_id)
    .bind(username)
    .bind(username)
    .bind(hash)
    .bind(role)
    .execute(pool)
    .await
    .expect("seed account");

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
    token: Option<&str>,
    body: Option<&T>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json");
    if let Some(tok) = token {
        builder = builder.header("authorization", format!("Bearer {tok}"));
    }
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
