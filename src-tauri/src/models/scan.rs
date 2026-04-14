use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scan {
    pub id: String,
    pub project_id: String,
    pub status: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub error_msg: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanFinding {
    pub id: String,
    pub scan_id: String,
    pub category: String,
    pub name: String,
    pub confidence: f64,
    pub evidence_path: Option<String>,
    pub metadata_json: Option<String>,
    pub created_at: String,
}

impl Scan {
    pub fn create(conn: &Connection, project_id: &str) -> Result<Scan, rusqlite::Error> {
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

        conn.execute(
            "INSERT INTO scans (id, project_id, status, started_at) VALUES (?1, ?2, 'running', ?3)",
            params![id, project_id, now],
        )?;

        Self::get_by_id(conn, &id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)
    }

    pub fn complete(conn: &Connection, id: &str) -> Result<(), rusqlite::Error> {
        let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        conn.execute(
            "UPDATE scans SET status = 'completed', completed_at = ?1 WHERE id = ?2",
            params![now, id],
        )?;
        Ok(())
    }

    pub fn fail(conn: &Connection, id: &str, error: &str) -> Result<(), rusqlite::Error> {
        let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        conn.execute(
            "UPDATE scans SET status = 'failed', completed_at = ?1, error_msg = ?2 WHERE id = ?3",
            params![now, error, id],
        )?;
        Ok(())
    }

    pub fn get_by_id(conn: &Connection, id: &str) -> Result<Option<Scan>, rusqlite::Error> {
        let mut stmt = conn.prepare(
            "SELECT id, project_id, status, started_at, completed_at, error_msg, created_at
             FROM scans WHERE id = ?1",
        )?;

        let mut rows = stmt.query_map(params![id], Self::from_row)?;
        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }

    pub fn list_for_project(conn: &Connection, project_id: &str) -> Result<Vec<Scan>, rusqlite::Error> {
        let mut stmt = conn.prepare(
            "SELECT id, project_id, status, started_at, completed_at, error_msg, created_at
             FROM scans WHERE project_id = ?1 ORDER BY created_at DESC",
        )?;

        let rows = stmt.query_map(params![project_id], Self::from_row)?;
        rows.collect()
    }

    fn from_row(row: &rusqlite::Row) -> Result<Scan, rusqlite::Error> {
        Ok(Scan {
            id: row.get(0)?,
            project_id: row.get(1)?,
            status: row.get(2)?,
            started_at: row.get(3)?,
            completed_at: row.get(4)?,
            error_msg: row.get(5)?,
            created_at: row.get(6)?,
        })
    }
}

impl ScanFinding {
    pub fn insert_batch(
        conn: &Connection,
        scan_id: &str,
        findings: &[crate::scanner::types::Detection],
    ) -> Result<Vec<ScanFinding>, rusqlite::Error> {
        let mut result = Vec::new();

        for detection in findings {
            let id = Uuid::new_v4().to_string();
            let metadata = detection
                .metadata
                .as_ref()
                .map(|m| serde_json::to_string(m).unwrap_or_default());

            conn.execute(
                "INSERT INTO scan_findings (id, scan_id, category, name, confidence, evidence_path, metadata_json)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    id,
                    scan_id,
                    detection.category.as_str(),
                    detection.name,
                    detection.confidence,
                    detection.evidence_path,
                    metadata,
                ],
            )?;

            result.push(ScanFinding {
                id,
                scan_id: scan_id.to_string(),
                category: detection.category.as_str().to_string(),
                name: detection.name.clone(),
                confidence: detection.confidence,
                evidence_path: Some(detection.evidence_path.clone()),
                metadata_json: metadata,
                created_at: chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            });
        }

        Ok(result)
    }

    pub fn list_for_scan(conn: &Connection, scan_id: &str) -> Result<Vec<ScanFinding>, rusqlite::Error> {
        let mut stmt = conn.prepare(
            "SELECT id, scan_id, category, name, confidence, evidence_path, metadata_json, created_at
             FROM scan_findings WHERE scan_id = ?1 ORDER BY category, name",
        )?;

        let rows = stmt.query_map(params![scan_id], |row| {
            Ok(ScanFinding {
                id: row.get(0)?,
                scan_id: row.get(1)?,
                category: row.get(2)?,
                name: row.get(3)?,
                confidence: row.get(4)?,
                evidence_path: row.get(5)?,
                metadata_json: row.get(6)?,
                created_at: row.get(7)?,
            })
        })?;

        rows.collect()
    }
}
