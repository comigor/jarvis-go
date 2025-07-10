use libsql::{Builder, Database, Connection};
use tokio::sync::OnceCell;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("libsql error: {0}")]
    Libsql(#[from] libsql::Error),
}

static DB: OnceCell<Database> = OnceCell::const_new();

pub async fn get_db() -> Result<&'static Database, DbError> {
    DB.get_or_try_init(init_db).await
}

async fn init_db() -> Result<Database, DbError> {
    let path = std::env::var("HISTORY_DB_PATH").unwrap_or_else(|_| "history.db".into());
    let db = if path == ":memory:" {
        Builder::new_local(&path).build().await?
    } else {
        if let Some(parent) = Path::new(&path).parent() {
            tokio::fs::create_dir_all(parent).await.ok();
        }
        Builder::new_local(&path).build().await?
    };

    let conn = db.connect()?;
    init_schema(&conn).await?;
    Ok(db)
}

async fn init_schema(conn: &Connection) -> Result<(), DbError> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS messages (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            session_id TEXT,
            role TEXT,
            content TEXT,
            created_at INTEGER
        );",
        (),
    )
    .await?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_messages_session ON messages(session_id);",
        (),
    ).await?;
    Ok(())
}
