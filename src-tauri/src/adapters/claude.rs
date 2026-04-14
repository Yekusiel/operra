use super::{AdapterMessage, AdapterResponse, ReasoningAdapter};
use async_trait::async_trait;
use std::process::Stdio;
use tokio::process::Command;

pub struct ClaudeCliAdapter {
    pub cli_path: String,
}

impl ClaudeCliAdapter {
    pub fn new(cli_path: String) -> Self {
        Self { cli_path }
    }

    pub fn default_path() -> Self {
        Self {
            cli_path: "claude".to_string(),
        }
    }

    /// Invoke Claude Code for plan generation specifically.
    /// Uses --print for non-interactive output.
    pub async fn invoke_plan(
        &self,
        prompt: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let output = Command::new(&self.cli_path)
            .args(["--print", "--output-format", "text", "--verbose"])
            .arg(prompt)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    format!(
                        "Claude Code CLI not found at '{}'. Make sure Claude Code is installed and in your PATH.",
                        self.cli_path
                    )
                } else {
                    format!("Failed to run Claude Code CLI: {}", e)
                }
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!(
                "Claude Code CLI exited with status {}: {}",
                output.status.code().unwrap_or(-1),
                stderr.trim()
            )
            .into());
        }

        let stdout = String::from_utf8(output.stdout)
            .map_err(|e| format!("Invalid UTF-8 in Claude output: {}", e))?;

        if stdout.trim().is_empty() {
            return Err("Claude Code returned empty response".into());
        }

        Ok(stdout)
    }
}

#[async_trait]
impl ReasoningAdapter for ClaudeCliAdapter {
    fn name(&self) -> &str {
        "claude-code"
    }

    async fn invoke(
        &self,
        prompt: &str,
        _context: &[AdapterMessage],
    ) -> Result<AdapterResponse, Box<dyn std::error::Error + Send + Sync>> {
        let content = self.invoke_plan(prompt).await?;
        Ok(AdapterResponse {
            content,
            structured: None,
        })
    }

    async fn is_available(&self) -> bool {
        Command::new(&self.cli_path)
            .arg("--version")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}
