use std::sync::Arc;
use crate::domain::match_model::MatchStatus;
use crate::domain::repositories::{MatchRepository, ParticipantRepository, TournamentRepository};
use crate::domain::tournament::TournamentType;

pub struct TournamentStats {
    pub total_matches: usize,
    pub completed_matches: usize,
    pub pending_matches: usize,
    pub in_progress_matches: usize,
    pub bye_matches: usize,
    /// (participant_id, participant_name, points, wins, losses, draws)
    pub standings: Vec<(String, String, i32, i32, i32, i32)>,
}

pub struct MatchService {
    match_repo: Arc<dyn MatchRepository>,
    participant_repo: Arc<dyn ParticipantRepository>,
    tournament_repo: Arc<dyn TournamentRepository>,
}

impl MatchService {
    pub fn new(
        match_repo: Arc<dyn MatchRepository>,
        participant_repo: Arc<dyn ParticipantRepository>,
        tournament_repo: Arc<dyn TournamentRepository>,
    ) -> Self {
        Self {
            match_repo,
            participant_repo,
            tournament_repo,
        }
    }

    pub fn submit_score(
        &self,
        match_id: &str,
        score1: i32,
        score2: i32,
    ) -> Result<(), String> {
        if score1 < 0 || score2 < 0 {
            return Err("Scores cannot be negative.".to_string());
        }

        let m = self.match_repo.get_match_by_id(match_id)
            .map_err(|e| format!("Database error: {}", e))?
            .ok_or("Match not found.")?;

        if m.status == MatchStatus::Completed {
            return Err("This match is already completed and locked.".to_string());
        }
        if m.status == MatchStatus::Bye {
            return Err("Cannot submit score for a BYE match.".to_string());
        }
        if m.player1_id.is_none() || m.player2_id.is_none() {
            return Err("Both players must be assigned before submitting a score.".to_string());
        }

        // For Round Robin, ties might be allowed, but for elimination they are not.
        // We will assume that if scores are equal, it's a tie (draw).
        // For elimination, a tie is usually not allowed.
        let is_tie = score1 == score2;
        
        // Ensure ties are not allowed in elimination tournaments
        let tournament = self.tournament_repo.get_tournament(&m.tournament_id)
            .map_err(|e| format!("Database error: {}", e))?
            .ok_or("Tournament not found.")?;
            
        let is_round_robin = tournament.tournament_type == TournamentType::RoundRobin;
        
        if is_tie && !is_round_robin {
            return Err("Ties are not allowed in elimination brackets. Please enter a decisive score.".to_string());
        }

        let mut winner_id = None;
        let mut winner_name = String::new();
        let mut loser_id = None;
        let mut loser_name = String::new();

        if !is_tie {
            if score1 > score2 {
                winner_id = Some(m.player1_id.as_ref().unwrap().clone());
                winner_name = m.player1_name.clone();
                loser_id = Some(m.player2_id.as_ref().unwrap().clone());
                loser_name = m.player2_name.clone();
            } else {
                winner_id = Some(m.player2_id.as_ref().unwrap().clone());
                winner_name = m.player2_name.clone();
                loser_id = Some(m.player1_id.as_ref().unwrap().clone());
                loser_name = m.player1_name.clone();
            }
        }

        self.match_repo.update_match_score(match_id, score1, score2, &MatchStatus::Completed, winner_id.as_deref())
            .map_err(|e| format!("Failed to update score: {}", e))?;

        if !is_tie {
            // Advance winner
            if let Some(ref next_match_id) = m.next_match_id {
                self.match_repo.set_match_player(next_match_id, m.next_match_slot, winner_id.as_ref().unwrap(), &winner_name)
                    .map_err(|e| format!("Failed to advance winner: {}", e))?;
            }

            // Drop loser to loser's bracket (for double elimination)
            if let Some(ref loser_next_match_id) = m.loser_next_match_id {
                self.match_repo.set_match_player(loser_next_match_id, m.loser_next_match_slot, loser_id.as_ref().unwrap(), &loser_name)
                    .map_err(|e| format!("Failed to drop loser: {}", e))?;
            }
        }

        Ok(())
    }

    pub fn is_tournament_complete(&self, tournament_id: &str) -> Result<bool, String> {
        let matches = self.match_repo.get_matches_by_tournament(tournament_id)
            .map_err(|e| format!("Database error: {}", e))?;
        Ok(matches.iter().all(|m| m.status == MatchStatus::Completed || m.status == MatchStatus::Bye))
    }

    pub fn get_champion(&self, tournament_id: &str) -> Result<Option<String>, String> {
        let matches = self.match_repo.get_matches_by_tournament(tournament_id)
            .map_err(|e| format!("Database error: {}", e))?;
        let final_match = matches.iter().find(|m| m.next_match_id.is_none());
        if let Some(fm) = final_match {
            if fm.status == MatchStatus::Completed {
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

    pub fn get_tournament_stats(&self, tournament_id: &str) -> Result<TournamentStats, String> {
        let matches = self.match_repo.get_matches_by_tournament(tournament_id)
            .map_err(|e| format!("Database error: {}", e))?;
        let participants = self.participant_repo.get_participants_by_tournament(tournament_id)
            .map_err(|e| format!("Database error: {}", e))?;
        
        let tournament = self.tournament_repo.get_tournament(tournament_id)
            .map_err(|e| format!("Database error: {}", e))?
            .ok_or("Tournament not found")?;

        let is_round_robin = tournament.tournament_type == TournamentType::RoundRobin;

        let total_matches = matches.len();
        let completed_matches = matches.iter().filter(|m| m.status == MatchStatus::Completed).count();
        let pending_matches = matches.iter().filter(|m| m.status == MatchStatus::Pending).count();
        let in_progress_matches = matches.iter().filter(|m| m.status == MatchStatus::InProgress).count();
        let bye_matches = matches.iter().filter(|m| m.status == MatchStatus::Bye).count();

        // Calculate win/loss per participant
        let mut standings: Vec<(String, String, i32, i32, i32, i32)> = participants
            .iter()
            .map(|p| {
                let mut wins = 0;
                let mut losses = 0;
                let mut draws = 0;
                let mut points = 0;

                for m in &matches {
                    if m.status == MatchStatus::Completed {
                        if m.winner_id.as_ref() == Some(&p.id) {
                            wins += 1;
                            points += 3; // 3 points for a win
                        } else if m.winner_id.is_some() && (m.player1_id.as_ref() == Some(&p.id) || m.player2_id.as_ref() == Some(&p.id)) {
                            losses += 1;
                        } else if m.winner_id.is_none() && (m.player1_id.as_ref() == Some(&p.id) || m.player2_id.as_ref() == Some(&p.id)) {
                            // Match is completed but no winner -> draw
                            draws += 1;
                            points += 1; // 1 point for a draw
                        }
                    } else if m.status == MatchStatus::Bye {
                        if m.player1_id.as_ref() == Some(&p.id) || m.player2_id.as_ref() == Some(&p.id) {
                            // Byes do not grant wins or points in the standings table for elimination brackets
                            // In Round Robin they don't give points either, they just mean no match.
                        }
                    }
                }

                (p.id.clone(), p.name.clone(), points, wins, losses, draws)
            })
            .collect();

        // Sort by points descending, then wins, then fewest losses
        standings.sort_by(|a, b| b.2.cmp(&a.2).then(b.3.cmp(&a.3)).then(a.4.cmp(&b.4)));

        Ok(TournamentStats {
            total_matches,
            completed_matches,
            pending_matches,
            in_progress_matches,
            bye_matches,
            standings,
        })
    }
}
