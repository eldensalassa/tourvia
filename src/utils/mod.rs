/// Generate round name based on round number and total rounds.
///
/// # Examples
/// - 4 rounds total: Round 1 → "Round of 16", Round 2 → "Quarter Final", Round 3 → "Semi Final", Round 4 → "Final"
/// - 3 rounds total: Round 1 → "Quarter Final", Round 2 → "Semi Final", Round 3 → "Final"
pub fn round_name(round_number: i32, total_rounds: i32) -> String {
    let rounds_from_end = total_rounds - round_number;
    match rounds_from_end {
        0 => "Final".to_string(),
        1 => "Semi Final".to_string(),
        2 => "Quarter Final".to_string(),
        n => format!("Round of {}", 2_i32.pow((n + 1) as u32)),
    }
}

/// Calculate the next power of two >= n.
pub fn next_power_of_two(n: usize) -> usize {
    if n == 0 {
        return 1;
    }
    let mut v = n - 1;
    v |= v >> 1;
    v |= v >> 2;
    v |= v >> 4;
    v |= v >> 8;
    v |= v >> 16;
    v + 1
}

/// Calculate the number of rounds needed for n participants.
pub fn num_rounds(n: usize) -> i32 {
    if n <= 1 {
        return 0;
    }
    let bracket_size = next_power_of_two(n);
    (bracket_size as f64).log2() as i32
}

/// Generate standard tournament seeding order for a given bracket size.
///
/// For bracket_size=8, returns [1, 8, 5, 4, 3, 6, 7, 2]
/// This ensures seed 1 plays seed N, seed 2 plays seed N-1, etc.,
/// and top seeds are on opposite sides of the bracket.
pub fn generate_seed_order(bracket_size: usize) -> Vec<usize> {
    if bracket_size == 1 {
        return vec![1];
    }
    if bracket_size == 2 {
        return vec![1, 2];
    }

    // Recursive approach: split bracket in half
    let half = bracket_size / 2;
    let prev = generate_seed_order(half);

    let mut result = Vec::with_capacity(bracket_size);
    for &seed in &prev {
        result.push(seed);
        result.push(bracket_size + 1 - seed);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next_power_of_two() {
        assert_eq!(next_power_of_two(1), 1);
        assert_eq!(next_power_of_two(2), 2);
        assert_eq!(next_power_of_two(3), 4);
        assert_eq!(next_power_of_two(5), 8);
        assert_eq!(next_power_of_two(7), 8);
        assert_eq!(next_power_of_two(8), 8);
        assert_eq!(next_power_of_two(9), 16);
        assert_eq!(next_power_of_two(16), 16);
    }

    #[test]
    fn test_num_rounds() {
        assert_eq!(num_rounds(2), 1);
        assert_eq!(num_rounds(4), 2);
        assert_eq!(num_rounds(8), 3);
        assert_eq!(num_rounds(16), 4);
        assert_eq!(num_rounds(3), 2);  // rounds up to 4 → 2 rounds
        assert_eq!(num_rounds(5), 3);  // rounds up to 8 → 3 rounds
    }

    #[test]
    fn test_round_name() {
        // 4-round tournament
        assert_eq!(round_name(1, 4), "Round of 16");
        assert_eq!(round_name(2, 4), "Quarter Final");
        assert_eq!(round_name(3, 4), "Semi Final");
        assert_eq!(round_name(4, 4), "Final");

        // 3-round tournament
        assert_eq!(round_name(1, 3), "Quarter Final");
        assert_eq!(round_name(2, 3), "Semi Final");
        assert_eq!(round_name(3, 3), "Final");
    }

    #[test]
    fn test_seed_order_4() {
        let order = generate_seed_order(4);
        assert_eq!(order, vec![1, 4, 2, 3]);
    }

    #[test]
    fn test_seed_order_8() {
        let order = generate_seed_order(8);
        assert_eq!(order, vec![1, 8, 4, 5, 2, 7, 3, 6]);
    }

    #[test]
    fn test_seed_order_preserves_top_seeds() {
        let order = generate_seed_order(16);
        // Seed 1 and seed 2 should be in opposite halves
        let pos_1 = order.iter().position(|&s| s == 1).unwrap();
        let pos_2 = order.iter().position(|&s| s == 2).unwrap();
        let half = order.len() / 2;
        assert!((pos_1 < half && pos_2 >= half) || (pos_2 < half && pos_1 >= half));
    }
}
