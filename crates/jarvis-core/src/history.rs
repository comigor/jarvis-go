use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::db::{get_db, DbError};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub id: i64,
    pub session_id: String,
    pub role: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Error)]
pub enum HistoryError {
    #[error(transparent)]
    Db(#[from] DbError),
    #[error(transparent)]
    Sql(#[from] libsql::Error),
}

pub async fn save(msg: Message) -> Result<(), HistoryError> {
    let db = get_db().await?;
    let conn = db.connect()?;
    conn.execute(
        "INSERT INTO messages (session_id, role, content, created_at) VALUES (?1, ?2, ?3, ?4);",
        (
            msg.session_id.as_str(),
            msg.role.as_str(),
            msg.content.as_str(),
            msg.created_at.timestamp(),
        ),
    )
    .await?;
    Ok(())
}

pub async fn list(session_id: &str) -> Result<Vec<Message>, HistoryError> {
    let db = get_db().await?;
    let conn = db.connect()?;
    let params = libsql::params![session_id];
    let mut rows = conn
        .query(
            "SELECT id, session_id, role, content, created_at FROM messages WHERE session_id = ?1 ORDER BY id ASC;",
            params,
        )
        .await?;

    let mut out = Vec::new();
    while let Some(row) = rows.next().await? {
        let id: i64 = row.get::<i64>(0)?;
        let sid: String = row.get::<String>(1)?;
        let role: String = row.get::<String>(2)?;
        let content: String = row.get::<String>(3)?;
        let ts: i64 = row.get::<i64>(4)?;
        out.push(Message {
            id,
            session_id: sid,
            role,
            content,
            created_at: DateTime::<Utc>::from_timestamp(ts, 0).unwrap(),
        });
    }
    Ok(out)
}
