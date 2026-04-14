use crate::db::AppDb;
use crate::models::deployment::Deployment;
use crate::models::plan::Plan;
use crate::models::plan_option::PlanOption;
use crate::adapters::claude::ClaudeCliAdapter;
use crate::tools::tofu;
use std::path::{Path, PathBuf};

#[derive(serde::Serialize)]
pub struct IacGenerationResult {
    pub output_dir: String,
    pub files: Vec<String>,
}

#[tauri::command]
pub async fn generate_iac(
    state: tauri::State<'_, AppDb>,
    project_id: String,
    plan_id: String,
) -> Result<IacGenerationResult, String> {
    let (plan, approved_option, project) = {
        let conn = state.conn.lock().map_err(|e| e.to_string())?;
        let plan = Plan::get_by_id(&conn, &plan_id)
            .map_err(|e| e.to_string())?
            .ok_or("Plan not found")?;

        if !plan.approved {
            return Err("No plan has been approved. Review and approve a plan option first.".to_string());
        }

        let approved_option = PlanOption::get_approved(&conn, &plan_id)
            .map_err(|e| e.to_string())?;

        let project = crate::models::project::Project::get_by_id(&conn, &project_id)
            .map_err(|e| e.to_string())?
            .ok_or("Project not found")?;
        (plan, approved_option, project)
    };

    // Use the approved option's content if available, otherwise fall back to full plan
    let plan_markdown = if let Some(ref opt) = approved_option {
        opt.content.as_str()
    } else {
        plan.plan_markdown.as_deref().ok_or("Plan has no content")?
    };

    // Build prompt asking Claude to generate OpenTofu files
    let prompt = format!(
        r#"You are an infrastructure-as-code expert. Generate OpenTofu (Terraform-compatible) files based on this infrastructure plan.

## Project
- Name: {}
- AWS Region: {}
- AWS Profile: {}

## Infrastructure Plan
{}

## Instructions

Generate complete, working OpenTofu files. Output ONLY the file contents in this exact format for each file:

=== FILE: providers.tf ===
<file contents>

=== FILE: main.tf ===
<file contents>

=== FILE: variables.tf ===
<file contents>

=== FILE: outputs.tf ===
<file contents>

Also generate a terraform.tfvars file with sensible default values for all variables:

=== FILE: terraform.tfvars ===
<variable values>

Rules:
- Use the `aws` provider with the specified region
- Use realistic, production-ready configurations
- Include proper tagging (Project, ManagedBy=Operra)
- Use variables for anything that should be configurable
- EVERY variable MUST have a default value in variables.tf OR a value in terraform.tfvars
- Include outputs for important values (endpoints, ARNs, etc.)
- Add comments explaining non-obvious choices
- Do NOT include any markdown formatting, explanations, or text outside the === FILE: === blocks
"#,
        project.name,
        project.aws_region,
        project.aws_profile.as_deref().unwrap_or("default"),
        plan_markdown,
    );

    let adapter = ClaudeCliAdapter::default_path();
    let response = adapter
        .invoke_plan(&prompt)
        .await
        .map_err(|e| e.to_string())?;

    // Parse the response into files
    let infra_dir = PathBuf::from(&project.repo_path).join("infrastructure");
    std::fs::create_dir_all(&infra_dir)
        .map_err(|e| format!("Failed to create infrastructure directory: {}", e))?;

    let files = parse_and_write_files(&response, &infra_dir)?;

    if files.is_empty() {
        return Err("AI did not generate any infrastructure files. Try regenerating.".to_string());
    }

    Ok(IacGenerationResult {
        output_dir: infra_dir.to_string_lossy().to_string(),
        files,
    })
}

fn parse_and_write_files(response: &str, output_dir: &Path) -> Result<Vec<String>, String> {
    let mut files = Vec::new();
    let mut current_file: Option<String> = None;
    let mut current_content = String::new();

    for line in response.lines() {
        if line.starts_with("=== FILE:") && line.ends_with("===") {
            // Save previous file
            if let Some(ref filename) = current_file {
                write_iac_file(output_dir, filename, &current_content)?;
                files.push(filename.clone());
            }
            // Start new file
            let filename = line
                .trim_start_matches("=== FILE:")
                .trim_end_matches("===")
                .trim()
                .to_string();
            current_file = Some(filename);
            current_content = String::new();
        } else if current_file.is_some() {
            current_content.push_str(line);
            current_content.push('\n');
        }
    }

    // Save last file
    if let Some(ref filename) = current_file {
        write_iac_file(output_dir, filename, &current_content)?;
        files.push(filename.clone());
    }

    Ok(files)
}

fn write_iac_file(output_dir: &Path, filename: &str, content: &str) -> Result<(), String> {
    // Sanitize filename -- only allow alphanumeric, dots, hyphens, underscores
    let safe_name: String = filename
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '.' || *c == '-' || *c == '_')
        .collect();

    if safe_name.is_empty() || (!safe_name.ends_with(".tf") && !safe_name.ends_with(".tfvars")) {
        return Ok(()); // Skip non-terraform files
    }

    let path = output_dir.join(&safe_name);
    let trimmed = content.trim();
    if !trimmed.is_empty() {
        std::fs::write(&path, trimmed)
            .map_err(|e| format!("Failed to write {}: {}", safe_name, e))?;
    }
    Ok(())
}

// ── Deployment Commands ──

#[tauri::command]
pub async fn run_tofu_plan(
    state: tauri::State<'_, AppDb>,
    project_id: String,
) -> Result<Deployment, String> {
    let project = {
        let conn = state.conn.lock().map_err(|e| e.to_string())?;
        crate::models::project::Project::get_by_id(&conn, &project_id)
            .map_err(|e| e.to_string())?
            .ok_or("Project not found")?
    };

    let infra_dir = PathBuf::from(&project.repo_path).join("infrastructure");
    if !infra_dir.exists() {
        return Err("No infrastructure directory found. Generate IaC first.".to_string());
    }

    // Create deployment record
    let deployment = {
        let conn = state.conn.lock().map_err(|e| e.to_string())?;
        Deployment::create(&conn, &project_id, None, "plan")
            .map_err(|e| e.to_string())?
    };

    // Run tofu init
    let init_result = tofu::init(&infra_dir).await?;
    if !init_result.success {
        let conn = state.conn.lock().map_err(|e| e.to_string())?;
        let err = format!("tofu init failed:\n{}\n{}", init_result.stdout, init_result.stderr);
        Deployment::fail(&conn, &deployment.id, &err).map_err(|e| e.to_string())?;
        return Err(err);
    }

    // Run tofu plan
    let plan_result = tofu::plan(&infra_dir).await?;
    let combined_output = format!("{}\n{}", plan_result.stdout, plan_result.stderr);
    let summary = tofu::parse_plan_output(&combined_output);

    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    if plan_result.success {
        Deployment::save_plan_output(
            &conn,
            &deployment.id,
            &combined_output,
            &format!(
                "{} to create, {} to update, {} to destroy",
                summary.to_create, summary.to_update, summary.to_destroy
            ),
            &summary.risk_level,
        )
        .map_err(|e| e.to_string())?;
    } else {
        let err = format!("tofu plan failed:\n{}", combined_output);
        Deployment::fail(&conn, &deployment.id, &err).map_err(|e| e.to_string())?;
    }

    Deployment::get_by_id(&conn, &deployment.id)
        .map_err(|e| e.to_string())?
        .ok_or("Deployment disappeared".to_string())
}

#[tauri::command]
pub async fn approve_deployment(
    state: tauri::State<'_, AppDb>,
    deployment_id: String,
) -> Result<Deployment, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    Deployment::approve(&conn, &deployment_id).map_err(|e| e.to_string())?;
    Deployment::get_by_id(&conn, &deployment_id)
        .map_err(|e| e.to_string())?
        .ok_or("Deployment not found".to_string())
}

#[tauri::command]
pub async fn run_tofu_apply(
    state: tauri::State<'_, AppDb>,
    deployment_id: String,
) -> Result<Deployment, String> {
    let (deployment, project) = {
        let conn = state.conn.lock().map_err(|e| e.to_string())?;
        let deployment = Deployment::get_by_id(&conn, &deployment_id)
            .map_err(|e| e.to_string())?
            .ok_or("Deployment not found")?;

        if !deployment.approved {
            return Err("Deployment has not been approved. Approve it first.".to_string());
        }

        let project = crate::models::project::Project::get_by_id(&conn, &deployment.project_id)
            .map_err(|e| e.to_string())?
            .ok_or("Project not found")?;

        Deployment::start_apply(&conn, &deployment_id).map_err(|e| e.to_string())?;
        (deployment, project)
    };

    let infra_dir = PathBuf::from(&project.repo_path).join("infrastructure");
    let apply_result = tofu::apply(&infra_dir).await?;
    let combined = format!("{}\n{}", apply_result.stdout, apply_result.stderr);

    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    if apply_result.success {
        Deployment::complete(&conn, &deployment.id, &combined, None)
            .map_err(|e| e.to_string())?;
    } else {
        Deployment::fail(&conn, &deployment.id, &combined)
            .map_err(|e| e.to_string())?;
    }

    Deployment::get_by_id(&conn, &deployment.id)
        .map_err(|e| e.to_string())?
        .ok_or("Deployment disappeared".to_string())
}

#[tauri::command]
pub async fn get_deployment(
    state: tauri::State<'_, AppDb>,
    deployment_id: String,
) -> Result<Option<Deployment>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    Deployment::get_by_id(&conn, &deployment_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_deployments(
    state: tauri::State<'_, AppDb>,
    project_id: String,
) -> Result<Vec<Deployment>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    Deployment::list_for_project(&conn, &project_id).map_err(|e| e.to_string())
}
