use rusqlite::{Connection, Result};
use std::sync::Mutex;

use super::schema;

/// Wrapper around a SQLite connection for the Tourvia database.
pub struct Database {
    pub conn: Mutex<Connection>,
}

impl Database {
    /// Open (or create) the SQLite database file and initialize the schema.
    pub fn open(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;

        // Enable WAL mode for better concurrent read performance
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;
        // Enable foreign key support
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;

        let db = Database { conn: Mutex::new(conn) };
        db.initialize()?;
        db.migrate();
        Ok(db)
    }

    /// Create all tables if they don't exist.
    fn initialize(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(schema::CREATE_TOURNAMENTS_TABLE)?;
        conn.execute_batch(schema::CREATE_PARTICIPANTS_TABLE)?;
        conn.execute_batch(schema::CREATE_ROUNDS_TABLE)?;
        conn.execute_batch(schema::CREATE_MATCHES_TABLE)?;
        conn.execute_batch(schema::CREATE_ROSTERS_TABLE)?;
        conn.execute_batch(schema::CREATE_ROSTER_MEMBERS_TABLE)?;
        conn.execute_batch(schema::CREATE_GAMES_TABLE)?;
        Ok(())
    }

    /// Run migrations to add new columns to existing tables.
    fn migrate(&self) {
        let conn = self.conn.lock().unwrap();
        for sql in schema::MIGRATIONS {
            let _ = conn.execute_batch(sql);
        }
    }
}
