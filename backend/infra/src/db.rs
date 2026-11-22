use std::str::FromStr;

use sqlx::{
    SqlitePool,
    migrate::Migrator,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
};

use crate::error::InfraError;

pub static MIGRATOR: Migrator = sqlx::migrate!("../../migrations");

pub async fn open_sqlite_pool(database_url: &str) -> Result<SqlitePool, InfraError> {
    let options = SqliteConnectOptions::from_str(database_url)?
        .create_if_missing(true)
        .foreign_keys(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal);

    let max_connections = if database_url.contains(":memory:") {
        1
    } else {
        8
    };

    let pool = SqlitePoolOptions::new()
        .max_connections(max_connections)
        .connect_with(options)
        .await?;

    MIGRATOR.run(&pool).await?;
    Ok(pool)
}
