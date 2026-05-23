use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Roster {
    pub id: String,
    pub name: String,
    pub game: String,
    #[serde(skip)]
    pub logo_data: Option<Vec<u8>>,
}

impl Roster {
    pub fn new(name: String, game: String, logo_data: Option<Vec<u8>>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            game,
            logo_data,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RosterMember {
    pub id: String,
    pub roster_id: String,
    pub name: String,
    #[serde(skip)]
    pub profile_picture: Option<Vec<u8>>,
}

impl RosterMember {
    pub fn new(roster_id: String, name: String, profile_picture: Option<Vec<u8>>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            roster_id,
            name,
            profile_picture,
        }
    }
}
