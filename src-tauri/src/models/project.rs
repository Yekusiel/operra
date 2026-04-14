use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub repo_path: String,
    pub aws_profile: Option<String>,
    pub aws_region: String,
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateProjectInput {
    pub name: String,
    pub repo_path: String,
    pub aws_profile: Option<String>,
    pub aws_region: Option<String>,
    pub description: Option<String>,
}

impl Project {
    pub fn create(conn: &Connection, input: CreateProjectInput) -> Result<Project, rusqlite::Error> {
        let id = Uuid::new_v4().to_string();
        let region = input.aws_region.unwrap_or_else(|| "us-east-1".to_string());

        conn.execute(
            "INSERT INTO projects (id, name, repo_path, aws_profile, aws_region, description)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![id, input.name, input.repo_path, input.aws_profile, region, input.description],
        )?;

        Self::get_by_id(conn, &id)?.ok_or_else(|| {
            rusqlite::Error::QueryReturnedNoRows
        })
    }

    pub fn list_all(conn: &Connection) -> Result<Vec<Project>, rusqlite::Error> {
        let mut stmt = conn.prepare(
            "SELECT id, name, repo_path, aws_profile, aws_region, description, created_at, updated_at
             FROM projects ORDER BY created_at DESC",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                repo_path: row.get(2)?,
                aws_profile: row.get(3)?,
                aws_region: row.get(4)?,
                description: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })?;

        rows.collect()
    }

    pub fn get_by_id(conn: &Connection, id: &str) -> Result<Option<Project>, rusqlite::Error> {
        let mut stmt = conn.prepare(
            "SELECT id, name, repo_path, aws_profile, aws_region, description, created_at, updated_at
             FROM projects WHERE id = ?1",
        )?;

        let mut rows = stmt.query_map(params![id], |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                repo_path: row.get(2)?,
                aws_profile: row.get(3)?,
                aws_region: row.get(4)?,
                description: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })?;

        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }

    pub fn delete(conn: &Connection, id: &str) -> Result<bool, rusqlite::Error> {
        let affected = conn.execute("DELETE FROM projects WHERE id = ?1", params![id])?;
        Ok(affected > 0)
    }
}
