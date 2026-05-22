/// Status of an individual match within a bracket.
#[derive(Debug, Clone, PartialEq)]
pub enum MatchStatus {
    /// Waiting for both players to be determined.
    Pending,
    /// Both players assigned, ready to play or in play.
    InProgress,
    /// Match completed and result locked.
    Completed,
    /// One player advances automatically (opponent slot empty).
    Bye,
}

impl MatchStatus {
    pub fn as_str(&self) -> &str {
        match self {
            MatchStatus::Pending => "Pending",
            MatchStatus::InProgress => "In Progress",
            MatchStatus::Completed => "Completed",
            MatchStatus::Bye => "Bye",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "In Progress" => MatchStatus::InProgress,
            "Completed" => MatchStatus::Completed,
            "Bye" => MatchStatus::Bye,
            _ => MatchStatus::Pending,
        }
    }
}

/// Indicates whether a match/round belongs to the Upper or Lower bracket, or is the Grand Final.
#[derive(Debug, Clone, PartialEq)]
pub enum BracketType {
    Upper,
    Lower,
    GrandFinal,
}

impl BracketType {
    pub fn as_str(&self) -> &str {
        match self {
            BracketType::Upper => "Upper",
            BracketType::Lower => "Lower",
            BracketType::GrandFinal => "GrandFinal",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "Lower" => BracketType::Lower,
            "GrandFinal" => BracketType::GrandFinal,
            _ => BracketType::Upper,
        }
    }
}

/// A single match within a tournament round.
///
/// Each match tracks two player slots, their scores,
/// the winner, and a link to the next match in the bracket.
#[derive(Debug, Clone)]
pub struct Match {
    pub id: String,
    pub tournament_id: String,
    pub round_id: String,
    pub match_order: i32,
    pub player1_id: Option<String>,
    pub player2_id: Option<String>,
    pub player1_name: String,
    pub player2_name: String,
    pub score1: i32,
    pub score2: i32,
    pub winner_id: Option<String>,
    pub status: MatchStatus,
    pub next_match_id: Option<String>,
    pub next_match_slot: i32, // 1 or 2 — which slot in the next match
    pub loser_next_match_id: Option<String>, // Used for Double Elimination
    pub loser_next_match_slot: i32,
    pub bracket_type: BracketType,
}

impl Match {
    pub fn new(
        tournament_id: String,
        round_id: String,
        match_order: i32,
        next_match_id: Option<String>,
        next_match_slot: i32,
        bracket_type: BracketType,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            tournament_id,
            round_id,
            match_order,
            player1_id: None,
            player2_id: None,
            player1_name: String::new(),
            player2_name: String::new(),
            score1: 0,
            score2: 0,
            winner_id: None,
            status: MatchStatus::Pending,
            next_match_id,
            next_match_slot,
            loser_next_match_id: None,
            loser_next_match_slot: 0,
            bracket_type,
        }
    }
}
