use crate::db::AppDb;
use crate::models::adapter_log::AdapterLog;
use crate::models::plan::Plan;
use crate::models::project::Project;
use crate::models::questionnaire::QuestionnaireResponse;
use crate::models::scan::{Scan, ScanFinding};
use crate::adapters::claude::ClaudeCliAdapter;
use std::time::Instant;

#[derive(serde::Serialize)]
pub struct PlanGenerationResult {
    pub plan: Plan,
    pub adapter_log_id: String,
}

#[tauri::command]
pub async fn generate_plan(
    state: tauri::State<'_, AppDb>,
    project_id: String,
) -> Result<PlanGenerationResult, String> {
    // Gather all context
    let (project, scan_findings, questionnaire, latest_scan_id, questionnaire_id) = {
        let conn = state.conn.lock().map_err(|e| e.to_string())?;

        let project = Project::get_by_id(&conn, &project_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Project not found: {}", project_id))?;

        // Get latest completed scan
        let scans = Scan::list_for_project(&conn, &project_id)
            .map_err(|e| e.to_string())?;
        let latest_scan = scans.iter().find(|s| s.status == "completed");

        let scan_findings = if let Some(scan) = latest_scan {
            ScanFinding::list_for_scan(&conn, &scan.id).map_err(|e| e.to_string())?
        } else {
            Vec::new()
        };

        let latest_scan_id = latest_scan.map(|s| s.id.clone());

        let questionnaire = QuestionnaireResponse::get_latest(&conn, &project_id)
            .map_err(|e| e.to_string())?;
        let questionnaire_id = questionnaire.as_ref().map(|q| q.id.clone());

        (project, scan_findings, questionnaire, latest_scan_id, questionnaire_id)
    };

    // Build the prompt
    let prompt = build_plan_prompt(&project, &scan_findings, &questionnaire);

    // Create plan and adapter log records
    let (plan, log_id) = {
        let conn = state.conn.lock().map_err(|e| e.to_string())?;
        let plan = Plan::create(
            &conn,
            &project_id,
            latest_scan_id.as_deref(),
            questionnaire_id.as_deref(),
        )
        .map_err(|e| e.to_string())?;

        let log = AdapterLog::create(&conn, &project_id, "claude-code", "architecture_plan", &prompt)
            .map_err(|e| e.to_string())?;

        (plan, log.id)
    };

    // Invoke Claude Code CLI
    let start = Instant::now();
    let adapter = ClaudeCliAdapter::default_path();

    match adapter.invoke_plan(&prompt).await {
        Ok(response) => {
            let duration = start.elapsed().as_millis() as i64;
            let conn = state.conn.lock().map_err(|e| e.to_string())?;

            // Parse structured sections from response
            let (markdown, alternatives, cost_notes) = parse_plan_response(&response);

            Plan::complete(
                &conn,
                &plan.id,
                &markdown,
                None,
                alternatives.as_deref(),
                cost_notes.as_deref(),
            )
            .map_err(|e| e.to_string())?;

            AdapterLog::complete(&conn, &log_id, &response, None, duration)
                .map_err(|e| e.to_string())?;

            let updated_plan = Plan::get_by_id(&conn, &plan.id)
                .map_err(|e| e.to_string())?
                .ok_or("Plan disappeared after update")?;

            Ok(PlanGenerationResult {
                plan: updated_plan,
                adapter_log_id: log_id,
            })
        }
        Err(err) => {
            let duration = start.elapsed().as_millis() as i64;
            let err_str = err.to_string();
            let conn = state.conn.lock().map_err(|e| e.to_string())?;

            Plan::fail(&conn, &plan.id, &err_str).map_err(|e| e.to_string())?;
            AdapterLog::fail(&conn, &log_id, &err_str, duration).map_err(|e| e.to_string())?;

            Err(err_str)
        }
    }
}

#[tauri::command]
pub async fn get_plan(
    state: tauri::State<'_, AppDb>,
    plan_id: String,
) -> Result<Option<Plan>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    Plan::get_by_id(&conn, &plan_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_latest_plan(
    state: tauri::State<'_, AppDb>,
    project_id: String,
) -> Result<Option<Plan>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    Plan::get_latest(&conn, &project_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_plans(
    state: tauri::State<'_, AppDb>,
    project_id: String,
) -> Result<Vec<Plan>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    Plan::list_for_project(&conn, &project_id).map_err(|e| e.to_string())
}

fn build_plan_prompt(
    project: &Project,
    scan_findings: &[ScanFinding],
    questionnaire: &Option<QuestionnaireResponse>,
) -> String {
    let mut prompt = String::new();

    prompt.push_str("You are an AWS infrastructure architect. Generate a detailed infrastructure plan based on the following project analysis.\n\n");

    // Project context
    prompt.push_str(&format!("## Project: {}\n", project.name));
    prompt.push_str(&format!("- Repository: {}\n", project.repo_path));
    prompt.push_str(&format!("- AWS Region: {}\n", project.aws_region));
    if let Some(ref profile) = project.aws_profile {
        prompt.push_str(&format!("- AWS Profile: {}\n", profile));
    }
    prompt.push_str("\n");

    // Scan findings
    if !scan_findings.is_empty() {
        prompt.push_str("## Detected Technologies\n");
        let categories = ["language", "framework", "infrastructure", "config", "ci_cd"];
        for cat in &categories {
            let findings: Vec<&ScanFinding> = scan_findings.iter().filter(|f| f.category == *cat).collect();
            if !findings.is_empty() {
                let label = match *cat {
                    "language" => "Languages",
                    "framework" => "Frameworks",
                    "infrastructure" => "Existing Infrastructure",
                    "config" => "Configuration",
                    "ci_cd" => "CI/CD",
                    _ => cat,
                };
                prompt.push_str(&format!("\n### {}\n", label));
                for f in findings {
                    prompt.push_str(&format!("- {} (confidence: {}%, evidence: {})\n",
                        f.name,
                        (f.confidence * 100.0) as i32,
                        f.evidence_path.as_deref().unwrap_or("n/a"),
                    ));
                }
            }
        }
        prompt.push_str("\n");
    }

    // Questionnaire answers
    if let Some(ref q) = questionnaire {
        if q.completed {
            prompt.push_str("## Architecture Requirements (User Answers)\n");
            if let Ok(answers) = serde_json::from_str::<serde_json::Value>(&q.answers_json) {
                if let Some(obj) = answers.as_object() {
                    for (key, value) in obj {
                        let label = key.replace('_', " ");
                        let label = label
                            .split_whitespace()
                            .map(|w| {
                                let mut c = w.chars();
                                match c.next() {
                                    None => String::new(),
                                    Some(f) => f.to_uppercase().to_string() + c.as_str(),
                                }
                            })
                            .collect::<Vec<_>>()
                            .join(" ");

                        if let Some(v) = value.as_str() {
                            if !v.is_empty() {
                                prompt.push_str(&format!("- {}: {}\n", label, v));
                            }
                        }
                    }
                }
            }
            prompt.push_str("\n");
        }
    }

    // Instructions for output format
    prompt.push_str(r#"## Instructions

Generate a complete AWS infrastructure plan. Structure your response with these exact sections:

### Recommended Architecture
Describe the proposed AWS architecture in detail. Include specific AWS services, how they connect, and why each was chosen.

### Why This Architecture
Explain the reasoning behind the choices. Reference the detected technologies and user requirements.

### Alternatives
List 2-3 alternative approaches with brief tradeoffs. Format each as:
- **Alternative Name**: Description. *Tradeoff: explanation*

### Cost Notes
Provide rough cost-aware notes. Include:
- Estimated monthly cost range for the recommended architecture
- Which services are the main cost drivers
- Cost optimization tips specific to this setup

### Deployment Prerequisites
List what needs to be in place before deployment:
- AWS account setup requirements
- IAM roles/policies needed
- DNS/domain requirements
- SSL certificate needs
- Any manual steps required

### Risk Assessment
Rate the deployment complexity as LOW, MEDIUM, or HIGH and explain why.

Use clear, practical language. This plan will be shown to a developer who needs to understand exactly what will be deployed and why.
"#);

    prompt
}

fn parse_plan_response(response: &str) -> (String, Option<String>, Option<String>) {
    let markdown = response.to_string();

    // Extract alternatives section
    let alternatives = extract_section(response, "### Alternatives", &["### Cost", "### Deployment", "### Risk"]);

    // Extract cost notes section
    let cost_notes = extract_section(response, "### Cost Notes", &["### Deployment", "### Risk"]);

    (markdown, alternatives, cost_notes)
}

fn extract_section(text: &str, header: &str, end_markers: &[&str]) -> Option<String> {
    let start = text.find(header)?;
    let content_start = start + header.len();
    let remaining = &text[content_start..];

    let end = end_markers
        .iter()
        .filter_map(|marker| remaining.find(marker))
        .min()
        .unwrap_or(remaining.len());

    let section = remaining[..end].trim();
    if section.is_empty() {
        None
    } else {
        Some(section.to_string())
    }
}
