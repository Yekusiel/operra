pub mod migrations;

use rusqlite::Connection;
use std::path::Path;
use std::sync::Mutex;

pub struct AppDb {
    pub conn: Mutex<Connection>,
}

impl AppDb {
    pub fn init(app_data_dir: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let db_path = app_data_dir.join("operra.db");
        let conn = Connection::open(&db_path)?;

        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA foreign_keys=ON;
             PRAGMA busy_timeout=5000;",
        )?;

        migrations::run_all(&conn)?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }
}
