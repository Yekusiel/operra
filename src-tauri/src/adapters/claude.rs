use super::{AdapterMessage, AdapterResponse, ReasoningAdapter};
use async_trait::async_trait;

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
}

#[async_trait]
impl ReasoningAdapter for ClaudeCliAdapter {
    fn name(&self) -> &str {
        "claude-code"
    }

    async fn invoke(
        &self,
        _prompt: &str,
        _context: &[AdapterMessage],
    ) -> Result<AdapterResponse, Box<dyn std::error::Error + Send + Sync>> {
        // Phase 2: Implement via tokio::process::Command
        // claude --print --output-format json "prompt"
        Err("Claude adapter not yet implemented. Coming in Phase 2.".into())
    }

    async fn is_available(&self) -> bool {
        // Phase 2: Check if claude CLI is in PATH
        // tokio::process::Command::new(&self.cli_path).arg("--version").output()
        false
    }
}
