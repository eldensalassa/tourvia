use crate::database::Database;
use crate::domain::match_model::MatchStatus;

/// Submit a score for a match, determine the winner, lock the match,
/// and advance the winner to the next round.
pub fn submit_score(
    db: &Database,
    match_id: &str,
    score1: i32,
    score2: i32,
) -> Result<(), String> {
    // Validate scores
    if score1 < 0 || score2 < 0 {
        return Err("Scores cannot be negative.".to_string());
    }
    if score1 == score2 {
        return Err("Scores cannot be tied. There must be a winner.".to_string());
    }

    // Get the match
    let m = db
        .get_match_by_id(match_id)
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or("Match not found.")?;

    // Validate match state
    if m.status == MatchStatus::Completed {
        return Err("This match is already completed and locked.".to_string());
    }
    if m.status == MatchStatus::Bye {
        return Err("Cannot submit score for a BYE match.".to_string());
    }
    if m.player1_id.is_none() || m.player2_id.is_none() {
        return Err("Both players must be assigned before submitting a score.".to_string());
    }

    // Determine winner
    let (winner_id, winner_name) = if score1 > score2 {
        (m.player1_id.as_ref().unwrap().clone(), m.player1_name.clone())
    } else {
        (m.player2_id.as_ref().unwrap().clone(), m.player2_name.clone())
    };

    // Update match in database
    db.update_match_score(match_id, score1, score2, Some(&winner_id), &MatchStatus::Completed)
        .map_err(|e| format!("Failed to update score: {}", e))?;

    // Advance winner to next match
    if let Some(ref next_match_id) = m.next_match_id {
        db.set_match_player(next_match_id, m.next_match_slot, &winner_id, &winner_name)
            .map_err(|e| format!("Failed to advance winner: {}", e))?;
    }

    Ok(())
}

/// Check if all matches in a tournament are completed.
pub fn is_tournament_complete(db: &Database, tournament_id: &str) -> Result<bool, String> {
    let matches = db
        .get_matches_by_tournament(tournament_id)
        .map_err(|e| format!("Database error: {}", e))?;

    Ok(matches
        .iter()
        .all(|m| m.status == MatchStatus::Completed || m.status == MatchStatus::Bye))
}

/// Get the champion (winner of the final match).
pub fn get_champion(db: &Database, tournament_id: &str) -> Result<Option<String>, String> {
    let matches = db
        .get_matches_by_tournament(tournament_id)
        .map_err(|e| format!("Database error: {}", e))?;

    // The final match is the one with no next_match_id
    let final_match = matches.iter().find(|m| m.next_match_id.is_none());

    if let Some(fm) = final_match {
        if fm.status == MatchStatus::Completed {
            // Return winner name
            if let Some(ref winner_id) = fm.winner_id {
                if fm.player1_id.as_ref() == Some(winner_id) {
                    return Ok(Some(fm.player1_name.clone()));
                } else {
                    return Ok(Some(fm.player2_name.clone()));
                }
            }
        }
    }

    Ok(None)
}

/// Tournament statistics.
pub struct TournamentStats {
    pub total_matches: usize,
    pub completed_matches: usize,
    pub pending_matches: usize,
    pub in_progress_matches: usize,
    pub bye_matches: usize,
    /// (participant_name, wins, losses)
    pub standings: Vec<(String, i32, i32)>,
}

/// Calculate tournament statistics.
pub fn get_tournament_stats(db: &Database, tournament_id: &str) -> Result<TournamentStats, String> {
    let matches = db
        .get_matches_by_tournament(tournament_id)
        .map_err(|e| format!("Database error: {}", e))?;

    let participants = db
        .get_participants_by_tournament(tournament_id)
        .map_err(|e| format!("Database error: {}", e))?;

    let total_matches = matches.len();
    let completed_matches = matches.iter().filter(|m| m.status == MatchStatus::Completed).count();
    let pending_matches = matches.iter().filter(|m| m.status == MatchStatus::Pending).count();
    let in_progress_matches = matches.iter().filter(|m| m.status == MatchStatus::InProgress).count();
    let bye_matches = matches.iter().filter(|m| m.status == MatchStatus::Bye).count();

    // Calculate win/loss per participant
    let mut standings: Vec<(String, i32, i32)> = participants
        .iter()
        .map(|p| {
            let wins = matches.iter().filter(|m| {
                m.status == MatchStatus::Completed && m.winner_id.as_ref() == Some(&p.id)
            }).count() as i32;

            let losses = matches.iter().filter(|m| {
                m.status == MatchStatus::Completed
                    && m.winner_id.is_some()
                    && m.winner_id.as_ref() != Some(&p.id)
                    && (m.player1_id.as_ref() == Some(&p.id) || m.player2_id.as_ref() == Some(&p.id))
            }).count() as i32;

            (p.name.clone(), wins, losses)
        })
        .collect();

    // Sort by wins descending
    standings.sort_by(|a, b| b.1.cmp(&a.1).then(a.2.cmp(&b.2)));

    Ok(TournamentStats {
        total_matches,
        completed_matches,
        pending_matches,
        in_progress_matches,
        bye_matches,
        standings,
    })
}
