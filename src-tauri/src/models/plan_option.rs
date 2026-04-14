use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanOption {
    pub id: String,
    pub plan_id: String,
    pub label: String,        // "A", "B", "C", ...
    pub title: String,        // "ECS Fargate with RDS", etc.
    pub content: String,      // Full markdown description
    pub source: String,       // "generation" or "chat"
    pub source_message_id: Option<String>,
    pub approved: bool,
    pub approved_at: Option<String>,
    pub created_at: String,
}

impl PlanOption {
    pub fn create(
        conn: &Connection,
        plan_id: &str,
        label: &str,
        title: &str,
        content: &str,
        source: &str,
        source_message_id: Option<&str>,
    ) -> Result<PlanOption, rusqlite::Error> {
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO plan_options (id, plan_id, label, title, content, source, source_message_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![id, plan_id, label, title, content, source, source_message_id],
        )?;
        Self::get_by_id(conn, &id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)
    }

    pub fn approve(conn: &Connection, id: &str) -> Result<(), rusqlite::Error> {
        let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        // First unapprove all options for this plan
        let plan_id: String = conn.query_row(
            "SELECT plan_id FROM plan_options WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )?;
        conn.execute(
            "UPDATE plan_options SET approved = 0, approved_at = NULL WHERE plan_id = ?1",
            params![plan_id],
        )?;
        // Then approve the selected one
        conn.execute(
            "UPDATE plan_options SET approved = 1, approved_at = ?1 WHERE id = ?2",
            params![now, id],
        )?;
        // Also mark the parent plan as approved
        conn.execute(
            "UPDATE plans SET approved = 1, approved_at = ?1 WHERE id = ?2",
            params![now, plan_id],
        )?;
        Ok(())
    }

    pub fn list_for_plan(conn: &Connection, plan_id: &str) -> Result<Vec<PlanOption>, rusqlite::Error> {
        let mut stmt = conn.prepare(
            "SELECT id, plan_id, label, title, content, source, source_message_id,
                    approved, approved_at, created_at
             FROM plan_options WHERE plan_id = ?1 ORDER BY created_at ASC",
        )?;
        let rows = stmt.query_map(params![plan_id], Self::from_row)?;
        rows.collect()
    }

    pub fn get_approved(conn: &Connection, plan_id: &str) -> Result<Option<PlanOption>, rusqlite::Error> {
        let mut stmt = conn.prepare(
            "SELECT id, plan_id, label, title, content, source, source_message_id,
                    approved, approved_at, created_at
             FROM plan_options WHERE plan_id = ?1 AND approved = 1 LIMIT 1",
        )?;
        let mut rows = stmt.query_map(params![plan_id], Self::from_row)?;
        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }

    pub fn next_label(conn: &Connection, plan_id: &str) -> Result<String, rusqlite::Error> {
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM plan_options WHERE plan_id = ?1",
            params![plan_id],
            |row| row.get(0),
        )?;
        let label = (b'A' + count as u8) as char;
        Ok(label.to_string())
    }

    fn get_by_id(conn: &Connection, id: &str) -> Result<Option<PlanOption>, rusqlite::Error> {
        let mut stmt = conn.prepare(
            "SELECT id, plan_id, label, title, content, source, source_message_id,
                    approved, approved_at, created_at
             FROM plan_options WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(params![id], Self::from_row)?;
        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }

    fn from_row(row: &rusqlite::Row) -> Result<PlanOption, rusqlite::Error> {
        Ok(PlanOption {
            id: row.get(0)?,
            plan_id: row.get(1)?,
            label: row.get(2)?,
            title: row.get(3)?,
            content: row.get(4)?,
            source: row.get(5)?,
            source_message_id: row.get(6)?,
            approved: row.get::<_, i32>(7)? != 0,
            approved_at: row.get(8)?,
            created_at: row.get(9)?,
        })
    }
}

/// Headings that signal the end of plan options (non-plan sections).
const STOP_HEADINGS: &[&str] = &[
    "recommendation",
    "summary",
    "conclusion",
    "comparison",
    "next steps",
    "which plan",
    "cost comparison",
    "final thoughts",
];

/// Parse plan options from AI response text (strict mode for initial generation).
/// Only matches lines that are markdown headings starting with "## Plan [A-Z]:".
pub fn parse_plan_options(text: &str) -> Vec<(String, String, String)> {
    parse_plan_options_inner(text, true)
}

/// Parse plan options with relaxed matching (for chat responses).
/// Also matches "**Plan D: Title**" and "Plan D: Title" without # prefix.
pub fn parse_plan_options_relaxed(text: &str) -> Vec<(String, String, String)> {
    parse_plan_options_inner(text, false)
}

fn parse_plan_options_inner(text: &str, strict: bool) -> Vec<(String, String, String)> {
    let mut options = Vec::new();
    let mut current_label: Option<String> = None;
    let mut current_title: Option<String> = None;
    let mut current_content = String::new();

    for line in text.lines() {
        // Check if this is a plan header
        if let Some(plan_info) = extract_plan_header(line, strict) {
            // Save previous option
            if let (Some(label), Some(title)) = (current_label.take(), current_title.take()) {
                let content = current_content.trim().to_string();
                if !content.is_empty() {
                    options.push((label, title, content));
                }
            }
            current_label = Some(plan_info.0);
            current_title = Some(plan_info.1);
            current_content = String::new();
        } else if is_non_plan_heading(line) {
            // Hit a section like "Recommendation" -- stop collecting for current plan
            if let (Some(label), Some(title)) = (current_label.take(), current_title.take()) {
                let content = current_content.trim().to_string();
                if !content.is_empty() {
                    options.push((label, title, content));
                }
            }
            current_content = String::new();
            // Don't collect any more content
        } else if current_label.is_some() {
            current_content.push_str(line);
            current_content.push('\n');
        }
    }

    // Save last option
    if let (Some(label), Some(title)) = (current_label, current_title) {
        let content = current_content.trim().to_string();
        if !content.is_empty() {
            options.push((label, title, content));
        }
    }

    options
}

/// Check if a line is a markdown heading that's NOT a plan header.
/// This detects sections like "## Recommendation", "## Summary", etc.
fn is_non_plan_heading(line: &str) -> bool {
    let trimmed = line.trim();
    if !trimmed.starts_with('#') {
        return false;
    }
    let heading_text = trimmed.trim_start_matches('#').trim().to_lowercase();
    // If it's a "Plan X" heading, it's not a stop heading
    if heading_text.starts_with("plan ") {
        if let Some(c) = heading_text.chars().nth(5) {
            if c.is_ascii_uppercase() || c.is_ascii_lowercase() {
                return false;
            }
        }
    }
    STOP_HEADINGS.iter().any(|s| heading_text.starts_with(s))
}

fn extract_plan_header(line: &str, strict: bool) -> Option<(String, String)> {
    let trimmed = line.trim();

    if strict {
        // Strict mode: must be a heading line (starts with #)
        if !trimmed.starts_with('#') {
            return None;
        }
    } else {
        // Relaxed mode: must start with #, **, or "Plan " directly at line start
        if !trimmed.starts_with('#') && !trimmed.starts_with("**Plan ") && !trimmed.starts_with("Plan ") {
            return None;
        }
    }

    let stripped = trimmed
        .trim_start_matches('#')
        .trim()
        .trim_start_matches("**")
        .trim_end_matches("**")
        .trim();

    // Match "Plan X: Title" or "Plan X - Title" where X is A-Z
    if !stripped.starts_with("Plan ") {
        return None;
    }

    let after_plan = &stripped[5..]; // skip "Plan "
    let label_char = after_plan.chars().next()?;
    if !label_char.is_ascii_uppercase() {
        return None;
    }

    // The character after the letter must be : or - or whitespace (not more word chars)
    // This prevents matching "Plan Details" or "Plan Overview"
    let after_letter = &after_plan[1..];
    let next_char = after_letter.trim_start().chars().next().unwrap_or(':');
    if next_char != ':' && next_char != '-' && next_char != '\u{2013}' {
        return None;
    }

    let title = after_letter
        .trim()
        .trim_start_matches(':')
        .trim_start_matches('-')
        .trim_start_matches('\u{2013}')
        .trim()
        .to_string();

    let title = if title.is_empty() {
        format!("Plan {}", label_char)
    } else {
        title
    };

    Some((label_char.to_string(), title))
}
