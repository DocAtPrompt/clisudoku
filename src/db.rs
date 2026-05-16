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
    pub fn save_game(
        &self,
        puzzle: &str,
        puzzle_type: &str,
        variant_json: Option<&str>,
        difficulty: &str,
        state: &crate::puzzle::game_state::GameState,
        elapsed_ms: u64,
        started_at: &str,
    ) -> rusqlite::Result<i64> {
        let state_json = serde_json::to_string(state)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        self.conn.execute(
            "INSERT INTO saves (puzzle, puzzle_type, variant_json, difficulty, state_json, elapsed_ms, started_at, last_saved_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![puzzle, puzzle_type, variant_json, difficulty, state_json, elapsed_ms as i64, started_at, started_at],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn update_game(
        &self,
        id: i64,
        state: &crate::puzzle::game_state::GameState,
        elapsed_ms: u64,
        last_saved_at: &str,
    ) -> rusqlite::Result<()> {
        let state_json = serde_json::to_string(state)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        self.conn.execute(
            "UPDATE saves SET state_json=?1, elapsed_ms=?2, last_saved_at=?3 WHERE id=?4",
            rusqlite::params![state_json, elapsed_ms as i64, last_saved_at, id],
        )?;
        Ok(())
    }

    pub fn load_game(&self, id: i64) -> rusqlite::Result<SaveEntry> {
        self.conn.query_row(
            "SELECT id, puzzle, puzzle_type, variant_json, difficulty, state_json, elapsed_ms, started_at, last_saved_at FROM saves WHERE id=?1",
            rusqlite::params![id],
            |row| Ok(SaveEntry {
                id: row.get(0)?,
                puzzle: row.get(1)?,
                puzzle_type: row.get(2)?,
                variant_json: row.get(3)?,
                difficulty: row.get(4)?,
                state_json: row.get(5)?,
                elapsed_ms: row.get::<_, i64>(6)? as u64,
                started_at: row.get(7)?,
                last_saved_at: row.get(8)?,
            }),
        )
    }

    pub fn list_saves(&self) -> rusqlite::Result<Vec<SaveSummary>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, puzzle, puzzle_type, difficulty, elapsed_ms, started_at, last_saved_at FROM saves ORDER BY last_saved_at DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(SaveSummary {
                id: row.get(0)?,
                puzzle: row.get(1)?,
                puzzle_type: row.get(2)?,
                difficulty: row.get(3)?,
                elapsed_ms: row.get::<_, i64>(4)? as u64,
                started_at: row.get(5)?,
                last_saved_at: row.get(6)?,
            })
        })?;
        rows.collect()
    }

    pub fn delete_save(&self, id: i64) -> rusqlite::Result<()> {
        self.conn.execute("DELETE FROM saves WHERE id=?1", rusqlite::params![id])?;
        Ok(())
    }

    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|_e| rusqlite::Error::InvalidPath(parent.to_path_buf()))?;
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
    use crate::puzzle::{grid::Grid, game_state::GameState};

    const EASY: &str =
        "530070000600195000098000060800060003400803001700020006060000280000419005000080079";

    fn easy_state() -> GameState {
        GameState::new(Grid::from_str(EASY).unwrap())
    }

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

    #[test]
    fn save_and_list() {
        let (db, _dir) = temp_db();
        let state = easy_state();
        let id = db.save_game(EASY, "Classic", None, "Hard", &state, 0, "2026-01-01T00:00:00Z").unwrap();
        assert!(id > 0);
        let list = db.list_saves().unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, id);
        assert_eq!(list[0].difficulty, "Hard");
        assert_eq!(list[0].puzzle_type, "Classic");
    }

    #[test]
    fn update_game_changes_elapsed() {
        let (db, _dir) = temp_db();
        let state = easy_state();
        let id = db.save_game(EASY, "Classic", None, "Hard", &state, 0, "2026-01-01T00:00:00Z").unwrap();
        db.update_game(id, &state, 12345, "2026-01-02T00:00:00Z").unwrap();
        let list = db.list_saves().unwrap();
        assert_eq!(list[0].elapsed_ms, 12345);
        assert_eq!(list[0].last_saved_at, "2026-01-02T00:00:00Z");
    }

    #[test]
    fn load_game_round_trip() {
        let (db, _dir) = temp_db();
        let state = easy_state();
        let id = db.save_game(EASY, "Classic", None, "Easy", &state, 500, "2026-01-01T00:00:00Z").unwrap();
        let entry = db.load_game(id).unwrap();
        assert_eq!(entry.puzzle, EASY);
        assert_eq!(entry.elapsed_ms, 500);
        // Verify state_json round-trips back to a valid GameState
        let restored: crate::puzzle::game_state::GameState =
            serde_json::from_str(&entry.state_json).expect("state_json must deserialise");
        assert_eq!(restored.grid().to_str(), EASY);
    }

    #[test]
    fn delete_save_removes_row() {
        let (db, _dir) = temp_db();
        let state = easy_state();
        let id = db.save_game(EASY, "Classic", None, "Easy", &state, 0, "2026-01-01T00:00:00Z").unwrap();
        db.delete_save(id).unwrap();
        assert_eq!(db.list_saves().unwrap().len(), 0);
    }

    #[test]
    fn list_saves_ordered_newest_first() {
        let (db, _dir) = temp_db();
        let state = easy_state();
        db.save_game(EASY, "Classic", None, "Easy", &state, 0, "2026-01-01T00:00:00Z").unwrap();
        db.save_game(EASY, "Classic", None, "Hard", &state, 0, "2026-01-03T00:00:00Z").unwrap();
        let list = db.list_saves().unwrap();
        assert_eq!(list[0].difficulty, "Hard"); // newest first
    }
}
