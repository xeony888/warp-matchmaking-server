use dotenvy::dotenv;
use error::{CannotJoinMatchError, InvalidInputError, UnauthorizedError};
use request::{JoinQuery, MatchRequest};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::env;
use std::net::Ipv4Addr;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use user::{with_user, User};
use validation::{validate_can_join_match, validate_game_not_started, validate_game_type, validate_prize_amount, validate_user_in_game};
use warp::{http::StatusCode, reject::Rejection, reply::Reply, Filter};
pub mod error;
pub mod request;
pub mod user;
pub mod validation;
use crate::error::NotFoundError;
// must add game local url here
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Match {
    id: u64,
    players: Vec<String>,
    prize: u32,
    game_type: String,
    expiry_time: u64,
}
type Matches = Arc<RwLock<HashMap<u64, Arc<RwLock<Match>>>>>;
const GAME_EXPIRY_TIME_SECS: u64 = 60 * 20;
async fn get_matches_handler(matches: Matches, user: User) -> Result<impl Reply, Rejection> {
    let matches_read = matches.read().await;
    let mut matches_list = Vec::new();

    for game in matches_read.values() {
        let game_read = game.read().await;
        matches_list.push(game_read.clone()); // Clone to avoid holding the lock
    }
    Ok(warp::reply::json(&matches_list))
}
async fn get_match_handler(matches: Matches, query: JoinQuery, user: User) -> Result<impl Reply, Rejection> {
    let matches_read = matches.read().await;
    if let Some(game) = matches_read.get(&query.id) {
        let game_read = game.read().await;
        if !validate_user_in_game(&user.username, &game_read) {
            return Err(warp::reject::custom(NotFoundError)); // need to return NotFoundError to not let user know whether or not game exists
        }
        Ok(warp::reply::json(&*game_read))
    } else {
        Err(warp::reject::custom(NotFoundError))
    }
}
async fn create_match_handler(matches: Matches, new_match: MatchRequest, user: User) -> Result<impl Reply, Rejection> {
    let mut matches_write = matches.write().await;
    let id = matches_write.len() as u64;
    // validate game type and prize
    if !validate_game_type(&new_match.game_type) || !validate_prize_amount(&new_match.prize) {
        return Err(warp::reject::custom(InvalidInputError));
    }
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let new_match = Match {
        id,
        players: vec![user.username],
        prize: new_match.prize,
        game_type: new_match.game_type,
        expiry_time: now + GAME_EXPIRY_TIME_SECS,
    };
    matches_write.insert(id, Arc::new(RwLock::new(new_match.clone())));
    Ok(warp::reply::json(&new_match))
}
async fn join_match_handler(matches: Matches, query: JoinQuery, user: User) -> Result<impl Reply, Rejection> {
    let matches_read = matches.read().await;
    if let Some(found) = matches_read.get(&query.id) {
        let mut match_write = found.write().await;
        if !validate_can_join_match(&match_write) {
            return Err(warp::reject::custom(CannotJoinMatchError));
        }
        match_write.players.push(user.username.clone());
        Ok(warp::reply::json(&*match_write))
    } else {
        Err(warp::reject::custom(NotFoundError))
    }
}
async fn cancel_match_handler(matches: Matches, query: JoinQuery, user: User) -> Result<impl Reply, Rejection> {
    let mut matches_write = matches.write().await;
    // validate user in game and game not started
    if let Some(match_data) = matches_write.get(&query.id) {
        let match_data_read = match_data.read().await;
        if !validate_user_in_game(&user.username, &match_data_read) || !validate_game_not_started(&match_data_read) {
            // fix this, for multiplayer games shouldnt remove game unless players is 0
        } else {
            return Err(warp::reject::custom(InvalidInputError));
        }
    } else {
        return Err(warp::reject::custom(NotFoundError));
    };
    matches_write.remove(&query.id);
    return Ok(warp::reply::with_status("", StatusCode::OK));
}

async fn end_match_handler() -> Result<impl Reply, Rejection> {
    Ok(warp::reply())
}
async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    if err.is_not_found() {
        println!("Not found");
        Ok(warp::reply::with_status("Not Found", StatusCode::NOT_FOUND))
    } else if let Some(_) = err.find::<NotFoundError>() {
        println!("Not found error");
        Ok(warp::reply::with_status("Match not found", StatusCode::NOT_FOUND))
    } else if let Some(_) = err.find::<UnauthorizedError>() {
        println!("Unauthorized");
        Ok(warp::reply::with_status("Unauthorized", StatusCode::UNAUTHORIZED))
    } else if let Some(_) = err.find::<InvalidInputError>() {
        println!("Invalid input");
        Ok(warp::reply::with_status("Invalid input", StatusCode::BAD_REQUEST))
    } else if let Some(_) = err.find::<CannotJoinMatchError>() {
        println!("Cannot join match");
        Ok(warp::reply::with_status("Cannot join selected match", StatusCode::FORBIDDEN))
    } else {
        println!("Other error: {:?}", err);
        Ok(warp::reply::with_status("Internal Server Error", StatusCode::INTERNAL_SERVER_ERROR))
    }
}
#[tokio::main]
async fn main() {
    dotenv().ok();
    let matches: Matches = Arc::new(RwLock::new(HashMap::new()));
    fn with_matches(matches: Matches) -> impl Filter<Extract = (Matches,), Error = std::convert::Infallible> + Clone {
        warp::any().map(move || matches.clone()) // .clone() just cloned the ref because it is an Arc
    }
    let matches_route = warp::path!("matches")
        .and(warp::get())
        .and(with_matches(matches.clone()))
        .and(with_user())
        .and_then(get_matches_handler);
    let match_route = warp::path("match")
        .and(warp::get())
        .and(with_matches(matches.clone()))
        .and(warp::query::<JoinQuery>())
        .and(with_user())
        .and_then(get_match_handler);
    let create_match_route = warp::path("create")
        .and(warp::post())
        .and(with_matches(matches.clone()))
        .and(warp::body::json())
        .and(with_user())
        .and_then(create_match_handler);
    let join_match_route = warp::path("join")
        .and(warp::post())
        .and(with_matches(matches.clone()))
        .and(warp::query::<JoinQuery>()) // Use struct instead of raw u64
        .and(with_user())
        .and_then(join_match_handler);
    let cancel_match_route = warp::path("cancel")
        .and(warp::post())
        .and(with_matches(matches.clone()))
        .and(warp::query::<JoinQuery>())
        .and(with_user())
        .and_then(cancel_match_handler);
    let end_match_route = warp::path("end_match").and(warp::post()).and_then(end_match_handler);
    let routes = matches_route
        .or(match_route)
        .or(create_match_route)
        .or(join_match_route)
        .or(cancel_match_route)
        .or(end_match_route)
        .recover(handle_rejection);

    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());

    // Parse host and port
    let host: Ipv4Addr = host.parse().expect("Invalid HOST address");
    let port: u16 = port.parse().expect("Invalid PORT number");

    println!("Starting server at {}:{}", host, port);

    warp::serve(routes).run((host, port)).await;
}
