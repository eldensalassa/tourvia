use crate::database::Database;
use crate::domain::tournament::{Tournament, TournamentStatus, TournamentType};

/// Create a new tournament and persist it.
pub fn create_tournament(
    db: &Database,
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
    db.create_tournament(&tournament)
        .map_err(|e| format!("Database error: {}", e))?;

    Ok(tournament)
}

/// Load all tournaments from the database.
pub fn load_all_tournaments(db: &Database) -> Result<Vec<Tournament>, String> {
    db.get_all_tournaments()
        .map_err(|e| format!("Failed to load tournaments: {}", e))
}

/// Delete a tournament and all its related data.
pub fn delete_tournament(db: &Database, tournament_id: &str) -> Result<(), String> {
    db.delete_tournament(tournament_id)
        .map_err(|e| format!("Failed to delete tournament: {}", e))
}

/// Update the status of a tournament.
pub fn update_status(db: &Database, tournament_id: &str, status: TournamentStatus) -> Result<(), String> {
    db.update_tournament_status(tournament_id, &status)
        .map_err(|e| format!("Failed to update status: {}", e))
}

/// Reset a tournament bracket: delete all rounds/matches, set status back to Draft.
pub fn reset_bracket(db: &Database, tournament_id: &str) -> Result<(), String> {
    db.delete_all_matches(tournament_id)
        .map_err(|e| format!("Failed to clear matches: {}", e))?;
    db.delete_all_rounds(tournament_id)
        .map_err(|e| format!("Failed to clear rounds: {}", e))?;
    db.update_tournament_status(tournament_id, &TournamentStatus::Draft)
        .map_err(|e| format!("Failed to reset status: {}", e))?;
    Ok(())
}

/// Export tournament data as a JSON string.
pub fn export_tournament_json(db: &Database, tournament_id: &str) -> Result<String, String> {
    let tournament = db.get_tournament(tournament_id)
        .map_err(|e| format!("DB error: {}", e))?
        .ok_or("Tournament not found")?;

    let participants = db.get_participants_by_tournament(tournament_id)
        .map_err(|e| format!("DB error: {}", e))?;

    let rounds = db.get_rounds_by_tournament(tournament_id)
        .map_err(|e| format!("DB error: {}", e))?;

    let matches = db.get_matches_by_tournament(tournament_id)
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
            })
        }).collect::<Vec<_>>(),
    });

    serde_json::to_string_pretty(&json)
        .map_err(|e| format!("JSON error: {}", e))
}
