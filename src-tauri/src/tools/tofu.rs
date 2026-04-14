use serde::Serialize;
use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;

#[derive(Debug, Clone, Serialize)]
pub struct TofuResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlanSummary {
    pub to_create: u32,
    pub to_update: u32,
    pub to_destroy: u32,
    pub risk_level: String,
    pub raw_output: String,
    pub summary_lines: Vec<String>,
}

fn build_tofu_command(args: &[&str], working_dir: &Path) -> Command {
    if cfg!(windows) {
        let mut cmd = Command::new("cmd");
        cmd.arg("/C").arg("tofu");
        for arg in args {
            cmd.arg(arg);
        }
        cmd.current_dir(working_dir);
        cmd
    } else {
        let mut cmd = Command::new("tofu");
        for arg in args {
            cmd.arg(arg);
        }
        cmd.current_dir(working_dir);
        cmd
    }
}

async fn run_tofu(args: &[&str], working_dir: &Path) -> Result<TofuResult, String> {
    if !working_dir.exists() {
        return Err(format!(
            "Infrastructure directory does not exist: {}",
            working_dir.display()
        ));
    }

    let mut cmd = build_tofu_command(args, working_dir);
    let output = cmd
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                "OpenTofu is not installed. Install it to manage infrastructure.\n\nWindows: winget install OpenTofu.OpenTofu\nmacOS: brew install opentofu\nLinux: https://opentofu.org/docs/intro/install/".to_string()
            } else {
                format!("Failed to run OpenTofu: {}", e)
            }
        })?;

    Ok(TofuResult {
        success: output.status.success(),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        exit_code: output.status.code(),
    })
}

pub async fn init(working_dir: &Path) -> Result<TofuResult, String> {
    run_tofu(&["init", "-no-color"], working_dir).await
}

pub async fn validate(working_dir: &Path) -> Result<TofuResult, String> {
    run_tofu(&["validate", "-no-color"], working_dir).await
}

pub async fn plan(working_dir: &Path) -> Result<TofuResult, String> {
    run_tofu(&["plan", "-no-color", "-input=false"], working_dir).await
}

pub async fn apply(working_dir: &Path) -> Result<TofuResult, String> {
    run_tofu(
        &["apply", "-no-color", "-input=false", "-auto-approve"],
        working_dir,
    )
    .await
}

pub async fn destroy(working_dir: &Path) -> Result<TofuResult, String> {
    run_tofu(
        &["destroy", "-no-color", "-input=false", "-auto-approve"],
        working_dir,
    )
    .await
}

pub fn parse_plan_output(output: &str) -> PlanSummary {
    let mut to_create: u32 = 0;
    let mut to_update: u32 = 0;
    let mut to_destroy: u32 = 0;
    let mut summary_lines = Vec::new();

    for line in output.lines() {
        let trimmed = line.trim();

        // Parse the summary line: "Plan: X to add, Y to change, Z to destroy."
        if trimmed.starts_with("Plan:") {
            if let Some(adds) = extract_count(trimmed, "to add") {
                to_create = adds;
            }
            if let Some(changes) = extract_count(trimmed, "to change") {
                to_update = changes;
            }
            if let Some(destroys) = extract_count(trimmed, "to destroy") {
                to_destroy = destroys;
            }
        }

        // Collect resource-level lines
        if trimmed.starts_with("# ") || trimmed.starts_with("+ ") || trimmed.starts_with("~ ") || trimmed.starts_with("- ") {
            summary_lines.push(trimmed.to_string());
        }
    }

    let risk_level = if to_destroy > 0 {
        "high".to_string()
    } else if to_update > 0 {
        "medium".to_string()
    } else if to_create > 0 {
        "low".to_string()
    } else {
        "none".to_string()
    };

    PlanSummary {
        to_create,
        to_update,
        to_destroy,
        risk_level,
        raw_output: output.to_string(),
        summary_lines,
    }
}

fn extract_count(text: &str, marker: &str) -> Option<u32> {
    let idx = text.find(marker)?;
    let before = &text[..idx];
    let num_str: String = before.chars().rev().take_while(|c| c.is_ascii_digit() || *c == ' ').collect::<String>().chars().rev().collect();
    num_str.trim().parse().ok()
}
