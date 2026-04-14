use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub id: String,
    pub project_id: String,
    pub scan_id: Option<String>,
    pub questionnaire_id: Option<String>,
    pub status: String,
    pub plan_markdown: Option<String>,
    pub plan_json: Option<String>,
    pub alternatives: Option<String>,
    pub cost_notes: Option<String>,
    pub error_msg: Option<String>,
    pub approved: bool,
    pub approved_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl Plan {
    pub fn create(
        conn: &Connection,
        project_id: &str,
        scan_id: Option<&str>,
        questionnaire_id: Option<&str>,
    ) -> Result<Plan, rusqlite::Error> {
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO plans (id, project_id, scan_id, questionnaire_id, status)
             VALUES (?1, ?2, ?3, ?4, 'generating')",
            params![id, project_id, scan_id, questionnaire_id],
        )?;
        Self::get_by_id(conn, &id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)
    }

    pub fn complete(
        conn: &Connection,
        id: &str,
        plan_markdown: &str,
        plan_json: Option<&str>,
        alternatives: Option<&str>,
        cost_notes: Option<&str>,
    ) -> Result<(), rusqlite::Error> {
        let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        conn.execute(
            "UPDATE plans SET status = 'completed', plan_markdown = ?1, plan_json = ?2,
             alternatives = ?3, cost_notes = ?4, updated_at = ?5 WHERE id = ?6",
            params![plan_markdown, plan_json, alternatives, cost_notes, now, id],
        )?;
        Ok(())
    }

    pub fn fail(conn: &Connection, id: &str, error: &str) -> Result<(), rusqlite::Error> {
        let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        conn.execute(
            "UPDATE plans SET status = 'failed', error_msg = ?1, updated_at = ?2 WHERE id = ?3",
            params![error, now, id],
        )?;
        Ok(())
    }

    pub fn approve(conn: &Connection, id: &str) -> Result<(), rusqlite::Error> {
        let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        conn.execute(
            "UPDATE plans SET approved = 1, approved_at = ?1, updated_at = ?1 WHERE id = ?2",
            params![now, id],
        )?;
        Ok(())
    }

    pub fn get_approved_for_project(conn: &Connection, project_id: &str) -> Result<Option<Plan>, rusqlite::Error> {
        let mut stmt = conn.prepare(
            "SELECT id, project_id, scan_id, questionnaire_id, status, plan_markdown,
                    plan_json, alternatives, cost_notes, error_msg, approved, approved_at, created_at, updated_at
             FROM plans WHERE project_id = ?1 AND approved = 1 ORDER BY approved_at DESC LIMIT 1",
        )?;
        let mut rows = stmt.query_map(params![project_id], Self::from_row)?;
        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }

    pub fn get_by_id(conn: &Connection, id: &str) -> Result<Option<Plan>, rusqlite::Error> {
        let mut stmt = conn.prepare(
            "SELECT id, project_id, scan_id, questionnaire_id, status, plan_markdown,
                    plan_json, alternatives, cost_notes, error_msg, approved, approved_at, created_at, updated_at
             FROM plans WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(params![id], Self::from_row)?;
        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }

    pub fn get_latest(conn: &Connection, project_id: &str) -> Result<Option<Plan>, rusqlite::Error> {
        let mut stmt = conn.prepare(
            "SELECT id, project_id, scan_id, questionnaire_id, status, plan_markdown,
                    plan_json, alternatives, cost_notes, error_msg, approved, approved_at, created_at, updated_at
             FROM plans WHERE project_id = ?1 ORDER BY created_at DESC LIMIT 1",
        )?;
        let mut rows = stmt.query_map(params![project_id], Self::from_row)?;
        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }

    pub fn list_for_project(conn: &Connection, project_id: &str) -> Result<Vec<Plan>, rusqlite::Error> {
        let mut stmt = conn.prepare(
            "SELECT id, project_id, scan_id, questionnaire_id, status, plan_markdown,
                    plan_json, alternatives, cost_notes, error_msg, approved, approved_at, created_at, updated_at
             FROM plans WHERE project_id = ?1 ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map(params![project_id], Self::from_row)?;
        rows.collect()
    }

    fn from_row(row: &rusqlite::Row) -> Result<Plan, rusqlite::Error> {
        Ok(Plan {
            id: row.get(0)?,
            project_id: row.get(1)?,
            scan_id: row.get(2)?,
            questionnaire_id: row.get(3)?,
            status: row.get(4)?,
            plan_markdown: row.get(5)?,
            plan_json: row.get(6)?,
            alternatives: row.get(7)?,
            cost_notes: row.get(8)?,
            error_msg: row.get(9)?,
            approved: row.get::<_, i32>(10)? != 0,
            approved_at: row.get(11)?,
            created_at: row.get(12)?,
            updated_at: row.get(13)?,
        })
    }
}
