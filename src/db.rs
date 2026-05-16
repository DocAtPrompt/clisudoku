use rusqlite::{Connection, Result};
use std::path::Path;

pub struct Database {
    pub(crate) conn: Connection,
}

#[derive(Debug, Clone)]
pub struct SaveSummary {
    pub id: i64,
    pub puzzle: String,
    pub puzzle_type: String,
    pub difficulty: String,
    pub elapsed_ms: u64,
    pub started_at: String,
    pub last_saved_at: String,
}

#[derive(Debug, Clone)]
pub struct SaveEntry {
    pub id: i64,
    pub puzzle: String,
    pub puzzle_type: String,
    pub variant_json: Option<String>,
    pub difficulty: String,
    pub state_json: String,
    pub elapsed_ms: u64,
    pub started_at: String,
    pub last_saved_at: String,
}

#[derive(Debug, Clone)]
pub struct ScoreEntry {
    pub id: Option<i64>,
    pub puzzle: String,
    pub puzzle_type: String,
    pub difficulty: String,
    pub time_ms: u64,
    pub hint_count: u32,
    pub error_count: u32,
    pub scan_used: bool,
    pub rating: Option<u8>,
    pub started_at: String,
    pub finished_at: String,
}

impl Database {
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        let conn = Connection::open(path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS saves (
                id            INTEGER PRIMARY KEY AUTOINCREMENT,
                puzzle        TEXT    NOT NULL,
                puzzle_type   TEXT    NOT NULL,
                variant_json  TEXT,
                difficulty    TEXT    NOT NULL,
                state_json    TEXT    NOT NULL,
                elapsed_ms    INTEGER NOT NULL,
                started_at    TEXT    NOT NULL,
                last_saved_at TEXT    NOT NULL
            );
            CREATE TABLE IF NOT EXISTS scores (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                puzzle      TEXT    NOT NULL,
                puzzle_type TEXT    NOT NULL,
                difficulty  TEXT    NOT NULL,
                time_ms     INTEGER NOT NULL,
                hint_count  INTEGER NOT NULL,
                error_count INTEGER NOT NULL,
                scan_used   INTEGER NOT NULL,
                rating      INTEGER,
                started_at  TEXT    NOT NULL,
                finished_at TEXT    NOT NULL
            );",
        )?;
        Ok(Self { conn })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn temp_db() -> (Database, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.db");
        let db = Database::open(&path).unwrap();
        (db, dir)
    }

    #[test]
    fn open_creates_tables() {
        let (db, _dir) = temp_db();
        let count: i64 = db.conn
            .query_row("SELECT COUNT(*) FROM saves", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0);
        let count: i64 = db.conn
            .query_row("SELECT COUNT(*) FROM scores", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn open_is_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.db");
        Database::open(&path).unwrap();
        Database::open(&path).unwrap();
    }
}
