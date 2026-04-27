use rusqlite::{Connection, Result};

use super::schema;

/// Wrapper around a SQLite connection for the Tourvia database.
pub struct Database {
    pub conn: Connection,
}

impl Database {
    /// Open (or create) the SQLite database file and initialize the schema.
    pub fn open(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;

        // Enable WAL mode for better concurrent read performance
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;
        // Enable foreign key support
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;

        let db = Database { conn };
        db.initialize()?;
        db.migrate();
        Ok(db)
    }

    /// Create all tables if they don't exist.
    fn initialize(&self) -> Result<()> {
        self.conn.execute_batch(schema::CREATE_TOURNAMENTS_TABLE)?;
        self.conn.execute_batch(schema::CREATE_PARTICIPANTS_TABLE)?;
        self.conn.execute_batch(schema::CREATE_ROUNDS_TABLE)?;
        self.conn.execute_batch(schema::CREATE_MATCHES_TABLE)?;
        Ok(())
    }

    /// Run migrations to add new columns to existing tables.
    /// Each migration is idempotent — silently ignores "duplicate column" errors.
    fn migrate(&self) {
        for sql in schema::MIGRATIONS {
            // Ignore errors (column already exists)
            let _ = self.conn.execute_batch(sql);
        }
    }
}
