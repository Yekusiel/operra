use crate::db::AppDb;
use crate::models::questionnaire::QuestionnaireResponse;
use crate::models::scan::{Scan, ScanFinding};
use crate::scanner::autofill::{autofill_from_findings, AutoFilledAnswers};

#[tauri::command]
pub async fn get_or_create_questionnaire(
    state: tauri::State<'_, AppDb>,
    project_id: String,
) -> Result<QuestionnaireResponse, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    QuestionnaireResponse::create_or_get(&conn, &project_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_questionnaire(
    state: tauri::State<'_, AppDb>,
    id: String,
    answers_json: String,
    completed: bool,
) -> Result<(), String> {
    // Validate JSON
    serde_json::from_str::<serde_json::Value>(&answers_json)
        .map_err(|e| format!("Invalid JSON: {}", e))?;

    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    QuestionnaireResponse::save_answers(&conn, &id, &answers_json, completed)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_questionnaire(
    state: tauri::State<'_, AppDb>,
    project_id: String,
) -> Result<Option<QuestionnaireResponse>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    QuestionnaireResponse::get_latest(&conn, &project_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn reset_questionnaire(
    state: tauri::State<'_, AppDb>,
    project_id: String,
) -> Result<QuestionnaireResponse, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    QuestionnaireResponse::delete_all_for_project(&conn, &project_id)
        .map_err(|e| e.to_string())?;
    QuestionnaireResponse::create_or_get(&conn, &project_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_autofill_suggestions(
    state: tauri::State<'_, AppDb>,
    project_id: String,
) -> Result<AutoFilledAnswers, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;

    // Get latest completed scan findings
    let scans = Scan::list_for_project(&conn, &project_id).map_err(|e| e.to_string())?;
    let latest_scan = scans.iter().find(|s| s.status == "completed");

    let findings = if let Some(scan) = latest_scan {
        ScanFinding::list_for_scan(&conn, &scan.id).map_err(|e| e.to_string())?
    } else {
        Vec::new()
    };

    Ok(autofill_from_findings(&findings))
}
