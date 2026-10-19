use std::path::{Path, PathBuf};

use rusqlite::{Connection, params};

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error(transparent)]
    Database(#[from] rusqlite::Error),
}

pub struct LibraryStore {
    connection: Connection,
}

impl LibraryStore {
    pub fn open(data_dir: &Path) -> Result<Self, StoreError> {
        let connection = Connection::open(data_dir.join("tournepage.sqlite3"))?;
        connection.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA foreign_keys=ON;
             CREATE TABLE IF NOT EXISTS library_folders (
                 path TEXT PRIMARY KEY NOT NULL
             );
             CREATE TABLE IF NOT EXISTS preferences (
                 key TEXT PRIMARY KEY NOT NULL,
                 value TEXT NOT NULL
             );
             PRAGMA user_version=1;",
        )?;
        Ok(Self { connection })
    }

    pub fn folders(&self) -> Result<Vec<PathBuf>, StoreError> {
        let mut query = self
            .connection
            .prepare("SELECT path FROM library_folders ORDER BY path")?;
        let paths = query.query_map([], |row| row.get::<_, String>(0))?;
        paths
            .map(|path| path.map(PathBuf::from).map_err(StoreError::from))
            .collect()
    }

    pub fn add_folder(&self, path: &Path) -> Result<(), StoreError> {
        self.connection.execute(
            "INSERT OR IGNORE INTO library_folders(path) VALUES (?1)",
            params![path.to_string_lossy()],
        )?;
        Ok(())
    }

    pub fn preference(&self, key: &str) -> Result<Option<String>, StoreError> {
        let value = self
            .connection
            .query_row(
                "SELECT value FROM preferences WHERE key=?1",
                params![key],
                |row| row.get(0),
            )
            .optional()?;
        Ok(value)
    }

    pub fn set_preference(&self, key: &str, value: &str) -> Result<(), StoreError> {
        self.connection.execute(
            "INSERT INTO preferences(key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value=excluded.value",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn preferences_with_prefix(
        &self,
        prefix: &str,
    ) -> Result<Vec<(String, String)>, StoreError> {
        let mut query = self
            .connection
            .prepare("SELECT key, value FROM preferences WHERE instr(key, ?1)=1 ORDER BY key")?;
        let rows = query.query_map(params![prefix], |row| Ok((row.get(0)?, row.get(1)?)))?;
        rows.map(|row| row.map_err(StoreError::from)).collect()
    }
}

use rusqlite::OptionalExtension;
