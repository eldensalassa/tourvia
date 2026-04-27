use crate::database::Database;
use crate::domain::match_model::{Match, MatchStatus};
use crate::domain::participant::Participant;
use crate::domain::round::Round;
use crate::domain::tournament::TournamentType;
use crate::utils;

/// Generate the full bracket structure for a tournament.
pub fn generate_bracket(
    db: &Database,
    tournament_id: &str,
    participants: &[Participant],
    tournament_type: &TournamentType,
) -> Result<(Vec<Round>, Vec<Match>), String> {
    match tournament_type {
        TournamentType::RoundRobin => generate_round_robin(db, tournament_id, participants),
        _ => generate_elimination(db, tournament_id, participants),
    }
}

/// Generate a single/double elimination bracket.
fn generate_elimination(
    db: &Database,
    tournament_id: &str,
    participants: &[Participant],
) -> Result<(Vec<Round>, Vec<Match>), String> {
    let n = participants.len();
    if n < 2 {
        return Err("Need at least 2 participants to generate a bracket.".to_string());
    }

    // Clean up any existing bracket data
    db.delete_all_matches(tournament_id)
        .map_err(|e| format!("Failed to clear matches: {}", e))?;
    db.delete_all_rounds(tournament_id)
        .map_err(|e| format!("Failed to clear rounds: {}", e))?;

    let bracket_size = utils::next_power_of_two(n);
    let total_rounds = utils::num_rounds(n);

    // Create rounds
    let mut rounds = Vec::new();
    for r in 1..=total_rounds {
        let name = utils::round_name(r, total_rounds);
        let round = Round::new(tournament_id.to_string(), r, name);
        db.create_round(&round)
            .map_err(|e| format!("Failed to create round: {}", e))?;
        rounds.push(round);
    }

    // Generate all matches round by round (forward)
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
            );
            round_matches.push(m);
        }
        all_matches.push(round_matches);
    }

    // Link matches: each match in round R feeds into match R+1
    for round_idx in 0..(total_rounds as usize - 1) {
        let next_round_matches: Vec<String> = all_matches[round_idx + 1]
            .iter()
            .map(|m| m.id.clone())
            .collect();

        for match_idx in 0..all_matches[round_idx].len() {
            let next_match_idx = match_idx / 2;
            let slot = (match_idx % 2) as i32 + 1;
            all_matches[round_idx][match_idx].next_match_id =
                Some(next_round_matches[next_match_idx].clone());
            all_matches[round_idx][match_idx].next_match_slot = slot;
        }
    }

    // Sort participants by seed
    let mut sorted_participants: Vec<&Participant> = participants.iter().collect();
    sorted_participants.sort_by_key(|p| p.seed);

    // Generate seeding order
    let seed_order = utils::generate_seed_order(bracket_size);

    // Assign participants to first-round matches
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

    // Process byes
    for match_idx in 0..first_round.len() {
        let has_p1 = first_round[match_idx].player1_id.is_some();
        let has_p2 = first_round[match_idx].player2_id.is_some();

        if has_p1 && has_p2 {
            first_round[match_idx].status = MatchStatus::InProgress;
        } else if has_p1 && !has_p2 {
            first_round[match_idx].status = MatchStatus::Bye;
            first_round[match_idx].winner_id = first_round[match_idx].player1_id.clone();
            first_round[match_idx].player2_name = "BYE".to_string();
        } else if !has_p1 && has_p2 {
            first_round[match_idx].status = MatchStatus::Bye;
            first_round[match_idx].winner_id = first_round[match_idx].player2_id.clone();
            first_round[match_idx].player1_name = "BYE".to_string();
        }
    }

    // Save all matches to database
    let mut flat_matches = Vec::new();
    for round_matches in &all_matches {
        for m in round_matches {
            db.create_match(m)
                .map_err(|e| format!("Failed to create match: {}", e))?;
            flat_matches.push(m.clone());
        }
    }

    // Advance BYE winners to the next round
    for round_matches in &all_matches[0..1] {
        for m in round_matches {
            if m.status == MatchStatus::Bye {
                if let (Some(ref winner_id), Some(ref next_id)) = (&m.winner_id, &m.next_match_id) {
                    let winner_name = if m.player1_id.as_ref() == Some(winner_id) {
                        &m.player1_name
                    } else {
                        &m.player2_name
                    };
                    db.set_match_player(next_id, m.next_match_slot, winner_id, winner_name)
                        .map_err(|e| format!("Failed to advance bye winner: {}", e))?;
                }
            }
        }
    }

    Ok((rounds, flat_matches))
}

/// Generate a round-robin schedule where every participant plays every other participant.
fn generate_round_robin(
    db: &Database,
    tournament_id: &str,
    participants: &[Participant],
) -> Result<(Vec<Round>, Vec<Match>), String> {
    let n = participants.len();
    if n < 2 {
        return Err("Need at least 2 participants for Round Robin.".to_string());
    }

    // Clean up
    db.delete_all_matches(tournament_id)
        .map_err(|e| format!("Failed to clear matches: {}", e))?;
    db.delete_all_rounds(tournament_id)
        .map_err(|e| format!("Failed to clear rounds: {}", e))?;

    // Sort by seed
    let mut sorted: Vec<&Participant> = participants.iter().collect();
    sorted.sort_by_key(|p| p.seed);

    let has_bye = n % 2 != 0;
    let effective_n = if has_bye { n + 1 } else { n };
    let num_rounds = effective_n - 1;
    let matches_per_round = effective_n / 2;

    // Build participant indices (index effective_n-1 is the BYE slot if odd)
    let mut all_rounds = Vec::new();
    let mut all_matches = Vec::new();

    for r in 0..num_rounds {
        let round = Round::new(
            tournament_id.to_string(),
            (r + 1) as i32,
            format!("Round {}", r + 1),
        );
        db.create_round(&round)
            .map_err(|e| format!("Failed to create round: {}", e))?;

        // Circle method: fix index 0, rotate indices 1..effective_n-1
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

            db.create_match(&match_)
                .map_err(|e| format!("Failed to create match: {}", e))?;
            all_matches.push(match_);
        }

        all_rounds.push(round);
    }

    Ok((all_rounds, all_matches))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_bracket_4_players() {
        let db = Database::open(":memory:").unwrap();
        let tournament_id = "test-t1";

        let t = crate::domain::tournament::Tournament {
            id: tournament_id.to_string(),
            name: "Test".to_string(),
            tournament_type: TournamentType::SingleElimination,
            participant_count: 4,
            status: crate::domain::tournament::TournamentStatus::Draft,
            created_at: "2025-01-01".to_string(),
            description: String::new(),
            game_name: String::new(),
        };
        db.create_tournament(&t).unwrap();

        let participants: Vec<Participant> = (1..=4)
            .map(|i| Participant::new(tournament_id.to_string(), format!("Player {}", i), i))
            .collect();

        for p in &participants {
            db.create_participant(p).unwrap();
        }

        let (rounds, matches) = generate_bracket(&db, tournament_id, &participants, &TournamentType::SingleElimination).unwrap();

        assert_eq!(rounds.len(), 2);
        assert_eq!(matches.len(), 3);
    }

    #[test]
    fn test_generate_bracket_with_byes() {
        let db = Database::open(":memory:").unwrap();
        let tournament_id = "test-t2";

        let t = crate::domain::tournament::Tournament {
            id: tournament_id.to_string(),
            name: "Test Bye".to_string(),
            tournament_type: TournamentType::SingleElimination,
            participant_count: 3,
            status: crate::domain::tournament::TournamentStatus::Draft,
            created_at: "2025-01-01".to_string(),
            description: String::new(),
            game_name: String::new(),
        };
        db.create_tournament(&t).unwrap();

        let participants: Vec<Participant> = (1..=3)
            .map(|i| Participant::new(tournament_id.to_string(), format!("Player {}", i), i))
            .collect();

        for p in &participants {
            db.create_participant(p).unwrap();
        }

        let (rounds, matches) = generate_bracket(&db, tournament_id, &participants, &TournamentType::SingleElimination).unwrap();

        assert_eq!(rounds.len(), 2);
        let bye_count = matches.iter().filter(|m| m.status == MatchStatus::Bye).count();
        assert!(bye_count >= 1, "Expected at least 1 bye match, got {}", bye_count);
    }

    #[test]
    fn test_generate_bracket_2_players() {
        let db = Database::open(":memory:").unwrap();
        let tournament_id = "test-t3";

        let t = crate::domain::tournament::Tournament {
            id: tournament_id.to_string(),
            name: "Test 2".to_string(),
            tournament_type: TournamentType::SingleElimination,
            participant_count: 2,
            status: crate::domain::tournament::TournamentStatus::Draft,
            created_at: "2025-01-01".to_string(),
            description: String::new(),
            game_name: String::new(),
        };
        db.create_tournament(&t).unwrap();

        let participants: Vec<Participant> = (1..=2)
            .map(|i| Participant::new(tournament_id.to_string(), format!("Player {}", i), i))
            .collect();

        for p in &participants {
            db.create_participant(p).unwrap();
        }

        let (rounds, matches) = generate_bracket(&db, tournament_id, &participants, &TournamentType::SingleElimination).unwrap();

        assert_eq!(rounds.len(), 1);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].status, MatchStatus::InProgress);
    }

    #[test]
    fn test_round_robin_4_players() {
        let db = Database::open(":memory:").unwrap();
        let tournament_id = "test-rr";

        let t = crate::domain::tournament::Tournament {
            id: tournament_id.to_string(),
            name: "RR Test".to_string(),
            tournament_type: TournamentType::RoundRobin,
            participant_count: 4,
            status: crate::domain::tournament::TournamentStatus::Draft,
            created_at: "2025-01-01".to_string(),
            description: String::new(),
            game_name: String::new(),
        };
        db.create_tournament(&t).unwrap();

        let participants: Vec<Participant> = (1..=4)
            .map(|i| Participant::new(tournament_id.to_string(), format!("Player {}", i), i))
            .collect();

        for p in &participants {
            db.create_participant(p).unwrap();
        }

        let (rounds, matches) = generate_bracket(&db, tournament_id, &participants, &TournamentType::RoundRobin).unwrap();

        // 4 players, even → 3 rounds, 2 matches per round = 6 total matches
        assert_eq!(rounds.len(), 3);
        assert_eq!(matches.len(), 6);
        // All should be InProgress (no byes with even count)
        assert!(matches.iter().all(|m| m.status == MatchStatus::InProgress));
    }
}
