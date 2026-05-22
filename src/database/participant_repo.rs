use rusqlite::params;
use crate::database::Database;
use crate::domain::participant::Participant;
use crate::domain::repositories::ParticipantRepository;

impl ParticipantRepository for Database {
    fn create_participant(&self, p: &Participant) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO participants (id, tournament_id, name, seed)
             VALUES (?1, ?2, ?3, ?4)",
            params![p.id, p.tournament_id, p.name, p.seed],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    fn get_participants_by_tournament(&self, tournament_id: &str) -> Result<Vec<Participant>, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, tournament_id, name, seed,
                    CASE WHEN logo_data IS NOT NULL THEN 1 ELSE 0 END as has_logo
             FROM participants WHERE tournament_id = ?1 ORDER BY seed ASC",
        ).map_err(|e| e.to_string())?;

        let rows = stmt.query_map(params![tournament_id], |row| {
            Ok(Participant {
                id: row.get(0)?,
                tournament_id: row.get(1)?,
                name: row.get(2)?,
                seed: row.get(3)?,
                has_logo: row.get::<_, i32>(4)? != 0,
            })
        }).map_err(|e| e.to_string())?;

        let mut participants = Vec::new();
        for row in rows {
            participants.push(row.map_err(|e| e.to_string())?);
        }
        Ok(participants)
    }

    fn participant_exists(&self, tournament_id: &str, name: &str) -> Result<bool, String> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM participants WHERE tournament_id = ?1 AND LOWER(name) = LOWER(?2)",
            params![tournament_id, name],
            |row| row.get(0),
        ).map_err(|e| e.to_string())?;
        Ok(count > 0)
    }

    fn update_participant(&self, id: &str, name: &str, seed: i32) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE participants SET name = ?1, seed = ?2 WHERE id = ?3",
            params![name, seed, id],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    fn delete_participant(&self, id: &str) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM participants WHERE id = ?1", params![id]).map_err(|e| e.to_string())?;
        Ok(())
    }

    fn delete_all_participants(&self, tournament_id: &str) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM participants WHERE tournament_id = ?1",
            params![tournament_id],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    fn get_participant_count(&self, tournament_id: &str) -> Result<usize, String> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM participants WHERE tournament_id = ?1",
            params![tournament_id],
            |row| row.get(0),
        ).map_err(|e| e.to_string())?;
        Ok(count as usize)
    }

    fn set_participant_logo(&self, participant_id: &str, data: &[u8]) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE participants SET logo_data = ?1 WHERE id = ?2",
            params![data, participant_id],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    fn get_participant_logo(&self, participant_id: &str) -> Result<Option<Vec<u8>>, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT logo_data FROM participants WHERE id = ?1",
        ).map_err(|e| e.to_string())?;
        let mut rows = stmt.query_map(params![participant_id], |row| {
            row.get::<_, Option<Vec<u8>>>(0)
        }).map_err(|e| e.to_string())?;

        match rows.next() {
            Some(Ok(data)) => Ok(data),
            Some(Err(e)) => Err(e.to_string()),
            None => Ok(None),
        }
    }

    fn delete_participant_logo(&self, participant_id: &str) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE participants SET logo_data = NULL WHERE id = ?1",
            params![participant_id],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }
}
