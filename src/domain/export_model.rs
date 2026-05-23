use serde::{Serialize, Deserialize};

use crate::domain::tournament::Tournament;
use crate::domain::participant::Participant;
use crate::domain::round::Round;
use crate::domain::match_model::Match;

#[derive(Serialize, Deserialize)]
pub struct TournamentExport {
    pub tournament: Tournament,
    pub participants: Vec<Participant>,
    pub rounds: Vec<Round>,
    pub matches: Vec<Match>,
}
