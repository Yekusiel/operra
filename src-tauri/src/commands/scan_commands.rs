use crate::db::AppDb;
use crate::models::project::Project;
use crate::models::scan::{Scan, ScanFinding};
use crate::scanner;
use crate::scanner::types::{ScanProgress, ScanReport};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tauri::ipc::Channel;

/// Clone a GitHub repo to a temp directory for scanning.
async fn clone_github_repo(repo: &str, branch: &str) -> Result<PathBuf, String> {
    let temp_dir = std::env::temp_dir().join(format!("operra-scan-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir)
        .map_err(|e| format!("Failed to create temp dir: {}", e))?;

    let url = format!("https://github.com/{}.git", repo);

    let output = if cfg!(windows) {
        tokio::process::Command::new("cmd")
            .args(["/C", "git", "clone", "--depth", "1", "--branch", branch, &url])
            .arg(temp_dir.to_string_lossy().as_ref())
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
    } else {
        tokio::process::Command::new("git")
            .args(["clone", "--depth", "1", "--branch", branch, &url])
            .arg(temp_dir.to_string_lossy().as_ref())
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
    };

    match output {
        Ok(out) if out.status.success() => Ok(temp_dir),
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            let _ = std::fs::remove_dir_all(&temp_dir);
            Err(format!("Failed to clone {}: {}", repo, stderr.trim()))
        }
        Err(e) => {
            let _ = std::fs::remove_dir_all(&temp_dir);
            if e.kind() == std::io::ErrorKind::NotFound {
                Err("Git is not installed. Install Git to scan GitHub repositories.".to_string())
            } else {
                Err(format!("Failed to run git: {}", e))
            }
        }
    }
}

#[tauri::command]
pub async fn start_scan(
    state: tauri::State<'_, AppDb>,
    project_id: String,
    on_progress: Channel<ScanProgress>,
) -> Result<ScanReport, String> {
    // Look up project
    let (project, scan_id) = {
        let conn = state.conn.lock().map_err(|e| e.to_string())?;
        let project = Project::get_by_id(&conn, &project_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Project not found: {}", project_id))?;

        let scan = Scan::create(&conn, &project_id).map_err(|e| e.to_string())?;
        (project, scan.id)
    };

    // Determine scan path based on source type
    let (scan_path, temp_dir) = if project.source_type == "github" {
        let repo = project.github_repo.as_deref().ok_or("GitHub repo not set")?;
        let branch = project.github_branch.as_deref().unwrap_or("main");
        let dir = clone_github_repo(repo, branch).await?;
        let path = dir.clone();
        (path, Some(dir))
    } else {
        (PathBuf::from(&project.repo_path), None)
    };

    // Run scanner
    let result = scanner::run_scan(&scan_path, |progress| {
        let _ = on_progress.send(progress);
    });

    // Clean up temp dir if we cloned
    if let Some(dir) = temp_dir {
        let _ = std::fs::remove_dir_all(dir);
    }

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
