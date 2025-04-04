use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct MatchRequest {
    pub prize: u32,
    pub game_type: String,
}

#[derive(Deserialize)]
pub struct JoinQuery {
    pub id: u64,
}
