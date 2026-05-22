/// A round within a tournament bracket (e.g., Quarter Final, Semi Final, Final).
#[derive(Debug, Clone)]
pub struct Round {
    pub id: String,
    pub tournament_id: String,
    pub round_number: i32,
    pub name: String,
    pub bracket_type: crate::domain::match_model::BracketType,
}

impl Round {
    pub fn new(
        tournament_id: String,
        round_number: i32,
        name: String,
        bracket_type: crate::domain::match_model::BracketType,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            tournament_id,
            round_number,
            name,
            bracket_type,
        }
    }
}
