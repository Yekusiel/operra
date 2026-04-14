use crate::db::AppDb;
use crate::models::aws_connection::AwsConnection;
use crate::models::project::Project;
use crate::tools::aws;
use crate::tools::dependency_check::{self, DependencyReport};

#[tauri::command]
pub async fn check_dependencies() -> Result<DependencyReport, String> {
    Ok(dependency_check::check_all().await)
}

#[tauri::command]
pub async fn test_aws_connection(
    state: tauri::State<'_, AppDb>,
    project_id: String,
) -> Result<AwsConnection, String> {
    let project = {
        let conn = state.conn.lock().map_err(|e| e.to_string())?;
        Project::get_by_id(&conn, &project_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Project not found: {}", project_id))?
    };

    let result = aws::test_connection(
        project.aws_profile.as_deref(),
        Some(&project.aws_region),
    )
    .await;

    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    if result.connected {
        let identity = result.identity.as_ref().unwrap();
        AwsConnection::upsert(
            &conn,
            &project_id,
            Some(&identity.account),
            Some(&identity.arn),
            Some(&identity.user_id),
            "connected",
            None,
        )
        .map_err(|e| e.to_string())
    } else {
        AwsConnection::upsert(
            &conn,
            &project_id,
            None,
            None,
            None,
            "failed",
            result.error.as_deref(),
        )
        .map_err(|e| e.to_string())
    }
}

#[tauri::command]
pub async fn get_aws_connection(
    state: tauri::State<'_, AppDb>,
    project_id: String,
) -> Result<Option<AwsConnection>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    AwsConnection::get_for_project(&conn, &project_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_aws_profiles() -> Result<Vec<String>, String> {
    Ok(aws::list_profiles().await)
}
