use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub source_type: String, // "local" or "github"
    pub repo_path: String,   // local path (always set -- for github, this is the clone dir)
    pub github_repo: Option<String>,   // "owner/repo"
    pub github_branch: Option<String>, // "main", "master", etc.
    pub aws_profile: Option<String>,
    pub aws_region: String,
    pub aws_access_key_id: Option<String>,
    pub aws_secret_access_key: Option<String>,
    pub domain: Option<String>,
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateProjectInput {
    pub name: String,
    pub source_type: Option<String>,
    pub repo_path: Option<String>,
    pub github_repo: Option<String>,
    pub github_branch: Option<String>,
    pub aws_profile: Option<String>,
    pub aws_region: Option<String>,
    pub aws_access_key_id: Option<String>,
    pub aws_secret_access_key: Option<String>,
    pub domain: Option<String>,
    pub description: Option<String>,
}

impl Project {
    pub fn create(conn: &Connection, input: CreateProjectInput) -> Result<Project, rusqlite::Error> {
        let id = Uuid::new_v4().to_string();
        let region = input.aws_region.unwrap_or_else(|| "us-east-1".to_string());
        let source_type = input.source_type.unwrap_or_else(|| "local".to_string());
        let repo_path = input.repo_path.unwrap_or_default();
        let branch = input.github_branch.or(Some("main".to_string()));

        conn.execute(
            "INSERT INTO projects (id, name, source_type, repo_path, github_repo, github_branch, aws_profile, aws_region, aws_access_key_id, aws_secret_access_key, domain, description)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![id, input.name, source_type, repo_path, input.github_repo, branch, input.aws_profile, region, input.aws_access_key_id, input.aws_secret_access_key, input.domain, input.description],
        )?;

        Self::get_by_id(conn, &id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)
    }

    pub fn list_all(conn: &Connection) -> Result<Vec<Project>, rusqlite::Error> {
        let mut stmt = conn.prepare(
            "SELECT id, name, source_type, repo_path, github_repo, github_branch,
                    aws_profile, aws_region, aws_access_key_id, aws_secret_access_key,
                    domain, description, created_at, updated_at
             FROM projects ORDER BY created_at DESC",
        )?;

        let rows = stmt.query_map([], Self::from_row)?;
        rows.collect()
    }

    pub fn get_by_id(conn: &Connection, id: &str) -> Result<Option<Project>, rusqlite::Error> {
        let mut stmt = conn.prepare(
            "SELECT id, name, source_type, repo_path, github_repo, github_branch,
                    aws_profile, aws_region, aws_access_key_id, aws_secret_access_key,
                    domain, description, created_at, updated_at
             FROM projects WHERE id = ?1",
        )?;

        let mut rows = stmt.query_map(params![id], Self::from_row)?;
        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }

    pub fn delete(conn: &Connection, id: &str) -> Result<bool, rusqlite::Error> {
        let affected = conn.execute("DELETE FROM projects WHERE id = ?1", params![id])?;
        Ok(affected > 0)
    }

    fn from_row(row: &rusqlite::Row) -> Result<Project, rusqlite::Error> {
        Ok(Project {
            id: row.get(0)?,
            name: row.get(1)?,
            source_type: row.get(2)?,
            repo_path: row.get(3)?,
            github_repo: row.get(4)?,
            github_branch: row.get(5)?,
            aws_profile: row.get(6)?,
            aws_region: row.get(7)?,
            aws_access_key_id: row.get(8)?,
            aws_secret_access_key: row.get(9)?,
            domain: row.get(10)?,
            description: row.get(11)?,
            created_at: row.get(12)?,
            updated_at: row.get(13)?,
        })
    }
}
