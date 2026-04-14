use super::{AdapterMessage, AdapterResponse, ReasoningAdapter};
use async_trait::async_trait;
use std::process::Stdio;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

pub struct ClaudeCliAdapter {
    pub cli_path: String,
}

impl ClaudeCliAdapter {
    pub fn new(cli_path: String) -> Self {
        Self { cli_path }
    }

    pub fn default_path() -> Self {
        if cfg!(windows) {
            if let Some(full_path) = Self::resolve_npm_global("claude.cmd") {
                return Self { cli_path: full_path };
            }
            Self { cli_path: "claude.cmd".to_string() }
        } else {
            Self { cli_path: "claude".to_string() }
        }
    }

    fn resolve_npm_global(cmd: &str) -> Option<String> {
        if let Ok(appdata) = std::env::var("APPDATA") {
            let path = std::path::PathBuf::from(&appdata).join("npm").join(cmd);
            if path.exists() {
                return Some(path.to_string_lossy().to_string());
            }
        }
        None
    }

    fn build_command(&self, args: &[&str]) -> Command {
        if cfg!(windows) {
            let mut cmd = Command::new("cmd");
            cmd.arg("/C").arg(&self.cli_path);
            for arg in args {
                cmd.arg(arg);
            }
            cmd
        } else {
            let mut cmd = Command::new(&self.cli_path);
            for arg in args {
                cmd.arg(arg);
            }
            cmd
        }
    }

    pub async fn invoke_plan(
        &self,
        prompt: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // Pipe prompt via stdin instead of CLI arg to avoid Windows
        // command-line length limits (8191 chars) truncating the context.
        let mut cmd = self.build_command(&["--print", "--output-format", "text", "-p"]);
        let mut child = cmd
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    format!(
                        "Claude Code CLI not found at '{}'. Make sure Claude Code is installed (npm install -g @anthropic-ai/claude-code) and the npm global bin directory is in your PATH.",
                        self.cli_path
                    )
                } else {
                    format!("Failed to run Claude Code CLI: {}", e)
                }
            })?;

        // Write the full prompt to stdin
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(prompt.as_bytes()).await
                .map_err(|e| format!("Failed to write prompt to Claude stdin: {}", e))?;
            // Drop stdin to close it and signal EOF
        }

        let output = child.wait_with_output().await
            .map_err(|e| format!("Failed to wait for Claude CLI: {}", e))?;

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
        self.build_command(&["--version"])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}
