use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwsConnection {
    pub id: String,
    pub project_id: String,
    pub account_id: Option<String>,
    pub arn: Option<String>,
    pub user_id: Option<String>,
    pub status: String,
    pub error_msg: Option<String>,
    pub checked_at: Option<String>,
    pub created_at: String,
}

impl AwsConnection {
    pub fn upsert(
        conn: &Connection,
        project_id: &str,
        account_id: Option<&str>,
        arn: Option<&str>,
        user_id: Option<&str>,
        status: &str,
        error_msg: Option<&str>,
    ) -> Result<AwsConnection, rusqlite::Error> {
        let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        let existing = Self::get_for_project(conn, project_id)?;

        if let Some(existing) = existing {
            conn.execute(
                "UPDATE aws_connections SET account_id = ?1, arn = ?2, user_id = ?3,
                 status = ?4, error_msg = ?5, checked_at = ?6 WHERE id = ?7",
                params![account_id, arn, user_id, status, error_msg, now, existing.id],
            )?;
            Self::get_for_project(conn, project_id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)
        } else {
            let id = Uuid::new_v4().to_string();
            conn.execute(
                "INSERT INTO aws_connections (id, project_id, account_id, arn, user_id, status, error_msg, checked_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![id, project_id, account_id, arn, user_id, status, error_msg, now],
            )?;
            Self::get_for_project(conn, project_id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)
        }
    }

    pub fn get_for_project(conn: &Connection, project_id: &str) -> Result<Option<AwsConnection>, rusqlite::Error> {
        let mut stmt = conn.prepare(
            "SELECT id, project_id, account_id, arn, user_id, status, error_msg, checked_at, created_at
             FROM aws_connections WHERE project_id = ?1",
        )?;
        let mut rows = stmt.query_map(params![project_id], |row| {
            Ok(AwsConnection {
                id: row.get(0)?,
                project_id: row.get(1)?,
                account_id: row.get(2)?,
                arn: row.get(3)?,
                user_id: row.get(4)?,
                status: row.get(5)?,
                error_msg: row.get(6)?,
                checked_at: row.get(7)?,
                created_at: row.get(8)?,
            })
        })?;
        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }
}
