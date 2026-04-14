use crate::db::AppDb;
use crate::models::adapter_log::AdapterLog;
use crate::models::plan::Plan;
use crate::models::plan_message::PlanMessage;
use crate::models::plan_option::{self, PlanOption};
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

            // Store full response as plan markdown
            Plan::complete(
                &conn,
                &plan.id,
                &response,
                None,
                None,
                None,
            )
            .map_err(|e| e.to_string())?;

            // Parse and store individual plan options (Plan A, Plan B, etc.)
            let parsed_options = plan_option::parse_plan_options(&response);
            for (label, title, content) in &parsed_options {
                PlanOption::create(
                    &conn, &plan.id, label, title, content, "generation", None,
                )
                .map_err(|e| e.to_string())?;
            }

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

#[tauri::command]
pub async fn approve_plan(
    state: tauri::State<'_, AppDb>,
    plan_id: String,
) -> Result<Plan, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    Plan::approve(&conn, &plan_id).map_err(|e| e.to_string())?;
    Plan::get_by_id(&conn, &plan_id)
        .map_err(|e| e.to_string())?
        .ok_or("Plan not found after approval".to_string())
}

#[tauri::command]
pub async fn get_approved_plan(
    state: tauri::State<'_, AppDb>,
    project_id: String,
) -> Result<Option<Plan>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    Plan::get_approved_for_project(&conn, &project_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_plan_options(
    state: tauri::State<'_, AppDb>,
    plan_id: String,
) -> Result<Vec<PlanOption>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    PlanOption::list_for_plan(&conn, &plan_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn approve_plan_option(
    state: tauri::State<'_, AppDb>,
    option_id: String,
) -> Result<PlanOption, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    PlanOption::approve(&conn, &option_id).map_err(|e| e.to_string())?;
    let plan_id: String = conn.query_row(
        "SELECT plan_id FROM plan_options WHERE id = ?1",
        rusqlite::params![option_id],
        |row| row.get(0),
    ).map_err(|e| e.to_string())?;
    PlanOption::get_approved(&conn, &plan_id)
        .map_err(|e| e.to_string())?
        .ok_or("Option not found after approval".to_string())
}

#[tauri::command]
pub async fn get_approved_option(
    state: tauri::State<'_, AppDb>,
    plan_id: String,
) -> Result<Option<PlanOption>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    PlanOption::get_approved(&conn, &plan_id).map_err(|e| e.to_string())
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
        let categories = ["language", "framework", "database", "queue", "infrastructure", "config", "ci_cd"];
        for cat in &categories {
            let findings: Vec<&ScanFinding> = scan_findings.iter().filter(|f| f.category == *cat).collect();
            if !findings.is_empty() {
                let label = match *cat {
                    "language" => "Languages",
                    "framework" => "Frameworks",
                    "infrastructure" => "Existing Infrastructure",
                    "config" => "Configuration",
                    "database" => "Databases",
                    "queue" => "Queues & Background Jobs",
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
            let mut explicit = Vec::new();
            let mut infer = Vec::new();

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
                            if v == "unknown" {
                                infer.push(label);
                            } else if !v.is_empty() {
                                explicit.push(format!("- {}: {}", label, v));
                            }
                        }
                    }
                }
            }

            if !explicit.is_empty() {
                prompt.push_str("## Architecture Requirements (User Answers)\n");
                for line in &explicit {
                    prompt.push_str(line);
                    prompt.push('\n');
                }
                prompt.push('\n');
            }

            if !infer.is_empty() {
                prompt.push_str("## Decisions to Infer\nThe user was not sure about the following. Use the detected technologies and codebase context to make the best decision and explain your reasoning:\n");
                for item in &infer {
                    prompt.push_str(&format!("- {}\n", item));
                }
                prompt.push('\n');
            }
        }
    }

    // Instructions for output format
    prompt.push_str(r#"## Instructions

Generate 2-3 infrastructure plan options for this project. The user will choose which one to approve.

IMPORTANT: Structure each plan as a separate section using this exact format:

## Plan A: [Short descriptive title]
[Full description of this architecture option including:]
- AWS services used and how they connect
- Why this approach fits the project
- Estimated monthly cost range
- Deployment complexity: LOW, MEDIUM, or HIGH
- Key tradeoffs

## Plan B: [Short descriptive title]
[Same structure as above for the alternative approach]

## Plan C: [Short descriptive title]
[Optional third option if meaningfully different]

Guidelines:
- Plan A should be the recommended/best-fit option
- Plan B should be a meaningfully different alternative (e.g., cheaper but with tradeoffs, or more scalable)
- Plan C is optional, only include if there's a genuinely different third approach
- Each plan must be self-contained with enough detail to implement
- Include cost estimates for each plan
- Be specific about AWS services, not vague
- Use clear, practical language
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

// ── Plan Chat ──

#[tauri::command]
pub async fn send_plan_message(
    state: tauri::State<'_, AppDb>,
    plan_id: String,
    message: String,
) -> Result<PlanMessage, String> {
    // Get plan, conversation history, and existing option count
    let (plan, history, next_label) = {
        let conn = state.conn.lock().map_err(|e| e.to_string())?;
        let plan = Plan::get_by_id(&conn, &plan_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Plan not found: {}", plan_id))?;
        let history = PlanMessage::list_for_plan(&conn, &plan_id)
            .map_err(|e| e.to_string())?;
        let next_label = PlanOption::next_label(&conn, &plan_id)
            .map_err(|e| e.to_string())?;
        (plan, history, next_label)
    };

    // Save user message
    {
        let conn = state.conn.lock().map_err(|e| e.to_string())?;
        PlanMessage::create(&conn, &plan_id, "user", &message)
            .map_err(|e| e.to_string())?;
    }

    // Build the conversation prompt
    let prompt = build_chat_prompt(&plan, &history, &message, &next_label);

    // Invoke Claude
    let adapter = ClaudeCliAdapter::default_path();
    let start = Instant::now();

    match adapter.invoke_plan(&prompt).await {
        Ok(response) => {
            let duration = start.elapsed().as_millis() as i64;
            let conn = state.conn.lock().map_err(|e| e.to_string())?;

            let assistant_msg = PlanMessage::create(&conn, &plan_id, "assistant", &response)
                .map_err(|e| e.to_string())?;

            // Check if the response contains new plan options
            let new_options = plan_option::parse_plan_options(&response);
            if !new_options.is_empty() {
                for (_label, title, content) in &new_options {
                    // Use next available label to avoid duplicates
                    let actual_label = PlanOption::next_label(&conn, &plan_id)
                        .map_err(|e| e.to_string())?;
                    PlanOption::create(
                        &conn, &plan_id, &actual_label, title, content,
                        "chat", Some(&assistant_msg.id),
                    )
                    .map_err(|e| e.to_string())?;
                }
            }

            // Log the interaction
            let _ = AdapterLog::create(&conn, &plan.project_id, "claude-code", "plan_chat", &prompt)
                .and_then(|log| AdapterLog::complete(&conn, &log.id, &response, None, duration));

            Ok(assistant_msg)
        }
        Err(err) => {
            let err_str = err.to_string();
            Err(err_str)
        }
    }
}

#[tauri::command]
pub async fn get_plan_messages(
    state: tauri::State<'_, AppDb>,
    plan_id: String,
) -> Result<Vec<PlanMessage>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    PlanMessage::list_for_plan(&conn, &plan_id).map_err(|e| e.to_string())
}

fn build_chat_prompt(plan: &Plan, history: &[PlanMessage], new_message: &str, next_label: &str) -> String {
    let mut prompt = String::new();

    prompt.push_str("You are an AWS infrastructure architect continuing a conversation about an infrastructure plan you previously generated.\n\n");

    // Include the original plan as context
    if let Some(ref md) = plan.plan_markdown {
        prompt.push_str("## The Infrastructure Plan You Generated\n\n");
        prompt.push_str(md);
        prompt.push_str("\n\n");
    }

    // Include conversation history
    if !history.is_empty() {
        prompt.push_str("## Conversation So Far\n\n");
        for msg in history {
            let role_label = if msg.role == "user" { "User" } else { "You" };
            prompt.push_str(&format!("**{}**: {}\n\n", role_label, msg.content));
        }
    }

    // The new user message
    prompt.push_str(&format!("## User's New Message\n\n{}\n\n", new_message));

    prompt.push_str(&format!(r#"Respond helpfully. If the user asks to change a plan or suggests a new approach, present it as a new plan option using EXACTLY this format:

## Plan {next_label}: [Short descriptive title]
[Full description with AWS services, cost estimate, complexity, tradeoffs]

This heading format is parsed automatically — the option will appear with its own approve button in the UI. The "## Plan" prefix and uppercase letter are required.

If the user asks questions or wants clarification, answer clearly WITHOUT creating a new plan section. Only use the "## Plan X:" format when presenting a genuinely new or revised infrastructure option. Keep using markdown formatting.
"#));

    prompt
}
