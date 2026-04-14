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

/// Parse plan options from AI response text.
/// Looks for "## Plan A:", "## Plan B:", etc. patterns.
pub fn parse_plan_options(text: &str) -> Vec<(String, String, String)> {
    let mut options = Vec::new();
    let mut current_label: Option<String> = None;
    let mut current_title: Option<String> = None;
    let mut current_content = String::new();

    for line in text.lines() {
        // Match patterns like "## Plan A: Title" or "### Plan A: Title" or "## Plan A - Title"
        if let Some(plan_info) = extract_plan_header(line) {
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

fn extract_plan_header(line: &str) -> Option<(String, String)> {
    let trimmed = line.trim();

    // Strip markdown heading markers and bold markers
    let stripped = trimmed
        .trim_start_matches('#')
        .trim()
        .trim_start_matches("**")
        .trim_end_matches("**")
        .trim();

    // Match "Plan X: Title" or "Plan X - Title" where X is A-Z
    if !stripped.to_lowercase().starts_with("plan ") {
        return None;
    }

    let after_plan = &stripped[5..]; // skip "Plan "
    let label_char = after_plan.chars().next()?;
    if !label_char.is_ascii_uppercase() {
        return None;
    }

    let rest = after_plan[1..].trim();
    let title = rest
        .trim_start_matches(':')
        .trim_start_matches('-')
        .trim_start_matches('\u{2013}') // en-dash
        .trim()
        .trim_end_matches("**")
        .trim()
        .to_string();

    let title = if title.is_empty() {
        format!("Plan {}", label_char)
    } else {
        title
    };

    Some((label_char.to_string(), title))
}
