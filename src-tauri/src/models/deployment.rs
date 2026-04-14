use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deployment {
    pub id: String,
    pub project_id: String,
    pub iac_id: Option<String>,
    pub action: String,
    pub status: String,
    pub plan_output: Option<String>,
    pub plan_summary: Option<String>,
    pub apply_output: Option<String>,
    pub resources_json: Option<String>,
    pub risk_level: Option<String>,
    pub approved: bool,
    pub approved_at: Option<String>,
    pub error_msg: Option<String>,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub created_at: String,
}

impl Deployment {
    pub fn create(
        conn: &Connection,
        project_id: &str,
        iac_id: Option<&str>,
        action: &str,
    ) -> Result<Deployment, rusqlite::Error> {
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        conn.execute(
            "INSERT INTO deployments (id, project_id, iac_id, action, status, started_at)
             VALUES (?1, ?2, ?3, ?4, 'planning', ?5)",
            params![id, project_id, iac_id, action, now],
        )?;
        Self::get_by_id(conn, &id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)
    }

    pub fn save_plan_output(
        conn: &Connection,
        id: &str,
        plan_output: &str,
        plan_summary: &str,
        risk_level: &str,
    ) -> Result<(), rusqlite::Error> {
        conn.execute(
            "UPDATE deployments SET status = 'awaiting_approval', plan_output = ?1,
             plan_summary = ?2, risk_level = ?3 WHERE id = ?4",
            params![plan_output, plan_summary, risk_level, id],
        )?;
        Ok(())
    }

    pub fn approve(conn: &Connection, id: &str) -> Result<(), rusqlite::Error> {
        let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        conn.execute(
            "UPDATE deployments SET approved = 1, approved_at = ?1, status = 'approved' WHERE id = ?2",
            params![now, id],
        )?;
        Ok(())
    }

    pub fn start_apply(conn: &Connection, id: &str) -> Result<(), rusqlite::Error> {
        conn.execute(
            "UPDATE deployments SET status = 'applying' WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }

    pub fn complete(
        conn: &Connection,
        id: &str,
        apply_output: &str,
        resources_json: Option<&str>,
    ) -> Result<(), rusqlite::Error> {
        let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        conn.execute(
            "UPDATE deployments SET status = 'completed', apply_output = ?1,
             resources_json = ?2, completed_at = ?3 WHERE id = ?4",
            params![apply_output, resources_json, now, id],
        )?;
        Ok(())
    }

    pub fn fail(conn: &Connection, id: &str, error: &str) -> Result<(), rusqlite::Error> {
        let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        conn.execute(
            "UPDATE deployments SET status = 'failed', error_msg = ?1, completed_at = ?2 WHERE id = ?3",
            params![error, now, id],
        )?;
        Ok(())
    }

    pub fn get_by_id(conn: &Connection, id: &str) -> Result<Option<Deployment>, rusqlite::Error> {
        let mut stmt = conn.prepare(
            "SELECT id, project_id, iac_id, action, status, plan_output, plan_summary,
                    apply_output, resources_json, risk_level, approved, approved_at,
                    error_msg, started_at, completed_at, created_at
             FROM deployments WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(params![id], Self::from_row)?;
        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }

    pub fn list_for_project(conn: &Connection, project_id: &str) -> Result<Vec<Deployment>, rusqlite::Error> {
        let mut stmt = conn.prepare(
            "SELECT id, project_id, iac_id, action, status, plan_output, plan_summary,
                    apply_output, resources_json, risk_level, approved, approved_at,
                    error_msg, started_at, completed_at, created_at
             FROM deployments WHERE project_id = ?1 ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map(params![project_id], Self::from_row)?;
        rows.collect()
    }

    fn from_row(row: &rusqlite::Row) -> Result<Deployment, rusqlite::Error> {
        Ok(Deployment {
            id: row.get(0)?,
            project_id: row.get(1)?,
            iac_id: row.get(2)?,
            action: row.get(3)?,
            status: row.get(4)?,
            plan_output: row.get(5)?,
            plan_summary: row.get(6)?,
            apply_output: row.get(7)?,
            resources_json: row.get(8)?,
            risk_level: row.get(9)?,
            approved: row.get::<_, i32>(10)? != 0,
            approved_at: row.get(11)?,
            error_msg: row.get(12)?,
            started_at: row.get(13)?,
            completed_at: row.get(14)?,
            created_at: row.get(15)?,
        })
    }
}
