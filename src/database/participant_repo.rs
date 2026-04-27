use rusqlite::{params, Result};

use crate::database::Database;
use crate::domain::participant::Participant;

impl Database {
    /// Insert a new participant.
    pub fn create_participant(&self, p: &Participant) -> Result<()> {
        self.conn.execute(
            "INSERT INTO participants (id, tournament_id, name, seed)
             VALUES (?1, ?2, ?3, ?4)",
            params![p.id, p.tournament_id, p.name, p.seed],
        )?;
        Ok(())
    }

    /// Get all participants for a tournament, ordered by seed.
    pub fn get_participants_by_tournament(&self, tournament_id: &str) -> Result<Vec<Participant>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, tournament_id, name, seed,
                    CASE WHEN logo_data IS NOT NULL THEN 1 ELSE 0 END as has_logo
             FROM participants WHERE tournament_id = ?1 ORDER BY seed ASC",
        )?;

        let rows = stmt.query_map(params![tournament_id], |row| {
            Ok(Participant {
                id: row.get(0)?,
                tournament_id: row.get(1)?,
                name: row.get(2)?,
                seed: row.get(3)?,
                has_logo: row.get::<_, i32>(4)? != 0,
            })
        })?;

        let mut participants = Vec::new();
        for row in rows {
            participants.push(row?);
        }
        Ok(participants)
    }

    /// Check if a participant name already exists in a tournament.
    pub fn participant_exists(&self, tournament_id: &str, name: &str) -> Result<bool> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM participants WHERE tournament_id = ?1 AND LOWER(name) = LOWER(?2)",
            params![tournament_id, name],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    /// Update participant name and seed.
    pub fn update_participant(&self, id: &str, name: &str, seed: i32) -> Result<()> {
        self.conn.execute(
            "UPDATE participants SET name = ?1, seed = ?2 WHERE id = ?3",
            params![name, seed, id],
        )?;
        Ok(())
    }

    /// Delete a participant.
    pub fn delete_participant(&self, id: &str) -> Result<()> {
        self.conn.execute("DELETE FROM participants WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// Delete all participants for a tournament.
    pub fn delete_all_participants(&self, tournament_id: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM participants WHERE tournament_id = ?1",
            params![tournament_id],
        )?;
        Ok(())
    }

    /// Get participant count for a tournament.
    pub fn get_participant_count(&self, tournament_id: &str) -> Result<usize> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM participants WHERE tournament_id = ?1",
            params![tournament_id],
            |row| row.get(0),
        )?;
        Ok(count as usize)
    }

    // ─── Logo CRUD ──────────────────────────────────────

    /// Store logo PNG data for a participant.
    pub fn set_participant_logo(&self, participant_id: &str, data: &[u8]) -> Result<()> {
        self.conn.execute(
            "UPDATE participants SET logo_data = ?1 WHERE id = ?2",
            params![data, participant_id],
        )?;
        Ok(())
    }

    /// Get logo PNG data for a participant.
    pub fn get_participant_logo(&self, participant_id: &str) -> Result<Option<Vec<u8>>> {
        let mut stmt = self.conn.prepare(
            "SELECT logo_data FROM participants WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(params![participant_id], |row| {
            row.get::<_, Option<Vec<u8>>>(0)
        })?;

        match rows.next() {
            Some(Ok(data)) => Ok(data),
            Some(Err(e)) => Err(e),
            None => Ok(None),
        }
    }

    /// Remove logo data for a participant.
    pub fn delete_participant_logo(&self, participant_id: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE participants SET logo_data = NULL WHERE id = ?1",
            params![participant_id],
        )?;
        Ok(())
    }
}
