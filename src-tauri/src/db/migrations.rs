use rusqlite::Connection;

struct Migration {
    version: i64,
    name: &'static str,
    sql: &'static str,
}

const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        name: "initial_schema",
        sql: "
            CREATE TABLE projects (
                id          TEXT PRIMARY KEY,
                name        TEXT NOT NULL,
                repo_path   TEXT NOT NULL,
                aws_profile TEXT,
                aws_region  TEXT NOT NULL DEFAULT 'us-east-1',
                description TEXT,
                created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
                updated_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
            );

            CREATE TABLE scans (
                id           TEXT PRIMARY KEY,
                project_id   TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
                status       TEXT NOT NULL DEFAULT 'pending',
                started_at   TEXT,
                completed_at TEXT,
                error_msg    TEXT,
                created_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
            );

            CREATE TABLE scan_findings (
                id            TEXT PRIMARY KEY,
                scan_id       TEXT NOT NULL REFERENCES scans(id) ON DELETE CASCADE,
                category      TEXT NOT NULL,
                name          TEXT NOT NULL,
                confidence    REAL NOT NULL DEFAULT 1.0,
                evidence_path TEXT,
                metadata_json TEXT,
                created_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
            );

            CREATE INDEX idx_scans_project_id ON scans(project_id);
            CREATE INDEX idx_scan_findings_scan_id ON scan_findings(scan_id);

            CREATE TABLE app_settings (
                key   TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
        ",
    },
    Migration {
        version: 2,
        name: "questionnaire_and_plans",
        sql: "
            CREATE TABLE questionnaire_responses (
                id              TEXT PRIMARY KEY,
                project_id      TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
                answers_json    TEXT NOT NULL DEFAULT '{}',
                completed       INTEGER NOT NULL DEFAULT 0,
                created_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
                updated_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
            );

            CREATE INDEX idx_questionnaire_project_id ON questionnaire_responses(project_id);

            CREATE TABLE plans (
                id              TEXT PRIMARY KEY,
                project_id      TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
                scan_id         TEXT REFERENCES scans(id),
                questionnaire_id TEXT REFERENCES questionnaire_responses(id),
                status          TEXT NOT NULL DEFAULT 'pending',
                plan_markdown   TEXT,
                plan_json       TEXT,
                alternatives    TEXT,
                cost_notes      TEXT,
                error_msg       TEXT,
                created_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
                updated_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
            );

            CREATE INDEX idx_plans_project_id ON plans(project_id);

            CREATE TABLE adapter_logs (
                id              TEXT PRIMARY KEY,
                project_id      TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
                adapter_name    TEXT NOT NULL,
                task_type       TEXT NOT NULL,
                prompt_text     TEXT NOT NULL,
                response_text   TEXT,
                response_json   TEXT,
                status          TEXT NOT NULL DEFAULT 'pending',
                duration_ms     INTEGER,
                error_msg       TEXT,
                created_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
            );

            CREATE INDEX idx_adapter_logs_project_id ON adapter_logs(project_id);
        ",
    },
];

pub fn run_all(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS _migrations (
            version     INTEGER PRIMARY KEY,
            name        TEXT NOT NULL,
            applied_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
        );",
    )?;

    let applied: Vec<i64> = {
        let mut stmt = conn.prepare("SELECT version FROM _migrations ORDER BY version")?;
        let rows = stmt.query_map([], |row| row.get(0))?;
        rows.collect::<Result<Vec<i64>, _>>()?
    };

    for migration in MIGRATIONS {
        if applied.contains(&migration.version) {
            continue;
        }
        log::info!(
            "Applying migration v{}: {}",
            migration.version,
            migration.name
        );
        conn.execute_batch(migration.sql)?;
        conn.execute(
            "INSERT INTO _migrations (version, name) VALUES (?1, ?2)",
            rusqlite::params![migration.version, migration.name],
        )?;
    }

    Ok(())
}
