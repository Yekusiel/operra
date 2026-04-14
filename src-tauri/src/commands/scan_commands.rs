use crate::db::AppDb;
use crate::models::project::Project;
use crate::models::scan::{Scan, ScanFinding};
use crate::scanner;
use crate::scanner::types::{ScanProgress, ScanReport};
use std::path::Path;
use tauri::ipc::Channel;

#[tauri::command]
pub async fn start_scan(
    state: tauri::State<'_, AppDb>,
    project_id: String,
    on_progress: Channel<ScanProgress>,
) -> Result<ScanReport, String> {
    // Look up project
    let (repo_path, scan_id) = {
        let conn = state.conn.lock().map_err(|e| e.to_string())?;
        let project = Project::get_by_id(&conn, &project_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Project not found: {}", project_id))?;

        let scan = Scan::create(&conn, &project_id).map_err(|e| e.to_string())?;
        (project.repo_path, scan.id)
    };

    // Run scanner (outside the lock)
    let result = scanner::run_scan(Path::new(&repo_path), |progress| {
        let _ = on_progress.send(progress);
    });

    // Store results
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    match result {
        Ok(report) => {
            ScanFinding::insert_batch(&conn, &scan_id, &report.detections)
                .map_err(|e| e.to_string())?;
            Scan::complete(&conn, &scan_id).map_err(|e| e.to_string())?;
            Ok(report)
        }
        Err(err) => {
            Scan::fail(&conn, &scan_id, &err).map_err(|e| e.to_string())?;
            Err(err)
        }
    }
}

#[tauri::command]
pub async fn get_scan_results(
    state: tauri::State<'_, AppDb>,
    scan_id: String,
) -> Result<(Scan, Vec<ScanFinding>), String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let scan = Scan::get_by_id(&conn, &scan_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Scan not found: {}", scan_id))?;
    let findings = ScanFinding::list_for_scan(&conn, &scan_id).map_err(|e| e.to_string())?;
    Ok((scan, findings))
}

#[tauri::command]
pub async fn list_scans_for_project(
    state: tauri::State<'_, AppDb>,
    project_id: String,
) -> Result<Vec<Scan>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    Scan::list_for_project(&conn, &project_id).map_err(|e| e.to_string())
}
