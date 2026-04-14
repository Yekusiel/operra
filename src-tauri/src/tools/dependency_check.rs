use serde::Serialize;
use std::process::Stdio;
use tokio::process::Command;

#[derive(Debug, Clone, Serialize)]
pub struct ToolStatus {
    pub name: String,
    pub installed: bool,
    pub version: Option<String>,
    pub path: Option<String>,
    pub install_instructions: String,
    pub required_for: String,
}

async fn check_tool(
    name: &str,
    commands_to_try: &[&str],
    version_args: &[&str],
    install_instructions: &str,
    required_for: &str,
) -> ToolStatus {
    for cmd_name in commands_to_try {
        let result = if cfg!(windows) {
            Command::new("cmd")
                .args(["/C", cmd_name])
                .args(version_args)
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
                .await
        } else {
            Command::new(cmd_name)
                .args(version_args)
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
                .await
        };

        if let Ok(output) = result {
            if output.status.success() {
                let version_str = String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .next()
                    .unwrap_or("")
                    .trim()
                    .to_string();

                // Try to find the path
                let path = find_tool_path(cmd_name).await;

                return ToolStatus {
                    name: name.to_string(),
                    installed: true,
                    version: Some(version_str),
                    path,
                    install_instructions: install_instructions.to_string(),
                    required_for: required_for.to_string(),
                };
            }
        }
    }

    ToolStatus {
        name: name.to_string(),
        installed: false,
        version: None,
        path: None,
        install_instructions: install_instructions.to_string(),
        required_for: required_for.to_string(),
    }
}

async fn find_tool_path(cmd: &str) -> Option<String> {
    let which_cmd = if cfg!(windows) { "where" } else { "which" };
    let output = Command::new(which_cmd)
        .arg(cmd)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .ok()?;

    if output.status.success() {
        let path = String::from_utf8_lossy(&output.stdout)
            .lines()
            .next()?
            .trim()
            .to_string();
        Some(path)
    } else {
        None
    }
}

pub async fn check_aws_cli() -> ToolStatus {
    check_tool(
        "AWS CLI",
        &[
            "aws",
            "C:\\Program Files\\Amazon\\AWSCLIV2\\aws.exe",
        ],
        &["--version"],
        if cfg!(windows) {
            "Install with: winget install Amazon.AWSCLI\nThen restart your terminal and run: aws configure"
        } else {
            "Install from https://docs.aws.amazon.com/cli/latest/userguide/getting-started-install.html\nThen run: aws configure"
        },
        "AWS account connection, resource inspection, deployment",
    )
    .await
}

pub async fn check_tofu() -> ToolStatus {
    check_tool(
        "OpenTofu",
        &["tofu"],
        &["--version"],
        if cfg!(windows) {
            "Install with: winget install --exact --id=OpenTofu.Tofu\nThen restart your terminal."
        } else {
            "Install from https://opentofu.org/docs/intro/install/\nOr via brew: brew install opentofu"
        },
        "Infrastructure code generation, deployment planning, resource provisioning",
    )
    .await
}

pub async fn check_claude_cli() -> ToolStatus {
    let commands = if cfg!(windows) {
        vec!["claude.cmd"]
    } else {
        vec!["claude"]
    };
    let cmds: Vec<&str> = commands.iter().map(|s| s.as_ref()).collect();

    check_tool(
        "Claude Code",
        &cmds,
        &["--version"],
        "Install with: npm install -g @anthropic-ai/claude-code\nThen run: claude (to authenticate)",
        "AI-powered infrastructure planning, code generation, optimization",
    )
    .await
}

#[derive(Debug, Clone, Serialize)]
pub struct DependencyReport {
    pub tools: Vec<ToolStatus>,
    pub all_installed: bool,
    pub missing_count: usize,
}

pub async fn check_all() -> DependencyReport {
    let tools = vec![
        check_aws_cli().await,
        check_tofu().await,
        check_claude_cli().await,
    ];

    let missing_count = tools.iter().filter(|t| !t.installed).count();

    DependencyReport {
        all_installed: missing_count == 0,
        missing_count,
        tools,
    }
}
