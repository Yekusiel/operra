use crate::db::AppDb;
use crate::models::project::{CreateProjectInput, Project};
use std::path::Path;

#[tauri::command]
pub async fn create_project(
    state: tauri::State<'_, AppDb>,
    input: CreateProjectInput,
) -> Result<Project, String> {
    let source_type = input.source_type.as_deref().unwrap_or("local");

    match source_type {
        "local" => {
            let repo_path = input.repo_path.as_deref().unwrap_or("");
            if repo_path.is_empty() {
                return Err("Repository path is required for local projects.".to_string());
            }
            let path = Path::new(repo_path);
            if !path.exists() {
                return Err(format!("Repository path does not exist: {}", repo_path));
            }
            if !path.is_dir() {
                return Err(format!("Repository path is not a directory: {}", repo_path));
            }
        }
        "github" => {
            let github_repo = input.github_repo.as_deref().unwrap_or("");
            if github_repo.is_empty() {
                return Err("GitHub repository (owner/repo) is required.".to_string());
            }
            if !github_repo.contains('/') {
                return Err("GitHub repository must be in owner/repo format.".to_string());
            }
        }
        _ => return Err(format!("Unknown source type: {}", source_type)),
    }

    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    Project::create(&conn, input).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_projects(
    state: tauri::State<'_, AppDb>,
) -> Result<Vec<Project>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    Project::list_all(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_project(
    state: tauri::State<'_, AppDb>,
    id: String,
) -> Result<Option<Project>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    Project::get_by_id(&conn, &id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_project(
    state: tauri::State<'_, AppDb>,
    id: String,
) -> Result<bool, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    Project::delete(&conn, &id).map_err(|e| e.to_string())
}
