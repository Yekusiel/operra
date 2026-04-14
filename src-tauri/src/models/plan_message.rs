use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanMessage {
    pub id: String,
    pub plan_id: String,
    pub role: String, // "user" | "assistant"
    pub content: String,
    pub created_at: String,
}

impl PlanMessage {
    pub fn create(
        conn: &Connection,
        plan_id: &str,
        role: &str,
        content: &str,
    ) -> Result<PlanMessage, rusqlite::Error> {
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO plan_messages (id, plan_id, role, content) VALUES (?1, ?2, ?3, ?4)",
            params![id, plan_id, role, content],
        )?;
        Self::get_by_id(conn, &id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)
    }

    pub fn list_for_plan(conn: &Connection, plan_id: &str) -> Result<Vec<PlanMessage>, rusqlite::Error> {
        let mut stmt = conn.prepare(
            "SELECT id, plan_id, role, content, created_at
             FROM plan_messages WHERE plan_id = ?1 ORDER BY created_at ASC",
        )?;
        let rows = stmt.query_map(params![plan_id], |row| {
            Ok(PlanMessage {
                id: row.get(0)?,
                plan_id: row.get(1)?,
                role: row.get(2)?,
                content: row.get(3)?,
                created_at: row.get(4)?,
            })
        })?;
        rows.collect()
    }

    fn get_by_id(conn: &Connection, id: &str) -> Result<Option<PlanMessage>, rusqlite::Error> {
        let mut stmt = conn.prepare(
            "SELECT id, plan_id, role, content, created_at
             FROM plan_messages WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(params![id], |row| {
            Ok(PlanMessage {
                id: row.get(0)?,
                plan_id: row.get(1)?,
                role: row.get(2)?,
                content: row.get(3)?,
                created_at: row.get(4)?,
            })
        })?;
        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }
}
