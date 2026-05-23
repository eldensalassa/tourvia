use serde::{Serialize, Deserialize};

/// Represents the type of tournament bracket structure.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TournamentType {
    SingleElimination,
    DoubleElimination,
    RoundRobin,
}

impl TournamentType {
    pub fn as_str(&self) -> &str {
        match self {
            TournamentType::SingleElimination => "Single Elimination",
            TournamentType::DoubleElimination => "Double Elimination",
            TournamentType::RoundRobin => "Round Robin",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "Double Elimination" => TournamentType::DoubleElimination,
            "Round Robin" => TournamentType::RoundRobin,
            _ => TournamentType::SingleElimination,
        }
    }
}

/// Current status of a tournament.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TournamentStatus {
    /// Tournament created but bracket not yet generated.
    Draft,
    /// Bracket generated and matches are being played.
    InProgress,
    /// All matches completed, champion determined.
    Completed,
}

impl TournamentStatus {
    pub fn as_str(&self) -> &str {
        match self {
            TournamentStatus::Draft => "Draft",
            TournamentStatus::InProgress => "In Progress",
            TournamentStatus::Completed => "Completed",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "In Progress" => TournamentStatus::InProgress,
            "Completed" => TournamentStatus::Completed,
            _ => TournamentStatus::Draft,
        }
    }
}

/// Core tournament entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tournament {
    pub id: String,
    pub name: String,
    pub tournament_type: TournamentType,
    pub participant_count: usize,
    pub status: TournamentStatus,
    pub created_at: String,
    pub description: String,
    pub game_name: String,
}

impl Tournament {
    pub fn new(name: String, tournament_type: TournamentType, description: String, game_name: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            tournament_type,
            participant_count: 0,
            status: TournamentStatus::Draft,
            created_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            description,
            game_name,
        }
    }
}
