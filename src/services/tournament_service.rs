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

        let export_data = crate::domain::export_model::TournamentExport {
            tournament,
            participants,
            rounds,
            matches,
        };

        serde_json::to_string_pretty(&export_data)
            .map_err(|e| format!("JSON error: {}", e))
    }

    pub fn import_tournament_json(&self, json_str: &str) -> Result<String, String> {
        let export_data: crate::domain::export_model::TournamentExport = serde_json::from_str(json_str)
            .map_err(|e| format!("Failed to parse JSON: {}", e))?;
        
        let t = &export_data.tournament;
        
        // Ensure it doesn't already exist or fail gracefully
        if self.tournament_repo.get_tournament(&t.id).map_err(|e| e.to_string())?.is_some() {
            // We could regenerate IDs here, but to keep it simple and perfectly matching:
            // we assume if it exists, we error out or delete the old one. Let's return error.
            return Err("Tournament with this ID already exists. Cannot import.".to_string());
        }

        self.tournament_repo.create_tournament(t)
            .map_err(|e| format!("Failed to insert tournament: {}", e))?;

        for p in &export_data.participants {
            self.participant_repo.create_participant(p)
                .map_err(|e| format!("Failed to insert participant: {}", e))?;
        }

        for r in &export_data.rounds {
            self.round_repo.create_round(r)
                .map_err(|e| format!("Failed to insert round: {}", e))?;
        }

        for m in &export_data.matches {
            self.match_repo.create_match(m)
                .map_err(|e| format!("Failed to insert match: {}", e))?;
        }

        Ok(t.id.clone())
    }
}
