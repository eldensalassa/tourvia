use rusqlite::params;
use crate::database::Database;
use crate::domain::tournament::{Tournament, TournamentStatus, TournamentType};
use crate::domain::repositories::TournamentRepository;

impl TournamentRepository for Database {
    fn create_tournament(&self, t: &Tournament) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
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
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    fn get_tournament(&self, id: &str) -> Result<Option<Tournament>, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, tournament_type, participant_count, status, created_at,
                    COALESCE(description, '') as description, COALESCE(game_name, '') as game_name
             FROM tournaments WHERE id = ?1",
        ).map_err(|e| e.to_string())?;

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
        }).map_err(|e| e.to_string())?;

        match rows.next() {
            Some(Ok(t)) => Ok(Some(t)),
            Some(Err(e)) => Err(e.to_string()),
            None => Ok(None),
        }
    }

    fn get_all_tournaments(&self) -> Result<Vec<Tournament>, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, tournament_type, participant_count, status, created_at,
                    COALESCE(description, '') as description, COALESCE(game_name, '') as game_name
             FROM tournaments ORDER BY created_at DESC",
        ).map_err(|e| e.to_string())?;

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
        }).map_err(|e| e.to_string())?;

        let mut tournaments = Vec::new();
        for row in rows {
            tournaments.push(row.map_err(|e| e.to_string())?);
        }
        Ok(tournaments)
    }

    fn update_tournament_status(&self, id: &str, status: &TournamentStatus) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE tournaments SET status = ?1 WHERE id = ?2",
            params![status.as_str(), id],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    fn update_tournament_participant_count(&self, id: &str, count: usize) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE tournaments SET participant_count = ?1 WHERE id = ?2",
            params![count as i64, id],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    fn delete_tournament(&self, id: &str) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM matches WHERE tournament_id = ?1", params![id]).map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM rounds WHERE tournament_id = ?1", params![id]).map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM participants WHERE tournament_id = ?1", params![id]).map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM tournaments WHERE id = ?1", params![id]).map_err(|e| e.to_string())?;
        Ok(())
    }
}
