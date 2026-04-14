use crate::db::AppDb;
use crate::models::deployment::Deployment;
use crate::models::plan::Plan;
use crate::models::plan_option::PlanOption;
use crate::adapters::claude::ClaudeCliAdapter;
use crate::tools::aws_resources;
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

    // Build the valid resources list based on what the plan mentions
    let valid_resources = aws_resources::format_valid_resources(plan_markdown);

    // Source info for provisioning
    let source_info = if project.source_type == "github" {
        format!(
            "- Source: GitHub repository {}, branch {}",
            project.github_repo.as_deref().unwrap_or("unknown"),
            project.github_branch.as_deref().unwrap_or("main"),
        )
    } else {
        format!("- Source: Local directory {}", project.repo_path)
    };

    let domain_info = match project.domain.as_deref() {
        Some(d) if !d.is_empty() => format!("- Custom Domain: {}", d),
        _ => "- Custom Domain: None (use IP address)".to_string(),
    };

    // Build prompt asking Claude to generate OpenTofu files
    let prompt = format!(
        r#"You are an infrastructure-as-code expert. Generate OpenTofu (Terraform-compatible) files based on this infrastructure plan.

## Project
- Name: {name}
- AWS Region: {region}
- AWS Profile: {profile}
{source_info}
{domain_info}

## Infrastructure Plan
{plan}

{resources}

## Instructions

Generate complete, working OpenTofu files that PROVISION THE APPLICATION -- not just create empty infrastructure.

Output ONLY the file contents in this exact format for each file:

=== FILE: providers.tf ===
<file contents>

=== FILE: main.tf ===
<file contents>

=== FILE: variables.tf ===
<file contents>

=== FILE: outputs.tf ===
<file contents>

=== FILE: terraform.tfvars ===
<variable values>

CRITICAL -- Application Provisioning:
- The instance user_data MUST include a complete setup script that:
  1. Installs required runtime (Node.js, Docker, etc.) based on the project type
  2. {clone_instruction}
  3. Installs dependencies (npm install, pip install, etc.)
  4. Builds the application if needed (npm run build, etc.)
  5. Sets up a process manager (PM2 for Node.js, systemd for others) to keep the app running
  6. Configures a reverse proxy (Caddy preferred -- auto-HTTPS, simpler than Nginx) on port 80/443
  7. The app should be accessible via HTTP immediately after provisioning
- The user_data script should be a complete bash script, not a skeleton

{domain_instructions}

Other Rules:
- ONLY use resource types from the Valid AWS Resource Types list above -- no exceptions
- Pin the AWS provider to a specific version (e.g., ~> 5.0)
- Include proper tagging (Project, ManagedBy=Operra)
- Use variables for anything that should be configurable
- EVERY variable MUST have a default value in variables.tf OR a value in terraform.tfvars
- Do NOT use placeholder values like "CHANGEME" -- use real working defaults or omit
- For SSH key pairs: use tls_private_key + aws key pair (let Terraform generate)
- For passwords: use random_password resource from the random provider
- Include outputs for: app URL, SSH command, static IP, and any credentials
- Do NOT include any markdown formatting or text outside the === FILE: === blocks
"#,
        name = project.name,
        region = project.aws_region,
        profile = project.aws_profile.as_deref().unwrap_or("default"),
        source_info = source_info,
        domain_info = domain_info,
        plan = plan_markdown,
        resources = valid_resources,
        clone_instruction = if project.source_type == "github" {
            format!(
                "Clones the repo from https://github.com/{}.git (branch: {})",
                project.github_repo.as_deref().unwrap_or(""),
                project.github_branch.as_deref().unwrap_or("main"),
            )
        } else {
            "Receives the application code (for local projects, the code will be pushed separately)".to_string()
        },
        domain_instructions = match project.domain.as_deref() {
            Some(d) if !d.is_empty() => format!(
                "Domain Setup:\n- Configure Caddy to serve on domain: {d}\n- Caddy will auto-provision HTTPS via Let's Encrypt\n- Output DNS instructions: the user needs to point {d} to the static IP via an A record"
            ),
            _ => "No custom domain -- configure Caddy to serve on the IP address with HTTP only.".to_string(),
        },
    );

    let adapter = ClaudeCliAdapter::default_path();
    let response = adapter
        .invoke_plan(&prompt)
        .await
        .map_err(|e| e.to_string())?;

    // Parse the response into IaC files
    let infra_dir = PathBuf::from(&project.repo_path).join("infrastructure");
    std::fs::create_dir_all(&infra_dir)
        .map_err(|e| format!("Failed to create infrastructure directory: {}", e))?;

    let mut files = parse_and_write_files(&response, &infra_dir)?;

    if files.is_empty() {
        return Err("AI did not generate any infrastructure files. Try regenerating.".to_string());
    }

    // Generate CI/CD pipeline for GitHub source projects
    if project.source_type == "github" {
        if let Ok(cicd_file) = generate_cicd_config(&adapter, &project).await {
            files.push(cicd_file);
        }
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

async fn generate_cicd_config(
    adapter: &ClaudeCliAdapter,
    project: &crate::models::project::Project,
) -> Result<String, String> {
    let repo = project.github_repo.as_deref().unwrap_or("");
    let branch = project.github_branch.as_deref().unwrap_or("main");
    let domain = project.domain.as_deref().unwrap_or("");

    let prompt = format!(
        r#"Generate a GitHub Actions deployment workflow for this project.

Project: {name}
GitHub Repo: {repo}
Branch: {branch}
Domain: {domain_info}

The workflow should:
1. Trigger on push to the {branch} branch
2. SSH into the production server
3. Pull the latest code from the repo
4. Install dependencies
5. Build the application
6. Restart the application (via PM2 or systemd)

The server's SSH private key and IP address should be stored as GitHub Secrets:
- SSH_PRIVATE_KEY: the server's SSH private key
- SERVER_IP: the server's static IP address
- SSH_USER: ubuntu (or the default user)

Output ONLY the raw YAML content. No markdown code fences, no explanations, no commentary.
Start with "name:" and end with the last line of the YAML.
"#,
        name = project.name,
        repo = repo,
        branch = branch,
        domain_info = if domain.is_empty() { "None".to_string() } else { domain.to_string() },
    );

    let response = adapter.invoke_plan(&prompt).await.map_err(|e| e.to_string())?;

    // Strip any markdown fences the AI might add despite instructions
    let yaml = response
        .trim()
        .trim_start_matches("```yaml")
        .trim_start_matches("```yml")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim()
        .to_string();

    // Write to .github/workflows/deploy.yml relative to repo root
    let repo_root = if project.source_type == "github" && !project.repo_path.is_empty() {
        PathBuf::from(&project.repo_path)
    } else {
        PathBuf::from(&project.repo_path)
    };

    let workflows_dir = repo_root.join(".github").join("workflows");
    std::fs::create_dir_all(&workflows_dir)
        .map_err(|e| format!("Failed to create .github/workflows: {}", e))?;

    let workflow_path = workflows_dir.join("deploy.yml");
    std::fs::write(&workflow_path, &yaml)
        .map_err(|e| format!("Failed to write deploy.yml: {}", e))?;

    Ok(".github/workflows/deploy.yml".to_string())
}

// ── Deployment Commands ──

const PLACEHOLDER_PATTERNS: &[&str] = &[
    "CHANGEME",
    "changeme",
    "CHANGE_ME",
    "change_me",
    "your-",
    "YOUR_",
    "REPLACE",
    "replace_me",
    "TODO",
    "xxx",
    "FIXME",
    "placeholder",
    "example.com",
];

fn check_for_placeholders(infra_dir: &Path) -> Result<(), String> {
    let tfvars_path = infra_dir.join("terraform.tfvars");
    if !tfvars_path.exists() {
        return Ok(());
    }

    let content = std::fs::read_to_string(&tfvars_path)
        .map_err(|e| format!("Failed to read terraform.tfvars: {}", e))?;

    let mut issues = Vec::new();
    for (line_num, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        for pattern in PLACEHOLDER_PATTERNS {
            if trimmed.contains(pattern) {
                issues.push(format!("  Line {}: {}", line_num + 1, trimmed));
                break;
            }
        }
    }

    if !issues.is_empty() {
        return Err(format!(
            "terraform.tfvars contains placeholder values that need to be replaced before deploying:\n\n{}\n\nEdit the file at:\n{}",
            issues.join("\n"),
            tfvars_path.display(),
        ));
    }

    Ok(())
}

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

    // Check for placeholder values before running anything
    check_for_placeholders(&infra_dir)?;

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

#[derive(serde::Serialize)]
pub struct DnsInstructions {
    pub domain: String,
    pub record_type: String,
    pub record_name: String,
    pub record_value: String,
    pub instructions: String,
}

#[tauri::command]
pub async fn get_dns_instructions(
    state: tauri::State<'_, AppDb>,
    project_id: String,
) -> Result<Option<DnsInstructions>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let project = crate::models::project::Project::get_by_id(&conn, &project_id)
        .map_err(|e| e.to_string())?
        .ok_or("Project not found")?;

    let domain = match project.domain.as_deref() {
        Some(d) if !d.is_empty() => d.to_string(),
        _ => return Ok(None),
    };

    // Try to get the static IP from the latest deployment output
    let deployments = Deployment::list_for_project(&conn, &project_id)
        .map_err(|e| e.to_string())?;
    let latest_completed = deployments.iter().find(|d| d.status == "completed");

    let ip = latest_completed
        .and_then(|d| d.apply_output.as_ref())
        .and_then(|output| {
            // Extract IP from "static_ip = X.X.X.X" in output
            output.lines()
                .find(|l| l.contains("static_ip") && l.contains("="))
                .and_then(|l| l.split('=').nth(1))
                .map(|ip| ip.trim().trim_matches('"').to_string())
        })
        .unwrap_or_else(|| "YOUR_SERVER_IP".to_string());

    Ok(Some(DnsInstructions {
        domain: domain.clone(),
        record_type: "A".to_string(),
        record_name: if domain.starts_with("www.") {
            domain.clone()
        } else {
            format!("@  (or {})", domain)
        },
        record_value: ip.clone(),
        instructions: format!(
            "To connect your domain, add this DNS record at your domain registrar:\n\n\
             Type: A\n\
             Name: {}\n\
             Value: {}\n\
             TTL: 300 (or Auto)\n\n\
             After the DNS propagates (usually 5-30 minutes), your app will be accessible at https://{}.\n\
             Caddy will automatically provision an SSL certificate via Let's Encrypt.",
            if domain.starts_with("www.") { "www" } else { "@" },
            ip,
            domain,
        ),
    }))
}

#[derive(serde::Serialize)]
pub struct CiCdSecrets {
    pub github_repo: String,
    pub secrets_url: String,
    pub server_ip: String,
    pub ssh_user: String,
    pub ssh_private_key: String,
    pub branch: String,
}

#[tauri::command]
pub async fn get_cicd_secrets(
    state: tauri::State<'_, AppDb>,
    project_id: String,
) -> Result<Option<CiCdSecrets>, String> {
    // Gather everything from DB first, then drop the lock before await
    let (project, server_ip, ssh_user, infra_dir) = {
        let conn = state.conn.lock().map_err(|e| e.to_string())?;
        let project = crate::models::project::Project::get_by_id(&conn, &project_id)
            .map_err(|e| e.to_string())?
            .ok_or("Project not found")?;

        let github_repo = match project.github_repo.as_deref() {
            Some(r) if !r.is_empty() => r.to_string(),
            _ => return Ok(None),
        };
        let _ = github_repo; // used later via project

        let deployments = Deployment::list_for_project(&conn, &project_id)
            .map_err(|e| e.to_string())?;
        let latest_completed = deployments.iter().find(|d| d.status == "completed");

        let apply_output = match latest_completed.and_then(|d| d.apply_output.as_ref()) {
            Some(o) => o.clone(),
            None => return Ok(None),
        };

        let server_ip = extract_output_value(&apply_output, "static_ip")
            .unwrap_or_else(|| "COULD_NOT_EXTRACT".to_string());

        let ssh_user = extract_output_value(&apply_output, "ssh_user")
            .or_else(|| extract_ssh_user_from_command(&apply_output))
            .unwrap_or_else(|| "ubuntu".to_string());

        let infra_dir = PathBuf::from(&project.repo_path).join("infrastructure");

        (project, server_ip, ssh_user, infra_dir)
    };
    // Lock is dropped here -- safe to await

    let github_repo = project.github_repo.as_deref().unwrap_or("").to_string();

    let ssh_key = get_tofu_output_sensitive(&infra_dir, "ssh_private_key").await
        .unwrap_or_else(|_| "Run: cd infrastructure && tofu output -raw ssh_private_key".to_string());

    Ok(Some(CiCdSecrets {
        secrets_url: format!("https://github.com/{}/settings/secrets/actions", github_repo),
        github_repo,
        server_ip,
        ssh_user,
        ssh_private_key: ssh_key,
        branch: project.github_branch.unwrap_or_else(|| "main".to_string()),
    }))
}

fn extract_output_value(output: &str, key: &str) -> Option<String> {
    output.lines()
        .find(|l| l.trim().starts_with(key) && l.contains('='))
        .and_then(|l| l.split('=').nth(1))
        .map(|v| v.trim().trim_matches('"').to_string())
}

fn extract_ssh_user_from_command(output: &str) -> Option<String> {
    // Extract from "ssh_command = ssh -i key.pem ubuntu@1.2.3.4"
    output.lines()
        .find(|l| l.contains("ssh_command"))
        .and_then(|l| l.split('@').next())
        .and_then(|l| l.split_whitespace().last())
        .map(|u| u.to_string())
}

async fn get_tofu_output_sensitive(infra_dir: &Path, key: &str) -> Result<String, String> {
    if !infra_dir.exists() {
        return Err("Infrastructure directory not found".to_string());
    }

    let output = if cfg!(windows) {
        tokio::process::Command::new("cmd")
            .args(["/C", "tofu", "output", "-raw", key])
            .current_dir(infra_dir)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .output()
            .await
    } else {
        tokio::process::Command::new("tofu")
            .args(["output", "-raw", key])
            .current_dir(infra_dir)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .output()
            .await
    };

    match output {
        Ok(o) if o.status.success() => {
            Ok(String::from_utf8_lossy(&o.stdout).trim().to_string())
        }
        Ok(o) => Err(String::from_utf8_lossy(&o.stderr).trim().to_string()),
        Err(e) => Err(format!("Failed to run tofu: {}", e)),
    }
}
