pub fn get_max_players_for_game(game_type: &String) -> usize {
    match game_type.as_str() {
        "soccer" => 2,
        "knockout" => 2,
        _ => 0,
    }
}
