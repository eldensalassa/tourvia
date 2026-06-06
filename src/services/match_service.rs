#![allow(dead_code)]

use std::sync::Arc;
use crate::domain::match_model::MatchStatus;
use crate::domain::repositories::{MatchRepository, ParticipantRepository, TournamentRepository, RoundRepository};
use crate::domain::tournament::TournamentType;

#[derive(Clone, Debug)]
pub struct ParticipantStanding {
    pub id: String,
    pub name: String,
    pub matches_played: i32,
    pub matches_won: i32,
    pub matches_lost: i32,
    pub matches_drawn: i32,
    pub games_won: i32,
    pub games_lost: i32,
    pub bracket_score: i32,
}

pub struct TournamentStats {
    pub total_matches: usize,
    pub completed_matches: usize,
    pub pending_matches: usize,
    pub in_progress_matches: usize,
    pub bye_matches: usize,
    pub standings: Vec<ParticipantStanding>,
}

pub struct MatchService {
    match_repo: Arc<dyn MatchRepository>,
    participant_repo: Arc<dyn ParticipantRepository>,
    tournament_repo: Arc<dyn TournamentRepository>,
    round_repo: Arc<dyn RoundRepository>,
}

impl MatchService {
    pub fn new(
        match_repo: Arc<dyn MatchRepository>,
        participant_repo: Arc<dyn ParticipantRepository>,
        tournament_repo: Arc<dyn TournamentRepository>,
        round_repo: Arc<dyn RoundRepository>,
    ) -> Self {
        Self {
            match_repo,
            participant_repo,
            tournament_repo,
            round_repo,
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

        // Trigger BYE sweep to auto-resolve any matches that just became Team vs BYE
        sweep_byes(&self.match_repo, &m.tournament_id);

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

        // A champion can only exist when ALL matches are completed
        let all_done = matches.iter().all(|m| m.status == MatchStatus::Completed || m.status == MatchStatus::Bye);
        if !all_done {
            return Ok(None);
        }

        let tournament = self.tournament_repo.get_tournament(tournament_id)
            .map_err(|e| format!("Database error: {}", e))?
            .ok_or("Tournament not found")?;

        if tournament.tournament_type == TournamentType::RoundRobin {
            // For Round Robin: champion = participant with the most wins
            // Tiebreaker: game differential (games_won - games_lost)
            let participants = self.participant_repo.get_participants_by_tournament(tournament_id)
                .map_err(|e| format!("Database error: {}", e))?;

            let mut best_name: Option<String> = None;
            let mut best_wins: i32 = -1;
            let mut best_diff: i32 = i32::MIN;

            for p in &participants {
                let mut wins = 0i32;
                let mut gw = 0i32;
                let mut gl = 0i32;

                for m in &matches {
                    if m.status != MatchStatus::Completed { continue; }
                    if m.player1_id.as_ref() == Some(&p.id) {
                        gw += m.score1;
                        gl += m.score2;
                        if m.winner_id.as_ref() == Some(&p.id) { wins += 1; }
                    } else if m.player2_id.as_ref() == Some(&p.id) {
                        gw += m.score2;
                        gl += m.score1;
                        if m.winner_id.as_ref() == Some(&p.id) { wins += 1; }
                    }
                }

                let diff = gw - gl;
                if wins > best_wins || (wins == best_wins && diff > best_diff) {
                    best_wins = wins;
                    best_diff = diff;
                    best_name = Some(p.name.clone());
                }
            }

            return Ok(best_name);
        }

        // For elimination brackets: champion = winner of the final match
        let final_match = matches.iter().find(|m| m.next_match_id.is_none() && m.bracket_type != crate::domain::match_model::BracketType::ThirdPlace);

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

        let rounds = self.round_repo.get_rounds_by_tournament(tournament_id)
            .map_err(|e| format!("Database error: {}", e))?;

        let _is_round_robin = tournament.tournament_type == TournamentType::RoundRobin;

        let total_matches = matches.len();
        let completed_matches = matches.iter().filter(|m| m.status == MatchStatus::Completed).count();
        let pending_matches = matches.iter().filter(|m| m.status == MatchStatus::Pending).count();
        let in_progress_matches = matches.iter().filter(|m| m.status == MatchStatus::InProgress).count();
        let bye_matches = matches.iter().filter(|m| m.status == MatchStatus::Bye).count();

        // Calculate win/loss per participant
        let mut standings: Vec<ParticipantStanding> = participants
            .iter()
            .map(|p| {
                let mut matches_won = 0;
                let mut matches_lost = 0;
                let mut matches_drawn = 0;
                let mut games_won = 0;
                let mut games_lost = 0;

                for m in &matches {
                    if m.status == MatchStatus::Completed {
                        let mut is_participant = false;
                        let mut p_score = 0;
                        let mut opp_score = 0;

                        if m.player1_id.as_ref() == Some(&p.id) {
                            is_participant = true;
                            p_score = m.score1;
                            opp_score = m.score2;
                        } else if m.player2_id.as_ref() == Some(&p.id) {
                            is_participant = true;
                            p_score = m.score2;
                            opp_score = m.score1;
                        }

                        if is_participant {
                            games_won += p_score;
                            games_lost += opp_score;

                            if m.winner_id.as_ref() == Some(&p.id) {
                                matches_won += 1;
                            } else if m.winner_id.is_some() {
                                matches_lost += 1;
                            } else {
                                matches_drawn += 1;
                            }
                        }
                    }
                }

                let mut bracket_score = 0;
                let mut is_eliminated = false;
                let mut elimination_score = 0;

                if tournament.tournament_type != TournamentType::RoundRobin {
                    for m in &matches {
                        if m.player1_id.as_ref() == Some(&p.id) || m.player2_id.as_ref() == Some(&p.id) {
                            let is_winner = m.winner_id.as_ref() == Some(&p.id);
                            let is_loser = m.winner_id.is_some() && !is_winner;
                            
                            if m.bracket_type == crate::domain::match_model::BracketType::GrandFinal {
                                let score = if is_winner { 100000 } else { 90000 };
                                if score > bracket_score { bracket_score = score; }
                                if is_loser { 
                                    is_eliminated = true;
                                    elimination_score = 90000;
                                }
                            } else if m.bracket_type == crate::domain::match_model::BracketType::ThirdPlace {
                                let score = if is_winner { 80000 } else { 70000 };
                                if score > bracket_score { bracket_score = score; }
                                if is_loser {
                                    is_eliminated = true;
                                    elimination_score = 70000;
                                }
                            } else {
                                let round_num = rounds.iter().find(|r| r.id == m.round_id).map(|r| r.round_number).unwrap_or(0);
                                if tournament.tournament_type == TournamentType::SingleElimination {
                                    let score = round_num * 1000 + if is_winner { 500 } else { 0 };
                                    if score > bracket_score { bracket_score = score; }
                                    if is_loser {
                                        is_eliminated = true;
                                        elimination_score = round_num * 1000;
                                    }
                                } else if tournament.tournament_type == TournamentType::DoubleElimination {
                                    if m.bracket_type == crate::domain::match_model::BracketType::Lower {
                                        let score = 20000 + round_num * 1000 + if is_winner { 500 } else { 0 };
                                        if score > bracket_score { bracket_score = score; }
                                        if is_loser {
                                            is_eliminated = true;
                                            elimination_score = 20000 + round_num * 1000;
                                        }
                                    } else if m.bracket_type == crate::domain::match_model::BracketType::Upper {
                                        let score = 50000 + round_num * 1000 + if is_winner { 500 } else { 0 };
                                        if score > bracket_score { bracket_score = score; }
                                    }
                                }
                            }
                        }
                    }
                }
                
                let final_bracket_score = if is_eliminated {
                    elimination_score
                } else {
                    bracket_score
                };

                let matches_played = matches_won + matches_lost + matches_drawn;

                ParticipantStanding {
                    id: p.id.clone(),
                    name: p.name.clone(),
                    matches_played,
                    matches_won,
                    matches_lost,
                    matches_drawn,
                    games_won,
                    games_lost,
                    bracket_score: final_bracket_score,
                }
            })
            .collect();

        // 1. Bracket Score (for SE/DE)
        // 2. Pts/GW (games_won)
        // 3. GD (games_won - games_lost)
        // 4. MW (matches_won)
        // 5. Head-to-Head
        standings.sort_by(|a, b| {
            b.bracket_score.cmp(&a.bracket_score)
             .then(b.games_won.cmp(&a.games_won))
             .then((b.games_won - b.games_lost).cmp(&(a.games_won - a.games_lost)))
             .then(b.matches_won.cmp(&a.matches_won))
             .then_with(|| {
                 let mut a_wins_h2h = 0;
                 let mut b_wins_h2h = 0;
                 for m in &matches {
                     if m.status == MatchStatus::Completed {
                         let is_h2h = (m.player1_id.as_ref() == Some(&a.id) && m.player2_id.as_ref() == Some(&b.id)) ||
                                      (m.player1_id.as_ref() == Some(&b.id) && m.player2_id.as_ref() == Some(&a.id));
                         if is_h2h {
                             if m.winner_id.as_ref() == Some(&a.id) {
                                 a_wins_h2h += 1;
                             } else if m.winner_id.as_ref() == Some(&b.id) {
                                 b_wins_h2h += 1;
                             }
                         }
                     }
                 }
                 b_wins_h2h.cmp(&a_wins_h2h)
             })
        });

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

pub fn sweep_byes(match_repo: &Arc<dyn MatchRepository>, tournament_id: &str) {
    let mut changes_made = true;
    let mut propagated_byes = std::collections::HashSet::new();

    while changes_made {
        changes_made = false;
        let current_matches = match_repo.get_matches_by_tournament(tournament_id).unwrap_or_default();
        
        for m in current_matches {
            let has_p1 = m.player1_id.is_some() && !m.player1_name.is_empty();
            let has_p2 = m.player2_id.is_some() && !m.player2_name.is_empty();
            
            let p1_bye = m.player1_name == "BYE" || m.player1_id.as_deref() == Some("BYE_ID");
            let p2_bye = m.player2_name == "BYE" || m.player2_id.as_deref() == Some("BYE_ID");
            
            if (m.status == MatchStatus::Pending || m.status == MatchStatus::Bye || m.status == MatchStatus::InProgress) && has_p1 && has_p2 {
                if (p1_bye || p2_bye) && !propagated_byes.contains(&m.id) {
                    
                    let winner_id;
                    let winner_name;
                    
                    if p1_bye && p2_bye {
                        winner_id = Some("BYE_ID".to_string());
                        winner_name = "BYE".to_string();
                    } else if p1_bye {
                        winner_id = m.player2_id.clone();
                        winner_name = m.player2_name.clone();
                    } else {
                        winner_id = m.player1_id.clone();
                        winner_name = m.player1_name.clone();
                    }
                    
                    match_repo.update_match_score(&m.id, 0, 0, &MatchStatus::Bye, winner_id.as_deref()).unwrap_or_default();
                    
                    if let (Some(ref w_id), Some(ref next_id)) = (&winner_id, &m.next_match_id) {
                        match_repo.set_match_player(next_id, m.next_match_slot, w_id, &winner_name).unwrap_or_default();
                    }
                    
                    if let Some(ref loser_next_id) = m.loser_next_match_id {
                        match_repo.set_match_player(loser_next_id, m.loser_next_match_slot, "BYE_ID", "BYE").unwrap_or_default();
                    }
                    
                    propagated_byes.insert(m.id.clone());
                    changes_made = true;
                }
            }
        }
    }
}
