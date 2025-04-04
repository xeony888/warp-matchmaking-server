use crate::Match;

pub const VALID_GAME_TYPES: [&str; 2] = ["soccer", "knockout"];
pub const VALID_PRIZE_AMOUNTS: [u32; 5] = [2, 5, 10, 25, 50];
pub fn validate_game_type(game: &String) -> bool {
    return VALID_GAME_TYPES.contains(&game.as_str());
}

pub fn validate_prize_amount(amount: &u32) -> bool {
    return VALID_PRIZE_AMOUNTS.contains(amount);
}
pub fn validate_can_join_match(m: &Match) -> bool {
    println!("Amount: {}", match_type_to_min_players(&m.game_type));
    return m.players.len() < match_type_to_max_players(&m.game_type);
}
fn match_type_to_max_players(match_type: &String) -> usize {
    match match_type.as_str() {
        "soccer" => 2,
        "knockout" => 2,
        _ => 0,
    }
}
fn match_type_to_min_players(match_type: &String) -> usize {
    match match_type.as_str() {
        "soccer" => 2,
        "knockout" => 2,
        _ => 0,
    }
}
pub fn validate_user_in_game(username: &String, m: &Match) -> bool {
    return m.players.contains(username);
}
pub fn validate_game_not_started(m: &Match) -> bool {
    return m.players.len() != match_type_to_min_players(&m.game_type);
}
