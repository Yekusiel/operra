use crate::models::scan::ScanFinding;
use serde::{Deserialize, Serialize};

/// Auto-filled questionnaire answers derived from scan findings.
/// Each field includes the inferred value and the evidence that led to it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoFilledAnswers {
    pub database_needs: Option<AutoFillEntry>,
    pub background_jobs: Option<AutoFillEntry>,
    pub networking: Option<AutoFillEntry>,
    pub storage_needs: Option<AutoFillEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoFillEntry {
    pub value: String,
    pub reason: String,
    pub evidence: Vec<String>,
}

pub fn autofill_from_findings(findings: &[ScanFinding]) -> AutoFilledAnswers {
    AutoFilledAnswers {
        database_needs: infer_database(findings),
        background_jobs: infer_background_jobs(findings),
        networking: infer_networking(findings),
        storage_needs: infer_storage(findings),
    }
}

fn infer_database(findings: &[ScanFinding]) -> Option<AutoFillEntry> {
    let db_findings: Vec<&ScanFinding> = findings
        .iter()
        .filter(|f| f.category == "database")
        .collect();

    if db_findings.is_empty() {
        return None;
    }

    let has_relational = db_findings.iter().any(|f| {
        f.metadata_json
            .as_deref()
            .unwrap_or("")
            .contains("\"relational\"")
    });
    let has_nosql = db_findings.iter().any(|f| {
        f.metadata_json
            .as_deref()
            .unwrap_or("")
            .contains("\"nosql\"")
    });

    let value = match (has_relational, has_nosql) {
        (true, true) => "both",
        (true, false) => "relational",
        (false, true) => "nosql",
        (false, false) => "existing",
    };

    let db_names: Vec<String> = db_findings.iter().map(|f| f.name.clone()).collect();
    let evidence: Vec<String> = db_findings
        .iter()
        .filter_map(|f| f.evidence_path.clone())
        .collect();

    Some(AutoFillEntry {
        value: value.to_string(),
        reason: format!("Detected: {}", db_names.join(", ")),
        evidence,
    })
}

fn infer_background_jobs(findings: &[ScanFinding]) -> Option<AutoFillEntry> {
    let queue_findings: Vec<&ScanFinding> = findings
        .iter()
        .filter(|f| f.category == "queue")
        .collect();

    if queue_findings.is_empty() {
        return None;
    }

    let has_event_streaming = queue_findings.iter().any(|f| {
        f.metadata_json
            .as_deref()
            .unwrap_or("")
            .contains("\"event_streaming\"")
    });
    let has_task_queue = queue_findings.iter().any(|f| {
        let meta = f.metadata_json.as_deref().unwrap_or("");
        meta.contains("\"task_queue\"") || meta.contains("\"redis_backed\"") || meta.contains("\"amqp\"") || meta.contains("\"aws_native\"")
    });
    let has_scheduler = queue_findings.iter().any(|f| {
        f.metadata_json
            .as_deref()
            .unwrap_or("")
            .contains("\"scheduler\"")
    });

    let value = if has_event_streaming {
        "complex"
    } else if has_task_queue {
        "queues"
    } else if has_scheduler {
        "simple"
    } else {
        "queues"
    };

    let names: Vec<String> = queue_findings.iter().map(|f| f.name.clone()).collect();
    let evidence: Vec<String> = queue_findings
        .iter()
        .filter_map(|f| f.evidence_path.clone())
        .collect();

    Some(AutoFillEntry {
        value: value.to_string(),
        reason: format!("Detected: {}", names.join(", ")),
        evidence,
    })
}

fn infer_networking(findings: &[ScanFinding]) -> Option<AutoFillEntry> {
    let names: Vec<&str> = findings.iter().map(|f| f.name.as_str()).collect();

    // If there's a frontend framework + backend framework, it's mixed networking
    let has_frontend = names.iter().any(|n| matches!(*n, "React" | "Next.js"));
    let has_backend = names.iter().any(|n| matches!(*n, "Express" | "Fastify" | "Django" | "Flask"));

    if has_frontend && has_backend {
        return Some(AutoFillEntry {
            value: "mixed".to_string(),
            reason: "Frontend + backend detected, suggesting public frontend with private backend".to_string(),
            evidence: vec!["package.json".to_string()],
        });
    }

    // WordPress is public
    if names.contains(&"WordPress") {
        return Some(AutoFillEntry {
            value: "public".to_string(),
            reason: "WordPress detected, typically public-facing".to_string(),
            evidence: vec!["wp-config.php".to_string()],
        });
    }

    None
}

fn infer_storage(findings: &[ScanFinding]) -> Option<AutoFillEntry> {
    let names: Vec<&str> = findings.iter().map(|f| f.name.as_str()).collect();

    // Static sites need static asset storage
    if names.contains(&"React") && !names.contains(&"Express") && !names.contains(&"Fastify") && !names.contains(&"Next.js") {
        return Some(AutoFillEntry {
            value: "static".to_string(),
            reason: "Static React app detected, needs static asset hosting".to_string(),
            evidence: vec!["package.json".to_string()],
        });
    }

    None
}
