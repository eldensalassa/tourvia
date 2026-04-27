use rusqlite::{params, Result};

use crate::database::Database;
use crate::domain::tournament::{Tournament, TournamentStatus, TournamentType};

impl Database {
    /// Insert a new tournament into the database.
    pub fn create_tournament(&self, t: &Tournament) -> Result<()> {
        self.conn.execute(
            "INSERT INTO tournaments (id, name, tournament_type, participant_count, status, created_at, description, game_name)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                t.id,
                t.name,
                t.tournament_type.as_str(),
                t.participant_count as i64,
                t.status.as_str(),
                t.created_at,
                t.description,
                t.game_name,
            ],
        )?;
        Ok(())
    }

    /// Get a tournament by its ID.
    pub fn get_tournament(&self, id: &str) -> Result<Option<Tournament>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, tournament_type, participant_count, status, created_at,
                    COALESCE(description, '') as description, COALESCE(game_name, '') as game_name
             FROM tournaments WHERE id = ?1",
        )?;

        let mut rows = stmt.query_map(params![id], |row| {
            Ok(Tournament {
                id: row.get(0)?,
                name: row.get(1)?,
                tournament_type: TournamentType::from_str(&row.get::<_, String>(2)?),
                participant_count: row.get::<_, i64>(3)? as usize,
                status: TournamentStatus::from_str(&row.get::<_, String>(4)?),
                created_at: row.get(5)?,
                description: row.get(6)?,
                game_name: row.get(7)?,
            })
        })?;

        match rows.next() {
            Some(Ok(t)) => Ok(Some(t)),
            Some(Err(e)) => Err(e),
            None => Ok(None),
        }
    }

    /// Get all tournaments, ordered by creation date (newest first).
    pub fn get_all_tournaments(&self) -> Result<Vec<Tournament>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, tournament_type, participant_count, status, created_at,
                    COALESCE(description, '') as description, COALESCE(game_name, '') as game_name
             FROM tournaments ORDER BY created_at DESC",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(Tournament {
                id: row.get(0)?,
                name: row.get(1)?,
                tournament_type: TournamentType::from_str(&row.get::<_, String>(2)?),
                participant_count: row.get::<_, i64>(3)? as usize,
                status: TournamentStatus::from_str(&row.get::<_, String>(4)?),
                created_at: row.get(5)?,
                description: row.get(6)?,
                game_name: row.get(7)?,
            })
        })?;

        let mut tournaments = Vec::new();
        for row in rows {
            tournaments.push(row?);
        }
        Ok(tournaments)
    }

    /// Update tournament status.
    pub fn update_tournament_status(&self, id: &str, status: &TournamentStatus) -> Result<()> {
        self.conn.execute(
            "UPDATE tournaments SET status = ?1 WHERE id = ?2",
            params![status.as_str(), id],
        )?;
        Ok(())
    }

    /// Update the participant count for a tournament.
    pub fn update_tournament_participant_count(&self, id: &str, count: usize) -> Result<()> {
        self.conn.execute(
            "UPDATE tournaments SET participant_count = ?1 WHERE id = ?2",
            params![count as i64, id],
        )?;
        Ok(())
    }

    /// Delete a tournament and all related data (cascade).
    pub fn delete_tournament(&self, id: &str) -> Result<()> {
        // Delete in order due to foreign keys (even though CASCADE is set)
        self.conn.execute("DELETE FROM matches WHERE tournament_id = ?1", params![id])?;
        self.conn.execute("DELETE FROM rounds WHERE tournament_id = ?1", params![id])?;
        self.conn.execute("DELETE FROM participants WHERE tournament_id = ?1", params![id])?;
        self.conn.execute("DELETE FROM tournaments WHERE id = ?1", params![id])?;
        Ok(())
    }
}
