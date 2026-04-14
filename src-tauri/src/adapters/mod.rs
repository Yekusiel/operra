pub mod claude;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterResponse {
    pub content: String,
    pub structured: Option<serde_json::Value>,
}

#[async_trait]
pub trait ReasoningAdapter: Send + Sync {
    fn name(&self) -> &str;

    async fn invoke(
        &self,
        prompt: &str,
        context: &[AdapterMessage],
    ) -> Result<AdapterResponse, Box<dyn std::error::Error + Send + Sync>>;

    async fn is_available(&self) -> bool;
}
