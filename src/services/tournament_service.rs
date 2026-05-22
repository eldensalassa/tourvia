use std::sync::Arc;
use crate::domain::repositories::{TournamentRepository, MatchRepository, RoundRepository, ParticipantRepository};
use crate::domain::tournament::{Tournament, TournamentStatus, TournamentType};

pub struct TournamentService {
    tournament_repo: Arc<dyn TournamentRepository>,
    match_repo: Arc<dyn MatchRepository>,
    round_repo: Arc<dyn RoundRepository>,
    participant_repo: Arc<dyn ParticipantRepository>,
}

impl TournamentService {
    pub fn new(
        tournament_repo: Arc<dyn TournamentRepository>,
        match_repo: Arc<dyn MatchRepository>,
        round_repo: Arc<dyn RoundRepository>,
        participant_repo: Arc<dyn ParticipantRepository>,
    ) -> Self {
        Self {
            tournament_repo,
            match_repo,
            round_repo,
            participant_repo,
        }
    }

    pub fn create_tournament(
        &self,
        name: &str,
        t_type: TournamentType,
        description: &str,
        game_name: &str,
    ) -> Result<Tournament, String> {
        if name.trim().is_empty() {
            return Err("Tournament name cannot be empty.".to_string());
        }

        let tournament = Tournament::new(
            name.trim().to_string(),
            t_type,
            description.trim().to_string(),
            game_name.trim().to_string(),
        );
        self.tournament_repo.create_tournament(&tournament)
            .map_err(|e| format!("Database error: {}", e))?;

        Ok(tournament)
    }

    pub fn load_all_tournaments(&self) -> Result<Vec<Tournament>, String> {
        self.tournament_repo.get_all_tournaments()
            .map_err(|e| format!("Failed to load tournaments: {}", e))
    }

    pub fn delete_tournament(&self, tournament_id: &str) -> Result<(), String> {
        self.tournament_repo.delete_tournament(tournament_id)
            .map_err(|e| format!("Failed to delete tournament: {}", e))
    }

    pub fn update_status(&self, tournament_id: &str, status: TournamentStatus) -> Result<(), String> {
        self.tournament_repo.update_tournament_status(tournament_id, &status)
            .map_err(|e| format!("Failed to update status: {}", e))
    }

    pub fn reset_bracket(&self, tournament_id: &str) -> Result<(), String> {
        self.match_repo.delete_all_matches(tournament_id)
            .map_err(|e| format!("Failed to clear matches: {}", e))?;
        self.round_repo.delete_all_rounds(tournament_id)
            .map_err(|e| format!("Failed to clear rounds: {}", e))?;
        self.tournament_repo.update_tournament_status(tournament_id, &TournamentStatus::Draft)
            .map_err(|e| format!("Failed to reset status: {}", e))?;
        Ok(())
    }

    pub fn export_tournament_json(&self, tournament_id: &str) -> Result<String, String> {
        let tournament = self.tournament_repo.get_tournament(tournament_id)
            .map_err(|e| format!("DB error: {}", e))?
            .ok_or("Tournament not found")?;

        let participants = self.participant_repo.get_participants_by_tournament(tournament_id)
            .map_err(|e| format!("DB error: {}", e))?;

        let rounds = self.round_repo.get_rounds_by_tournament(tournament_id)
            .map_err(|e| format!("DB error: {}", e))?;

        let matches = self.match_repo.get_matches_by_tournament(tournament_id)
            .map_err(|e| format!("DB error: {}", e))?;

        let json = serde_json::json!({
            "tournament": {
                "id": tournament.id,
                "name": tournament.name,
                "type": tournament.tournament_type.as_str(),
                "status": tournament.status.as_str(),
                "participant_count": tournament.participant_count,
                "created_at": tournament.created_at,
                "description": tournament.description,
                "game_name": tournament.game_name,
            },
            "participants": participants.iter().map(|p| {
                serde_json::json!({
                    "id": p.id,
                    "name": p.name,
                    "seed": p.seed,
                })
            }).collect::<Vec<_>>(),
            "rounds": rounds.iter().map(|r| {
                serde_json::json!({
                    "id": r.id,
                    "round_number": r.round_number,
                    "name": r.name,
                    "bracket_type": r.bracket_type.as_str(),
                })
            }).collect::<Vec<_>>(),
            "matches": matches.iter().map(|m| {
                serde_json::json!({
                    "id": m.id,
                    "round_id": m.round_id,
                    "match_order": m.match_order,
                    "player1_name": m.player1_name,
                    "player2_name": m.player2_name,
                    "score1": m.score1,
                    "score2": m.score2,
                    "status": m.status.as_str(),
                    "winner_id": m.winner_id,
                    "bracket_type": m.bracket_type.as_str(),
                })
            }).collect::<Vec<_>>(),
        });

        serde_json::to_string_pretty(&json)
            .map_err(|e| format!("JSON error: {}", e))
    }
}
