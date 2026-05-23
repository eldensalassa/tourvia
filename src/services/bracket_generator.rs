use std::sync::Arc;
use crate::domain::match_model::{BracketType, Match, MatchStatus};
use crate::domain::participant::Participant;
use crate::domain::round::Round;
use crate::domain::tournament::TournamentType;
use crate::domain::repositories::{MatchRepository, RoundRepository};
use crate::utils;

pub struct BracketGeneratorService {
    match_repo: Arc<dyn MatchRepository>,
    round_repo: Arc<dyn RoundRepository>,
}

impl BracketGeneratorService {
    pub fn new(
        match_repo: Arc<dyn MatchRepository>,
        round_repo: Arc<dyn RoundRepository>,
    ) -> Self {
        Self { match_repo, round_repo }
    }

    pub fn generate_bracket(
        &self,
        tournament_id: &str,
        participants: &[Participant],
        tournament_type: &TournamentType,
    ) -> Result<(Vec<Round>, Vec<Match>), String> {
        match tournament_type {
            TournamentType::RoundRobin => self.generate_round_robin(tournament_id, participants),
            TournamentType::DoubleElimination => self.generate_double_elimination(tournament_id, participants),
            _ => self.generate_elimination(tournament_id, participants, false),
        }
    }

    fn generate_elimination(
        &self,
        tournament_id: &str,
        participants: &[Participant],
        is_double: bool,
    ) -> Result<(Vec<Round>, Vec<Match>), String> {
        let n = participants.len();
        if n < 2 {
            return Err("Need at least 2 participants to generate a bracket.".to_string());
        }

        self.match_repo.delete_all_matches(tournament_id)
            .map_err(|e| format!("Failed to clear matches: {}", e))?;
        self.round_repo.delete_all_rounds(tournament_id)
            .map_err(|e| format!("Failed to clear rounds: {}", e))?;

        let bracket_size = utils::next_power_of_two(n);
        let total_rounds = utils::num_rounds(n);

        let mut rounds = Vec::new();
        for r in 1..=total_rounds {
            let name = utils::round_name(r, total_rounds);
            let round = Round::new(tournament_id.to_string(), r, name, BracketType::Upper);
            self.round_repo.create_round(&round)
                .map_err(|e| format!("Failed to create round: {}", e))?;
            rounds.push(round);
        }

        let mut all_matches: Vec<Vec<Match>> = Vec::new();
        for round_idx in 0..total_rounds as usize {
            let num_matches = bracket_size / 2_usize.pow((round_idx + 1) as u32);
            let round = &rounds[round_idx];
            let mut round_matches = Vec::new();

            for match_order in 0..num_matches {
                let m = Match::new(
                    tournament_id.to_string(),
                    round.id.clone(),
                    match_order as i32,
                    None,
                    0,
                    BracketType::Upper,
                );
                round_matches.push(m);
            }
            all_matches.push(round_matches);
        }

        // Link matches
        for round_idx in 0..(total_rounds as usize - 1) {
            let next_round_matches: Vec<String> = all_matches[round_idx + 1]
                .iter()
                .map(|m| m.id.clone())
                .collect();

            for match_idx in 0..all_matches[round_idx].len() {
                let next_match_idx = match_idx / 2;
                let slot = (match_idx % 2) as i32 + 1;
                all_matches[round_idx][match_idx].next_match_id = Some(next_round_matches[next_match_idx].clone());
                all_matches[round_idx][match_idx].next_match_slot = slot;
            }
        }

        let mut sorted_participants: Vec<&Participant> = participants.iter().collect();
        sorted_participants.sort_by_key(|p| p.seed);
        let seed_order = utils::generate_seed_order(bracket_size);
        let first_round = &mut all_matches[0];
        
        for (slot_idx, &seed_pos) in seed_order.iter().enumerate() {
            let match_idx = slot_idx / 2;
            let player_slot = (slot_idx % 2) + 1;

            if seed_pos <= n {
                let participant = &sorted_participants[seed_pos - 1];
                if player_slot == 1 {
                    first_round[match_idx].player1_id = Some(participant.id.clone());
                    first_round[match_idx].player1_name = participant.name.clone();
                } else {
                    first_round[match_idx].player2_id = Some(participant.id.clone());
                    first_round[match_idx].player2_name = participant.name.clone();
                }
            }
        }

        // Byes
        for match_idx in 0..first_round.len() {
            let has_p1 = first_round[match_idx].player1_id.is_some();
            let has_p2 = first_round[match_idx].player2_id.is_some();

            if has_p1 && has_p2 {
                first_round[match_idx].status = MatchStatus::InProgress;
            } else if has_p1 && !has_p2 {
                first_round[match_idx].status = MatchStatus::Bye;
                first_round[match_idx].winner_id = first_round[match_idx].player1_id.clone();
                first_round[match_idx].player2_id = Some("BYE_ID".to_string());
                first_round[match_idx].player2_name = "BYE".to_string();
            } else if !has_p1 && has_p2 {
                first_round[match_idx].status = MatchStatus::Bye;
                first_round[match_idx].winner_id = first_round[match_idx].player2_id.clone();
                first_round[match_idx].player1_id = Some("BYE_ID".to_string());
                first_round[match_idx].player1_name = "BYE".to_string();
            }
        }

        // Generate lower bracket if Double Elimination
        let mut lower_rounds = Vec::new();
        let mut lower_matches = Vec::new();
        
        if is_double && total_rounds > 1 {
            let num_lower_rounds = 2 * (total_rounds - 1);
            let mut current_matches_count = bracket_size / 4;
            
            for lr_idx in 0..num_lower_rounds {
                let round_num = lr_idx + 1;
                let name = format!("Lower Bracket {}", round_num);
                let round = Round::new(tournament_id.to_string(), round_num as i32, name, BracketType::Lower);
                self.round_repo.create_round(&round).map_err(|e| format!("Failed to create lower round: {}", e))?;
                lower_rounds.push(round.clone());
                
                let mut round_matches = Vec::new();
                for m_idx in 0..current_matches_count {
                    let m = Match::new(
                        tournament_id.to_string(),
                        round.id.clone(),
                        m_idx as i32,
                        None,
                        0,
                        BracketType::Lower,
                    );
                    round_matches.push(m);
                }
                lower_matches.push(round_matches);
                
                if lr_idx % 2 == 1 {
                    current_matches_count /= 2;
                }
            }
            
            // Link Lower Bracket internally
            for lr_idx in 0..num_lower_rounds as usize {
                if lr_idx < (num_lower_rounds - 1) as usize {
                    let next_lr_idx = lr_idx + 1;
                    for m_idx in 0..lower_matches[lr_idx].len() {
                        let next_match_id = lower_matches[next_lr_idx][if lr_idx % 2 == 0 { m_idx } else { m_idx / 2 }].id.clone();
                        let slot = if lr_idx % 2 == 0 {
                            2 // Same number of matches, winner goes to slot 2
                        } else {
                            (m_idx % 2) as i32 + 1 // Halving, winners face each other
                        };
                        
                        lower_matches[lr_idx][m_idx].next_match_id = Some(next_match_id);
                        lower_matches[lr_idx][m_idx].next_match_slot = slot;
                    }
                }
            }
            
            // Link Upper Bracket Losers to Lower Bracket
            // UR1 losers -> LR1
            for m_idx in 0..all_matches[0].len() {
                let lr_match_idx = m_idx / 2;
                let lr_slot = (m_idx % 2) as i32 + 1;
                if lr_match_idx < lower_matches[0].len() {
                    all_matches[0][m_idx].loser_next_match_id = Some(lower_matches[0][lr_match_idx].id.clone());
                    all_matches[0][m_idx].loser_next_match_slot = lr_slot;
                }
            }
            
            // UR(k) losers -> LR(2k-2)
            for ur_idx in 1..total_rounds as usize {
                let lr_idx = 2 * ur_idx - 1; 
                if lr_idx < num_lower_rounds as usize {
                    for m_idx in 0..all_matches[ur_idx].len() {
                        // Cross matching (reverse index)
                        let lr_match_idx = all_matches[ur_idx].len() - 1 - m_idx;
                        if lr_match_idx < lower_matches[lr_idx].len() {
                            all_matches[ur_idx][m_idx].loser_next_match_id = Some(lower_matches[lr_idx][lr_match_idx].id.clone());
                            all_matches[ur_idx][m_idx].loser_next_match_slot = 1;
                        }
                    }
                }
            }
            
            // Generate Grand Final
            let gf_round = Round::new(tournament_id.to_string(), 1, "Grand Final".to_string(), BracketType::GrandFinal);
            self.round_repo.create_round(&gf_round).map_err(|e| format!("Failed to create GF round: {}", e))?;
            let mut gf_match = Match::new(
                tournament_id.to_string(),
                gf_round.id.clone(),
                0,
                None,
                0,
                BracketType::GrandFinal,
            );
            
            let ur_last_idx = total_rounds as usize - 1;
            all_matches[ur_last_idx][0].next_match_id = Some(gf_match.id.clone());
            all_matches[ur_last_idx][0].next_match_slot = 1;
            
            if num_lower_rounds > 0 {
                let lr_last_idx = num_lower_rounds as usize - 1;
                lower_matches[lr_last_idx][0].next_match_id = Some(gf_match.id.clone());
                lower_matches[lr_last_idx][0].next_match_slot = 2;
            }
            
            lower_matches.push(vec![gf_match]);
            lower_rounds.push(gf_round);
        }

        let mut flat_matches = Vec::new();
        for round_matches in &all_matches {
            for m in round_matches {
                self.match_repo.create_match(m)
                    .map_err(|e| format!("Failed to create match: {}", e))?;
                flat_matches.push(m.clone());
            }
        }
        for round_matches in &lower_matches {
            for m in round_matches {
                self.match_repo.create_match(m)
                    .map_err(|e| format!("Failed to create match: {}", e))?;
                flat_matches.push(m.clone());
            }
        }
        rounds.extend(lower_rounds);

        crate::services::match_service::sweep_byes(&self.match_repo, tournament_id);

        Ok((rounds, flat_matches))
    }

    fn generate_double_elimination(
        &self,
        tournament_id: &str,
        participants: &[Participant],
    ) -> Result<(Vec<Round>, Vec<Match>), String> {
        self.generate_elimination(tournament_id, participants, true)
    }

    fn generate_round_robin(
        &self,
        tournament_id: &str,
        participants: &[Participant],
    ) -> Result<(Vec<Round>, Vec<Match>), String> {
        let n = participants.len();
        if n < 2 {
            return Err("Need at least 2 participants for Round Robin.".to_string());
        }

        self.match_repo.delete_all_matches(tournament_id)
            .map_err(|e| format!("Failed to clear matches: {}", e))?;
        self.round_repo.delete_all_rounds(tournament_id)
            .map_err(|e| format!("Failed to clear rounds: {}", e))?;

        let mut sorted: Vec<&Participant> = participants.iter().collect();
        sorted.sort_by_key(|p| p.seed);

        let has_bye = n % 2 != 0;
        let effective_n = if has_bye { n + 1 } else { n };
        let num_rounds = effective_n - 1;
        let matches_per_round = effective_n / 2;

        let mut all_rounds = Vec::new();
        let mut all_matches = Vec::new();

        for r in 0..num_rounds {
            let round = Round::new(
                tournament_id.to_string(),
                (r + 1) as i32,
                format!("Round {}", r + 1),
                BracketType::Upper,
            );
            self.round_repo.create_round(&round)
                .map_err(|e| format!("Failed to create round: {}", e))?;

            let mut order: Vec<usize> = Vec::with_capacity(effective_n);
            order.push(0);
            for i in 1..effective_n {
                let rotated = 1 + ((i - 1 + r) % (effective_n - 1));
                order.push(rotated);
            }

            for m_idx in 0..matches_per_round {
                let idx_a = order[m_idx];
                let idx_b = order[effective_n - 1 - m_idx];

                let mut match_ = Match::new(
                    tournament_id.to_string(),
                    round.id.clone(),
                    m_idx as i32,
                    None,
                    0,
                    BracketType::Upper,
                );

                let a_is_bye = idx_a >= n;
                let b_is_bye = idx_b >= n;

                if !a_is_bye {
                    match_.player1_id = Some(sorted[idx_a].id.clone());
                    match_.player1_name = sorted[idx_a].name.clone();
                }
                if !b_is_bye {
                    match_.player2_id = Some(sorted[idx_b].id.clone());
                    match_.player2_name = sorted[idx_b].name.clone();
                }

                if a_is_bye || b_is_bye {
                    match_.status = MatchStatus::Bye;
                    if a_is_bye && !b_is_bye {
                        match_.player1_name = "BYE".to_string();
                        match_.winner_id = match_.player2_id.clone();
                    } else if b_is_bye && !a_is_bye {
                        match_.player2_name = "BYE".to_string();
                        match_.winner_id = match_.player1_id.clone();
                    }
                } else {
                    match_.status = MatchStatus::InProgress;
                }

                self.match_repo.create_match(&match_)
                    .map_err(|e| format!("Failed to create match: {}", e))?;
                all_matches.push(match_);
            }

            all_rounds.push(round);
        }

        Ok((all_rounds, all_matches))
    }
}
