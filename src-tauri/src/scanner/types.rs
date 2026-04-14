use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FindingCategory {
    Language,
    Framework,
    Infrastructure,
    Config,
    CiCd,
    Database,
    Queue,
}

impl FindingCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            FindingCategory::Language => "language",
            FindingCategory::Framework => "framework",
            FindingCategory::Infrastructure => "infrastructure",
            FindingCategory::Config => "config",
            FindingCategory::CiCd => "ci_cd",
            FindingCategory::Database => "database",
            FindingCategory::Queue => "queue",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Detection {
    pub category: FindingCategory,
    pub name: String,
    pub confidence: f64,
    pub evidence_path: String,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScanReport {
    pub detections: Vec<Detection>,
    pub files_scanned: u64,
    pub duration_ms: u64,
    pub inferred_stack: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScanProgress {
    pub phase: String,
    pub files_checked: u64,
    pub detections_so_far: u64,
}
