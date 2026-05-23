use super::match_model::Match;
use super::participant::Participant;
use super::round::Round;
use super::tournament::{Tournament, TournamentStatus};

pub trait TournamentRepository: Send + Sync {
    fn create_tournament(&self, t: &Tournament) -> Result<(), String>;
    fn get_tournament(&self, id: &str) -> Result<Option<Tournament>, String>;
    fn get_all_tournaments(&self) -> Result<Vec<Tournament>, String>;
    fn update_tournament_status(&self, id: &str, status: &TournamentStatus) -> Result<(), String>;
    fn update_tournament_participant_count(&self, id: &str, count: usize) -> Result<(), String>;
    fn delete_tournament(&self, id: &str) -> Result<(), String>;
}

pub trait ParticipantRepository: Send + Sync {
    fn create_participant(&self, p: &Participant) -> Result<(), String>;
    fn get_participants_by_tournament(&self, tournament_id: &str) -> Result<Vec<Participant>, String>;
    fn participant_exists(&self, tournament_id: &str, name: &str) -> Result<bool, String>;
    fn update_participant(&self, id: &str, name: &str, seed: i32) -> Result<(), String>;
    fn delete_participant(&self, id: &str) -> Result<(), String>;
    fn delete_all_participants(&self, tournament_id: &str) -> Result<(), String>;
    fn get_participant_count(&self, tournament_id: &str) -> Result<usize, String>;
    fn set_participant_logo(&self, participant_id: &str, data: &[u8]) -> Result<(), String>;
    fn get_participant_logo(&self, participant_id: &str) -> Result<Option<Vec<u8>>, String>;
    fn delete_participant_logo(&self, participant_id: &str) -> Result<(), String>;
}

pub trait RoundRepository: Send + Sync {
    fn create_round(&self, r: &Round) -> Result<(), String>;
    fn get_rounds_by_tournament(&self, tournament_id: &str) -> Result<Vec<Round>, String>;
    fn delete_all_rounds(&self, tournament_id: &str) -> Result<(), String>;
}

pub trait MatchRepository: Send + Sync {
    fn create_match(&self, m: &Match) -> Result<(), String>;
    fn get_matches_by_tournament(&self, tournament_id: &str) -> Result<Vec<Match>, String>;
    fn get_match_by_id(&self, match_id: &str) -> Result<Option<Match>, String>;
    fn update_match_score(&self, id: &str, score1: i32, score2: i32, status: &super::match_model::MatchStatus, winner_id: Option<&str>) -> Result<(), String>;
    fn set_match_player(&self, id: &str, slot: i32, player_id: &str, player_name: &str) -> Result<(), String>;
    fn delete_all_matches(&self, tournament_id: &str) -> Result<(), String>;
}

pub trait RosterRepository: Send + Sync {
    fn create_roster(&self, roster: &crate::domain::roster::Roster) -> Result<(), String>;
    fn get_rosters(&self) -> Result<Vec<crate::domain::roster::Roster>, String>;
    fn update_roster(&self, roster: &crate::domain::roster::Roster) -> Result<(), String>;
    fn delete_roster(&self, id: &str) -> Result<(), String>;
    
    // Roster Members
    fn add_roster_member(&self, member: &crate::domain::roster::RosterMember) -> Result<(), String>;
    fn get_roster_members(&self, roster_id: &str) -> Result<Vec<crate::domain::roster::RosterMember>, String>;
    fn update_roster_member(&self, member: &crate::domain::roster::RosterMember) -> Result<(), String>;
    fn delete_roster_member(&self, member_id: &str) -> Result<(), String>;
}

pub trait GameRepository: Send + Sync {
    fn create_game(&self, game: &crate::domain::game::Game) -> Result<(), String>;
    fn get_games(&self) -> Result<Vec<crate::domain::game::Game>, String>;
    fn delete_game(&self, id: &str) -> Result<(), String>;
}
