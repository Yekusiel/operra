use crate::db::AppDb;
use crate::models::project::{CreateProjectInput, Project};
use std::path::Path;

#[tauri::command]
pub async fn create_project(
    state: tauri::State<'_, AppDb>,
    input: CreateProjectInput,
) -> Result<Project, String> {
    // Validate repo path exists
    let path = Path::new(&input.repo_path);
    if !path.exists() {
        return Err(format!("Repository path does not exist: {}", input.repo_path));
    }
    if !path.is_dir() {
        return Err(format!("Repository path is not a directory: {}", input.repo_path));
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
