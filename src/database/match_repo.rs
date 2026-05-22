use rusqlite::params;
use crate::database::Database;
use crate::domain::match_model::{BracketType, Match, MatchStatus};
use crate::domain::round::Round;
use crate::domain::repositories::{MatchRepository, RoundRepository};

impl RoundRepository for Database {
    fn create_round(&self, r: &Round) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO rounds (id, tournament_id, round_number, name, bracket_type)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![r.id, r.tournament_id, r.round_number, r.name, r.bracket_type.as_str()],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    fn get_rounds_by_tournament(&self, tournament_id: &str) -> Result<Vec<Round>, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, tournament_id, round_number, name, bracket_type
             FROM rounds WHERE tournament_id = ?1 ORDER BY round_number ASC",
        ).map_err(|e| e.to_string())?;

        let rows = stmt.query_map(params![tournament_id], |row| {
            Ok(Round {
                id: row.get(0)?,
                tournament_id: row.get(1)?,
                round_number: row.get(2)?,
                name: row.get(3)?,
                bracket_type: BracketType::from_str(&row.get::<_, String>(4)?),
            })
        }).map_err(|e| e.to_string())?;

        let mut rounds = Vec::new();
        for row in rows {
            rounds.push(row.map_err(|e| e.to_string())?);
        }
        Ok(rounds)
    }

    fn delete_all_rounds(&self, tournament_id: &str) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM rounds WHERE tournament_id = ?1",
            params![tournament_id],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }
}

impl MatchRepository for Database {
    fn create_match(&self, m: &Match) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO matches (id, tournament_id, round_id, match_order, player1_id, player2_id,
             player1_name, player2_name, score1, score2, winner_id, status, next_match_id, next_match_slot,
             loser_next_match_id, loser_next_match_slot, bracket_type)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
            params![
                m.id,
                m.tournament_id,
                m.round_id,
                m.match_order,
                m.player1_id,
                m.player2_id,
                m.player1_name,
                m.player2_name,
                m.score1,
                m.score2,
                m.winner_id,
                m.status.as_str(),
                m.next_match_id,
                m.next_match_slot,
                m.loser_next_match_id,
                m.loser_next_match_slot,
                m.bracket_type.as_str(),
            ],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    fn get_matches_by_tournament(&self, tournament_id: &str) -> Result<Vec<Match>, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT m.id, m.tournament_id, m.round_id, m.match_order,
                    m.player1_id, m.player2_id, m.player1_name, m.player2_name,
                    m.score1, m.score2, m.winner_id, m.status, m.next_match_id, m.next_match_slot,
                    m.loser_next_match_id, m.loser_next_match_slot, m.bracket_type
             FROM matches m
             JOIN rounds r ON m.round_id = r.id
             WHERE m.tournament_id = ?1
             ORDER BY r.round_number ASC, m.match_order ASC",
        ).map_err(|e| e.to_string())?;

        let rows = stmt.query_map(params![tournament_id], |row| {
            Ok(Match {
                id: row.get(0)?,
                tournament_id: row.get(1)?,
                round_id: row.get(2)?,
                match_order: row.get(3)?,
                player1_id: row.get(4)?,
                player2_id: row.get(5)?,
                player1_name: row.get(6)?,
                player2_name: row.get(7)?,
                score1: row.get(8)?,
                score2: row.get(9)?,
                winner_id: row.get(10)?,
                status: MatchStatus::from_str(&row.get::<_, String>(11)?),
                next_match_id: row.get(12)?,
                next_match_slot: row.get(13)?,
                loser_next_match_id: row.get(14)?,
                loser_next_match_slot: row.get(15)?,
                bracket_type: BracketType::from_str(&row.get::<_, String>(16)?),
            })
        }).map_err(|e| e.to_string())?;

        let mut matches = Vec::new();
        for row in rows {
            matches.push(row.map_err(|e| e.to_string())?);
        }
        Ok(matches)
    }

    fn update_match_score(
        &self,
        match_id: &str,
        score1: i32,
        score2: i32,
        status: &MatchStatus,
        winner_id: Option<&str>,
    ) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE matches SET score1 = ?1, score2 = ?2, winner_id = ?3, status = ?4 WHERE id = ?5",
            params![score1, score2, winner_id, status.as_str(), match_id],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    fn set_match_player(
        &self,
        match_id: &str,
        slot: i32,
        player_id: &str,
        player_name: &str,
    ) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        if slot == 1 {
            conn.execute(
                "UPDATE matches SET player1_id = ?1, player1_name = ?2, status = CASE
                    WHEN player2_id IS NOT NULL THEN 'In Progress'
                    ELSE status
                 END WHERE id = ?3",
                params![player_id, player_name, match_id],
            ).map_err(|e| e.to_string())?;
        } else {
            conn.execute(
                "UPDATE matches SET player2_id = ?1, player2_name = ?2, status = CASE
                    WHEN player1_id IS NOT NULL THEN 'In Progress'
                    ELSE status
                 END WHERE id = ?3",
                params![player_id, player_name, match_id],
            ).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    fn delete_all_matches(&self, tournament_id: &str) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM matches WHERE tournament_id = ?1",
            params![tournament_id],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    fn get_match_by_id(&self, match_id: &str) -> Result<Option<Match>, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, tournament_id, round_id, match_order,
                    player1_id, player2_id, player1_name, player2_name,
                    score1, score2, winner_id, status, next_match_id, next_match_slot,
                    loser_next_match_id, loser_next_match_slot, bracket_type
             FROM matches WHERE id = ?1",
        ).map_err(|e| e.to_string())?;

        let mut rows = stmt.query_map(params![match_id], |row| {
            Ok(Match {
                id: row.get(0)?,
                tournament_id: row.get(1)?,
                round_id: row.get(2)?,
                match_order: row.get(3)?,
                player1_id: row.get(4)?,
                player2_id: row.get(5)?,
                player1_name: row.get(6)?,
                player2_name: row.get(7)?,
                score1: row.get(8)?,
                score2: row.get(9)?,
                winner_id: row.get(10)?,
                status: MatchStatus::from_str(&row.get::<_, String>(11)?),
                next_match_id: row.get(12)?,
                next_match_slot: row.get(13)?,
                loser_next_match_id: row.get(14)?,
                loser_next_match_slot: row.get(15)?,
                bracket_type: BracketType::from_str(&row.get::<_, String>(16)?),
            })
        }).map_err(|e| e.to_string())?;

        match rows.next() {
            Some(Ok(m)) => Ok(Some(m)),
            Some(Err(e)) => Err(e.to_string()),
            None => Ok(None),
        }
    }
}
