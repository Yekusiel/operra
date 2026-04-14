use crate::db::AppDb;
use crate::models::adapter_log::AdapterLog;
use crate::models::plan::Plan;
use crate::models::plan_message::PlanMessage;
use crate::models::plan_option::PlanOption;
use crate::models::project::Project;
use crate::models::questionnaire::QuestionnaireResponse;
use crate::models::scan::{Scan, ScanFinding};
use crate::adapters::claude::ClaudeCliAdapter;
use std::time::Instant;

#[derive(serde::Serialize)]
pub struct PlanGenerationResult {
    pub plan: Plan,
    pub options: Vec<PlanOption>,
}

/// Generate a plan session: creates the plan record, then generates 3 options sequentially.
/// Each option is generated with the previous options as context so they're meaningfully different.
#[tauri::command]
pub async fn generate_plan(
    state: tauri::State<'_, AppDb>,
    project_id: String,
) -> Result<PlanGenerationResult, String> {
    // Gather project context
    let (project, scan_findings, questionnaire, latest_scan_id, questionnaire_id) = {
        let conn = state.conn.lock().map_err(|e| e.to_string())?;

        let project = Project::get_by_id(&conn, &project_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Project not found: {}", project_id))?;

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

    let base_context = build_project_context(&project, &scan_findings, &questionnaire);

    // Create plan record
    let plan = {
        let conn = state.conn.lock().map_err(|e| e.to_string())?;
        Plan::create(
            &conn,
            &project_id,
            latest_scan_id.as_deref(),
            questionnaire_id.as_deref(),
        )
        .map_err(|e| e.to_string())?
    };

    let adapter = ClaudeCliAdapter::default_path();
    let mut generated_options: Vec<PlanOption> = Vec::new();

    // Generate 3 plans sequentially, each with previous plans as context
    for i in 0..3 {
        let label = ((b'A' + i as u8) as char).to_string();
        let prompt = build_single_plan_prompt(&base_context, &generated_options, &label);

        let start = Instant::now();
        match adapter.invoke_plan(&prompt).await {
            Ok(response) => {
                let duration = start.elapsed().as_millis() as i64;
                let conn = state.conn.lock().map_err(|e| e.to_string())?;

                // Extract a title from the first line of the response
                let title = extract_title_from_response(&response, &label);

                let option = PlanOption::create(
                    &conn, &plan.id, &label, &title, &response, "generation", None,
                )
                .map_err(|e| e.to_string())?;

                // Log it
                let _ = AdapterLog::create(&conn, &project_id, "claude-code", "plan_option", &prompt)
                    .and_then(|log| AdapterLog::complete(&conn, &log.id, &response, None, duration));

                generated_options.push(option);
            }
            Err(err) => {
                // If the first plan fails, fail the whole session.
                // If B or C fail, keep what we have.
                if i == 0 {
                    let conn = state.conn.lock().map_err(|e| e.to_string())?;
                    Plan::fail(&conn, &plan.id, &err.to_string()).map_err(|e| e.to_string())?;
                    return Err(err.to_string());
                }
                break;
            }
        }
    }

    // Mark plan as completed
    {
        let conn = state.conn.lock().map_err(|e| e.to_string())?;
        // Store a summary as the plan markdown
        let summary = generated_options
            .iter()
            .map(|o| format!("**Plan {}**: {}", o.label, o.title))
            .collect::<Vec<_>>()
            .join("\n");
        Plan::complete(&conn, &plan.id, &summary, None, None, None)
            .map_err(|e| e.to_string())?;
    }

    let updated_plan = {
        let conn = state.conn.lock().map_err(|e| e.to_string())?;
        Plan::get_by_id(&conn, &plan.id)
            .map_err(|e| e.to_string())?
            .ok_or("Plan disappeared")?
    };

    Ok(PlanGenerationResult {
        plan: updated_plan,
        options: generated_options,
    })
}

/// Generate a single additional plan option (e.g., when user asks for a Plan D in chat).
#[tauri::command]
pub async fn generate_additional_option(
    state: tauri::State<'_, AppDb>,
    plan_id: String,
    user_request: Option<String>,
) -> Result<PlanOption, String> {
    let (plan, existing_options, project, scan_findings, questionnaire) = {
        let conn = state.conn.lock().map_err(|e| e.to_string())?;
        let plan = Plan::get_by_id(&conn, &plan_id)
            .map_err(|e| e.to_string())?
            .ok_or("Plan not found")?;
        let options = PlanOption::list_for_plan(&conn, &plan_id)
            .map_err(|e| e.to_string())?;
        let project = Project::get_by_id(&conn, &plan.project_id)
            .map_err(|e| e.to_string())?
            .ok_or("Project not found")?;

        let scans = Scan::list_for_project(&conn, &plan.project_id)
            .map_err(|e| e.to_string())?;
        let latest_scan = scans.iter().find(|s| s.status == "completed");
        let scan_findings = if let Some(scan) = latest_scan {
            ScanFinding::list_for_scan(&conn, &scan.id).map_err(|e| e.to_string())?
        } else {
            Vec::new()
        };
        let questionnaire = QuestionnaireResponse::get_latest(&conn, &plan.project_id)
            .map_err(|e| e.to_string())?;

        (plan, options, project, scan_findings, questionnaire)
    };

    let base_context = build_project_context(&project, &scan_findings, &questionnaire);
    let next_label = PlanOption::next_label(
        &state.conn.lock().map_err(|e| e.to_string())?,
        &plan_id,
    )
    .map_err(|e| e.to_string())?;

    let mut prompt = build_single_plan_prompt(&base_context, &existing_options, &next_label);
    if let Some(ref req) = user_request {
        prompt.push_str(&format!(
            "\n\nThe user specifically asked for: {}\nTailor this plan option to their request.\n",
            req
        ));
    }

    let adapter = ClaudeCliAdapter::default_path();
    let start = Instant::now();
    let response = adapter.invoke_plan(&prompt).await.map_err(|e| e.to_string())?;
    let duration = start.elapsed().as_millis() as i64;

    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let title = extract_title_from_response(&response, &next_label);
    let option = PlanOption::create(
        &conn, &plan_id, &next_label, &title, &response, "chat", None,
    )
    .map_err(|e| e.to_string())?;

    let _ = AdapterLog::create(&conn, &plan.project_id, "claude-code", "plan_option", &prompt)
        .and_then(|log| AdapterLog::complete(&conn, &log.id, &response, None, duration));

    Ok(option)
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

// ── Plan Chat ──

#[tauri::command]
pub async fn send_plan_message(
    state: tauri::State<'_, AppDb>,
    plan_id: String,
    message: String,
) -> Result<PlanMessage, String> {
    let (plan, history, options) = {
        let conn = state.conn.lock().map_err(|e| e.to_string())?;
        let plan = Plan::get_by_id(&conn, &plan_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Plan not found: {}", plan_id))?;
        let history = PlanMessage::list_for_plan(&conn, &plan_id)
            .map_err(|e| e.to_string())?;
        let options = PlanOption::list_for_plan(&conn, &plan_id)
            .map_err(|e| e.to_string())?;
        (plan, history, options)
    };

    // Save user message
    {
        let conn = state.conn.lock().map_err(|e| e.to_string())?;
        PlanMessage::create(&conn, &plan_id, "user", &message)
            .map_err(|e| e.to_string())?;
    }

    // Build chat prompt (no parsing of response for plans -- chat is just chat)
    let prompt = build_chat_prompt(&options, &history, &message);

    let adapter = ClaudeCliAdapter::default_path();
    let start = Instant::now();

    match adapter.invoke_plan(&prompt).await {
        Ok(response) => {
            let duration = start.elapsed().as_millis() as i64;
            let conn = state.conn.lock().map_err(|e| e.to_string())?;

            let assistant_msg = PlanMessage::create(&conn, &plan_id, "assistant", &response)
                .map_err(|e| e.to_string())?;

            let _ = AdapterLog::create(&conn, &plan.project_id, "claude-code", "plan_chat", &prompt)
                .and_then(|log| AdapterLog::complete(&conn, &log.id, &response, None, duration));

            Ok(assistant_msg)
        }
        Err(err) => Err(err.to_string()),
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

// ── Prompt Builders ──

fn build_project_context(
    project: &Project,
    scan_findings: &[ScanFinding],
    questionnaire: &Option<QuestionnaireResponse>,
) -> String {
    let mut ctx = String::new();

    ctx.push_str(&format!("## Project: {}\n", project.name));
    ctx.push_str(&format!("- Repository: {}\n", project.repo_path));
    ctx.push_str(&format!("- AWS Region: {}\n", project.aws_region));
    if let Some(ref profile) = project.aws_profile {
        ctx.push_str(&format!("- AWS Profile: {}\n", profile));
    }
    ctx.push('\n');

    if !scan_findings.is_empty() {
        ctx.push_str("## Detected Technologies\n");
        let categories = ["language", "framework", "database", "queue", "infrastructure", "config", "ci_cd"];
        for cat in &categories {
            let findings: Vec<&ScanFinding> = scan_findings.iter().filter(|f| f.category == *cat).collect();
            if !findings.is_empty() {
                let label = match *cat {
                    "language" => "Languages",
                    "framework" => "Frameworks",
                    "database" => "Databases",
                    "queue" => "Queues & Background Jobs",
                    "infrastructure" => "Existing Infrastructure",
                    "config" => "Configuration",
                    "ci_cd" => "CI/CD",
                    _ => cat,
                };
                ctx.push_str(&format!("\n### {}\n", label));
                for f in findings {
                    ctx.push_str(&format!("- {} (confidence: {}%, evidence: {})\n",
                        f.name,
                        (f.confidence * 100.0) as i32,
                        f.evidence_path.as_deref().unwrap_or("n/a"),
                    ));
                }
            }
        }
        ctx.push('\n');
    }

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
                ctx.push_str("## Architecture Requirements\n");
                for line in &explicit {
                    ctx.push_str(line);
                    ctx.push('\n');
                }
                ctx.push('\n');
            }

            if !infer.is_empty() {
                ctx.push_str("## Decisions to Infer\nUse detected technologies to decide:\n");
                for item in &infer {
                    ctx.push_str(&format!("- {}\n", item));
                }
                ctx.push('\n');
            }
        }
    }

    ctx
}

fn build_single_plan_prompt(
    base_context: &str,
    previous_options: &[PlanOption],
    label: &str,
) -> String {
    let mut prompt = String::new();

    prompt.push_str("You are an AWS infrastructure architect. Generate ONE infrastructure plan option.\n\n");
    prompt.push_str(base_context);

    if !previous_options.is_empty() {
        prompt.push_str("## Previously Generated Plans (DO NOT repeat these -- propose something meaningfully different)\n\n");
        for opt in previous_options {
            prompt.push_str(&format!("### Plan {} - {}\n{}\n\n", opt.label, opt.title, opt.content));
        }
    }

    prompt.push_str(&format!(
        r#"## Your Task

Generate Plan {label}. This must be a SINGLE, complete infrastructure plan.

{context}

Include in your response:
- A clear title for this approach (first line, e.g., "ECS Fargate with RDS")
- Which AWS services are used and how they connect
- Why this approach fits the project
- Estimated monthly cost range
- Deployment complexity: LOW, MEDIUM, or HIGH
- Key tradeoffs (pros and cons)
- Any prerequisites

Write in clear markdown. Do NOT include "Plan {label}:" in your response -- the app adds the label automatically. Just start with the title and go.
"#,
        context = if previous_options.is_empty() {
            "This should be the RECOMMENDED option -- the best overall fit for this project."
        } else if previous_options.len() == 1 {
            "This should be a meaningfully DIFFERENT alternative. For example, if Plan A uses containers, consider serverless. If Plan A prioritizes simplicity, consider scalability."
        } else {
            "This should be a THIRD distinct approach. Consider a different cost/complexity tradeoff than the previous plans."
        },
    ));

    prompt
}

fn build_chat_prompt(
    options: &[PlanOption],
    history: &[PlanMessage],
    new_message: &str,
) -> String {
    let mut prompt = String::new();

    prompt.push_str("You are an AWS infrastructure architect discussing infrastructure plans with a user.\n\n");

    if !options.is_empty() {
        prompt.push_str("## The Plan Options\n\n");
        for opt in options {
            let status = if opt.approved { " [APPROVED]" } else { "" };
            prompt.push_str(&format!("### Plan {}{} - {}\n{}\n\n", opt.label, status, opt.title, opt.content));
        }
    }

    if !history.is_empty() {
        prompt.push_str("## Conversation So Far\n\n");
        for msg in history {
            let role_label = if msg.role == "user" { "User" } else { "You" };
            prompt.push_str(&format!("**{}**: {}\n\n", role_label, msg.content));
        }
    }

    prompt.push_str(&format!("## User's Message\n\n{}\n\n", new_message));
    prompt.push_str("Answer the user's question about the plans. If they want a new plan option, tell them to click the 'Add Another Option' button. Keep responses focused and practical.\n");

    prompt
}

/// Extract a title from the first meaningful line of the AI response.
fn extract_title_from_response(response: &str, label: &str) -> String {
    for line in response.lines() {
        let trimmed = line.trim()
            .trim_start_matches('#')
            .trim()
            .trim_start_matches("**")
            .trim_end_matches("**")
            .trim();

        // Skip empty lines and lines that are just the plan label
        if trimmed.is_empty() {
            continue;
        }

        // If the AI included "Plan X:" despite being told not to, strip it
        let cleaned = if trimmed.to_lowercase().starts_with("plan ") {
            let rest = &trimmed[5..];
            if rest.starts_with(|c: char| c.is_ascii_uppercase()) {
                rest[1..].trim().trim_start_matches(':').trim_start_matches('-').trim()
            } else {
                trimmed
            }
        } else {
            trimmed
        };

        if !cleaned.is_empty() {
            return cleaned.to_string();
        }
    }

    format!("Plan {}", label)
}
