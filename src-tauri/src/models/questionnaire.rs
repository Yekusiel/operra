use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionnaireResponse {
    pub id: String,
    pub project_id: String,
    pub answers_json: String,
    pub completed: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectureAnswers {
    pub expected_traffic: Option<String>,
    pub environment: Option<String>,
    pub uptime_requirements: Option<String>,
    pub preferred_region: Option<String>,
    pub budget_sensitivity: Option<String>,
    pub database_needs: Option<String>,
    pub background_jobs: Option<String>,
    pub networking: Option<String>,
    pub storage_needs: Option<String>,
    pub cost_vs_performance: Option<String>,
    pub custom_notes: Option<String>,
}

impl QuestionnaireResponse {
    pub fn create_or_get(conn: &Connection, project_id: &str) -> Result<QuestionnaireResponse, rusqlite::Error> {
        // Return existing incomplete questionnaire if one exists
        if let Some(existing) = Self::get_latest(conn, project_id)? {
            if !existing.completed {
                return Ok(existing);
            }
        }

        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO questionnaire_responses (id, project_id) VALUES (?1, ?2)",
            params![id, project_id],
        )?;

        Self::get_by_id(conn, &id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)
    }

    pub fn save_answers(
        conn: &Connection,
        id: &str,
        answers_json: &str,
        completed: bool,
    ) -> Result<(), rusqlite::Error> {
        let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        conn.execute(
            "UPDATE questionnaire_responses SET answers_json = ?1, completed = ?2, updated_at = ?3 WHERE id = ?4",
            params![answers_json, completed as i32, now, id],
        )?;
        Ok(())
    }

    pub fn get_by_id(conn: &Connection, id: &str) -> Result<Option<QuestionnaireResponse>, rusqlite::Error> {
        let mut stmt = conn.prepare(
            "SELECT id, project_id, answers_json, completed, created_at, updated_at
             FROM questionnaire_responses WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(params![id], Self::from_row)?;
        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }

    pub fn get_latest(conn: &Connection, project_id: &str) -> Result<Option<QuestionnaireResponse>, rusqlite::Error> {
        let mut stmt = conn.prepare(
            "SELECT id, project_id, answers_json, completed, created_at, updated_at
             FROM questionnaire_responses WHERE project_id = ?1 ORDER BY created_at DESC LIMIT 1",
        )?;
        let mut rows = stmt.query_map(params![project_id], Self::from_row)?;
        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }

    fn from_row(row: &rusqlite::Row) -> Result<QuestionnaireResponse, rusqlite::Error> {
        Ok(QuestionnaireResponse {
            id: row.get(0)?,
            project_id: row.get(1)?,
            answers_json: row.get(2)?,
            completed: row.get::<_, i32>(3)? != 0,
            created_at: row.get(4)?,
            updated_at: row.get(5)?,
        })
    }
}
