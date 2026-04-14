use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterLog {
    pub id: String,
    pub project_id: String,
    pub adapter_name: String,
    pub task_type: String,
    pub prompt_text: String,
    pub response_text: Option<String>,
    pub response_json: Option<String>,
    pub status: String,
    pub duration_ms: Option<i64>,
    pub error_msg: Option<String>,
    pub created_at: String,
}

impl AdapterLog {
    pub fn create(
        conn: &Connection,
        project_id: &str,
        adapter_name: &str,
        task_type: &str,
        prompt_text: &str,
    ) -> Result<AdapterLog, rusqlite::Error> {
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO adapter_logs (id, project_id, adapter_name, task_type, prompt_text, status)
             VALUES (?1, ?2, ?3, ?4, ?5, 'running')",
            params![id, project_id, adapter_name, task_type, prompt_text],
        )?;
        Self::get_by_id(conn, &id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)
    }

    pub fn complete(
        conn: &Connection,
        id: &str,
        response_text: &str,
        response_json: Option<&str>,
        duration_ms: i64,
    ) -> Result<(), rusqlite::Error> {
        conn.execute(
            "UPDATE adapter_logs SET status = 'completed', response_text = ?1, response_json = ?2, duration_ms = ?3 WHERE id = ?4",
            params![response_text, response_json, duration_ms, id],
        )?;
        Ok(())
    }

    pub fn fail(conn: &Connection, id: &str, error: &str, duration_ms: i64) -> Result<(), rusqlite::Error> {
        conn.execute(
            "UPDATE adapter_logs SET status = 'failed', error_msg = ?1, duration_ms = ?2 WHERE id = ?3",
            params![error, duration_ms, id],
        )?;
        Ok(())
    }

    pub fn get_by_id(conn: &Connection, id: &str) -> Result<Option<AdapterLog>, rusqlite::Error> {
        let mut stmt = conn.prepare(
            "SELECT id, project_id, adapter_name, task_type, prompt_text, response_text,
                    response_json, status, duration_ms, error_msg, created_at
             FROM adapter_logs WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(params![id], Self::from_row)?;
        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }

    fn from_row(row: &rusqlite::Row) -> Result<AdapterLog, rusqlite::Error> {
        Ok(AdapterLog {
            id: row.get(0)?,
            project_id: row.get(1)?,
            adapter_name: row.get(2)?,
            task_type: row.get(3)?,
            prompt_text: row.get(4)?,
            response_text: row.get(5)?,
            response_json: row.get(6)?,
            status: row.get(7)?,
            duration_ms: row.get(8)?,
            error_msg: row.get(9)?,
            created_at: row.get(10)?,
        })
    }
}
