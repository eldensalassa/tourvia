/// A participant (player/team) in a tournament.
#[derive(Debug, Clone)]
pub struct Participant {
    pub id: String,
    pub tournament_id: String,
    pub name: String,
    pub seed: i32,
    /// Whether this participant has a logo stored in the database.
    pub has_logo: bool,
}

impl Participant {
    pub fn new(tournament_id: String, name: String, seed: i32) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            tournament_id,
            name,
            seed,
            has_logo: false,
        }
    }
}
