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
    Migration {
        version: 3,
        name: "plan_messages",
        sql: "
            CREATE TABLE plan_messages (
                id          TEXT PRIMARY KEY,
                plan_id     TEXT NOT NULL REFERENCES plans(id) ON DELETE CASCADE,
                role        TEXT NOT NULL,
                content     TEXT NOT NULL,
                created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
            );

            CREATE INDEX idx_plan_messages_plan_id ON plan_messages(plan_id);
        ",
    },
    Migration {
        version: 4,
        name: "deployments_and_iac",
        sql: "
            CREATE TABLE iac_generations (
                id              TEXT PRIMARY KEY,
                project_id      TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
                plan_id         TEXT REFERENCES plans(id),
                output_dir      TEXT NOT NULL,
                files_json      TEXT,
                status          TEXT NOT NULL DEFAULT 'pending',
                error_msg       TEXT,
                created_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
            );

            CREATE INDEX idx_iac_project_id ON iac_generations(project_id);

            CREATE TABLE deployments (
                id              TEXT PRIMARY KEY,
                project_id      TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
                iac_id          TEXT REFERENCES iac_generations(id),
                action          TEXT NOT NULL DEFAULT 'apply',
                status          TEXT NOT NULL DEFAULT 'pending',
                plan_output     TEXT,
                plan_summary    TEXT,
                apply_output    TEXT,
                resources_json  TEXT,
                risk_level      TEXT,
                approved        INTEGER NOT NULL DEFAULT 0,
                approved_at     TEXT,
                error_msg       TEXT,
                started_at      TEXT,
                completed_at    TEXT,
                created_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
            );

            CREATE INDEX idx_deployments_project_id ON deployments(project_id);

            CREATE TABLE aws_connections (
                id              TEXT PRIMARY KEY,
                project_id      TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
                account_id      TEXT,
                arn             TEXT,
                user_id         TEXT,
                status          TEXT NOT NULL DEFAULT 'unchecked',
                error_msg       TEXT,
                checked_at      TEXT,
                created_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
            );

            CREATE UNIQUE INDEX idx_aws_conn_project_id ON aws_connections(project_id);
        ",
    },
    Migration {
        version: 5,
        name: "plan_approval",
        sql: "
            ALTER TABLE plans ADD COLUMN approved INTEGER NOT NULL DEFAULT 0;
            ALTER TABLE plans ADD COLUMN approved_at TEXT;
        ",
    },
    Migration {
        version: 6,
        name: "plan_options",
        sql: "
            CREATE TABLE plan_options (
                id              TEXT PRIMARY KEY,
                plan_id         TEXT NOT NULL REFERENCES plans(id) ON DELETE CASCADE,
                label           TEXT NOT NULL,
                title           TEXT NOT NULL,
                content         TEXT NOT NULL,
                source          TEXT NOT NULL DEFAULT 'generation',
                source_message_id TEXT,
                approved        INTEGER NOT NULL DEFAULT 0,
                approved_at     TEXT,
                created_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
            );

            CREATE INDEX idx_plan_options_plan_id ON plan_options(plan_id);
        ",
    },
    Migration {
        version: 7,
        name: "github_source_and_domain",
        sql: "
            ALTER TABLE projects ADD COLUMN source_type TEXT NOT NULL DEFAULT 'local';
        ",
    },
    Migration {
        version: 8,
        name: "github_source_fields",
        sql: "
            ALTER TABLE projects ADD COLUMN github_repo TEXT;
        ",
    },
    Migration {
        version: 9,
        name: "github_branch",
        sql: "
            ALTER TABLE projects ADD COLUMN github_branch TEXT DEFAULT 'main';
        ",
    },
    Migration {
        version: 10,
        name: "project_domain",
        sql: "
            ALTER TABLE projects ADD COLUMN domain TEXT;
        ",
    },
    Migration {
        version: 11,
        name: "project_aws_keys",
        sql: "
            ALTER TABLE projects ADD COLUMN aws_access_key_id TEXT;
        ",
    },
    Migration {
        version: 12,
        name: "project_aws_secret",
        sql: "
            ALTER TABLE projects ADD COLUMN aws_secret_access_key TEXT;
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
