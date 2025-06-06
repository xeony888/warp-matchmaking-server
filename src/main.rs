use async_stream::stream;
use dotenvy::dotenv;
use error::{CannotBroadcastError, CannotJoinMatchError, IdGenerationError, InvalidInputError, NoAvailablePorts, UnauthorizedError};
use futures::lock::Mutex;
use getrandom;
use info::get_max_players_for_game;
use request::{JoinQuery, MatchRequest};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::env;
use std::net::Ipv4Addr;
use std::ops::Index;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{collections::HashMap, sync::Arc};
use tokio::process::Command;
use tokio::sync::{watch, RwLock};
use user::{with_user, User};
use utils::{game_type_to_path, NumberPool, SharedNumberPool};
use validation::{validate_can_join_match, validate_game_not_started, validate_game_type, validate_prize_amount, validate_user_in_game};
use warp::filters::sse;
use warp::{http::StatusCode, reject::Rejection, reply::Reply, Filter};
pub mod error;
pub mod info;
pub mod request;
pub mod user;
pub mod utils;
pub mod validation;
use crate::error::NotFoundError;

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum MatchState {
    OPEN,
    READYING,
    PLAYING,
}
// must add game local url here
#[derive(Serialize, Debug, Clone)]
pub struct Match {
    pub id: u32,
    pub players: Vec<String>,
    #[serde(skip)]
    pub player_tokens: Vec<String>,
    pub ready: Vec<bool>,
    pub prize: u32,
    pub game_type: String,
    pub expiry_time: u64,
    pub port: u32,
    pub state: MatchState,
    #[serde(skip)]
    pub state_channel: watch::Sender<(MatchState, Vec<bool>, Vec<String>, u32)>,
}
type Matches = Arc<RwLock<HashMap<u32, Arc<RwLock<Match>>>>>;

const GAME_EXPIRY_TIME_SECS: u64 = 60 * 20;

async fn health_handler() -> Result<impl Reply, Rejection> {
    Ok(warp::reply::with_status("Health check successful", StatusCode::OK))
}
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
async fn create_match_handler(matches: Matches, port_pool: SharedNumberPool, new_match: MatchRequest, user: User) -> Result<impl Reply, Rejection> {
    let mut matches_write = matches.write().await;
    let mut id: u32;
    loop {
        let mut buffer = [0u8; 4];

        getrandom::fill(&mut buffer).map_err(|_| warp::reject::custom(IdGenerationError))?;

        id = u32::from_ne_bytes(buffer);

        if !matches_write.contains_key(&id) {
            break;
        }
    }
    // validate game type and prize
    if !validate_game_type(&new_match.game_type) || !validate_prize_amount(&new_match.prize) {
        return Err(warp::reject::custom(InvalidInputError));
    }
    let port = match port_pool.lock().await.get() {
        Some(p) => p,
        None => return Err(warp::reject::custom(NoAvailablePorts)),
    };
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let (state_tx, _) = watch::channel((MatchState::OPEN, vec![false], vec![user.username.clone()], port));
    let new_match = Match {
        id,
        players: vec![user.username],
        player_tokens: vec![user.auth_token],
        ready: vec![false],
        prize: new_match.prize,
        game_type: new_match.game_type,
        expiry_time: now + GAME_EXPIRY_TIME_SECS,
        state_channel: state_tx,
        port,
        state: MatchState::OPEN,
    };
    println!("Inserting with id: {}", id);
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
        match_write.ready.push(false);
        match_write.player_tokens.push(user.auth_token.clone());
        let max_players = get_max_players_for_game(&match_write.game_type);
        if match_write.players.len() == max_players {
            match_write.state = MatchState::READYING;
        }
        match_write
            .state_channel
            .send((
                match_write.state.clone(),
                match_write.ready.clone(),
                match_write.players.clone(),
                match_write.port,
            ))
            .map_err(|err| {
                println!("{:?}", err);
                warp::reject::custom(CannotBroadcastError)
            })?;
        Ok(warp::reply::json(&*match_write))
    } else {
        println!("Not found in id {}", query.id);
        Err(warp::reject::custom(NotFoundError))
    }
}
async fn cancel_match_handler(matches: Matches, port_pool: SharedNumberPool, query: JoinQuery, user: User) -> Result<impl Reply, Rejection> {
    let mut matches_write = matches.write().await;
    let port: u32;
    let mut remove: bool = false;
    if let Some(match_data) = matches_write.get(&query.id) {
        let mut match_data_write = match_data.write().await;
        if !validate_user_in_game(&user.username, &match_data_write) || !validate_game_not_started(&match_data_write) {
            return Err(warp::reject::custom(InvalidInputError));
        } else {
            let index = match_data_write.players.iter().position(|t| *t == user.username).unwrap();
            match_data_write.players.remove(index);
            match_data_write.player_tokens.remove(index);
            match_data_write.ready.remove(index);
            port = match_data_write.port;
            if match_data_write.players.len() == 0 {
                remove = true;
            }
        }
    } else {
        return Err(warp::reject::custom(NotFoundError));
    };
    if remove {
        matches_write.remove(&query.id);
        port_pool.lock().await.release(port);
    }
    return Ok(warp::reply::with_status("", StatusCode::OK));
}

async fn end_match_handler() -> Result<impl Reply, Rejection> {
    Ok(warp::reply())
}
async fn ready_handler(matches: Matches, query: JoinQuery, user: User) -> Result<impl Reply, Rejection> {
    let matches = matches.read().await;
    let match_arc = matches.get(&query.id).ok_or_else(|| warp::reject::custom(NotFoundError))?;
    let mut game = match_arc.write().await;

    let idx = game
        .players
        .iter()
        .position(|u| u == &user.username)
        .ok_or_else(|| warp::reject::custom(InvalidInputError))?;

    game.ready[idx] = true;

    let all_ready = game.ready.iter().all(|&r| r);
    if all_ready {
        game.state = MatchState::PLAYING;
    }
    game.state_channel
        .send((game.state.clone(), game.ready.clone(), game.players.clone(), game.port))
        .map_err(|_| warp::reject::custom(CannotBroadcastError))?;
    if all_ready {
        let game_type = game.game_type.clone();
        let port = game.port.clone();
        let players = game.players.clone();
        let player_tokens = game.player_tokens.clone();
        // check that this does not block and the mutexes claimed earlier are released
        tokio::spawn(async move {
            let result = run_game_process(&game_type, port, &players[0], &player_tokens[0], &players[1], &player_tokens[1]).await;
            match result {
                Ok(exit_code) => {
                    println!("Game process exited with code: {}", exit_code);
                    if exit_code == 1001 {
                        // player 1 won
                    } else if exit_code == 1002 {
                        // player 2 won
                    } else {
                        // error occurred, add back balance to both players
                    }
                }
                Err(e) => {
                    // error occurred, add back balance to both players
                    println!("Failed to run game process: {:?}", e);
                }
            }
        });
    }
    return Ok(warp::reply::with_status("reply", StatusCode::OK));
}
async fn match_ready_updates(matches: Matches, query: JoinQuery) -> Result<impl Reply, Rejection> {
    let matches_read = matches.read().await;
    let match_arc = matches_read.get(&query.id).ok_or_else(|| warp::reject::custom(NotFoundError))?;
    let match_read = match_arc.read().await;
    let mut rx = match_read.state_channel.subscribe();
    drop(match_read);
    drop(matches_read);
    let stream = stream! {
        loop {
            match rx.changed().await {
                Ok(()) => {
                    let state = rx.borrow().clone();
                    yield Ok::<warp::sse::Event, warp::Error>(
                        warp::sse::Event::default().json_data(state).unwrap()
                    )
                },
                Err(_) => break
            }
        }
    };
    Ok(sse::reply(stream))
}
async fn run_game_process(
    game_type: &String,
    port: u32,
    player_1: &String,
    player_1_token: &String,
    player_2: &String,
    player_2_token: &String,
) -> Result<i32, std::io::Error> {
    // Example: Running a command as a process
    let path = game_type_to_path(game_type);
    let formatted = match env::var("ENVIRONMENT").ok() {
        Some(_) => String::from("./target/release/game-simulation"),
        None => format!("./builds/{}", path),
    };
    println!(
        "Starting {} game at port {} for players {}, {} with tokens {}, {}",
        game_type, port, player_1, player_2, player_1_token, player_2_token
    );
    let mut child = Command::new(formatted)
        .arg("-port")
        .arg(port.to_string())
        .arg("-username1")
        .arg(player_1)
        .arg("-player1token")
        .arg(player_1_token)
        .arg("-username2")
        .arg(player_2)
        .arg("-player2token")
        .arg(player_2_token)
        .spawn()?;
    let exit_status = child.wait().await?;
    Ok(exit_status.code().unwrap_or(1000)) // Return the exit code, or -1 on error
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
    } else if let Some(_) = err.find::<CannotBroadcastError>() {
        println!("Broadcasting failed");
        Ok(warp::reply::with_status("Broadcasting failed", StatusCode::INTERNAL_SERVER_ERROR))
    } else if let Some(_) = err.find::<IdGenerationError>() {
        println!("Id generation failed");
        Ok(warp::reply::with_status("ID generation failed", StatusCode::INTERNAL_SERVER_ERROR))
    } else if let Some(_) = err.find::<NoAvailablePorts>() {
        println!("No available ports");
        Ok(warp::reply::with_status("No available ports", StatusCode::INTERNAL_SERVER_ERROR))
    } else {
        println!("Other error: {:?}", err);
        Ok(warp::reply::with_status("Internal Server Error", StatusCode::INTERNAL_SERVER_ERROR))
    }
}
#[tokio::main]
async fn main() {
    dotenv().ok();
    let matches: Matches = Arc::new(RwLock::new(HashMap::new()));
    let low_port: u32 = env::var("PORT_START")
        .unwrap_or_else(|_| "30000".to_string())
        .parse()
        .expect("Invalid low port");
    let high_port: u32 = env::var("PORT_END")
        .unwrap_or_else(|_| "31000".to_string())
        .parse()
        .expect("Invalid high port");
    let port_pool: SharedNumberPool = Arc::new(Mutex::new(NumberPool::new(low_port..high_port)));
    fn with_matches(matches: Matches) -> impl Filter<Extract = (Matches,), Error = std::convert::Infallible> + Clone {
        return warp::any().map(move || matches.clone()); // .clone() just cloned the ref because it is an Arc
    }
    fn with_port_pool(port_pool: SharedNumberPool) -> impl Filter<Extract = (SharedNumberPool,), Error = std::convert::Infallible> + Clone {
        return warp::any().map(move || port_pool.clone());
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
        .and(with_port_pool(port_pool.clone()))
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
        .and(with_port_pool(port_pool.clone()))
        .and(warp::query::<JoinQuery>())
        .and(with_user())
        .and_then(cancel_match_handler);
    let ready_route = warp::path("ready")
        .and(warp::post())
        .and(with_matches(matches.clone()))
        .and(warp::query::<JoinQuery>())
        .and(with_user())
        .and_then(ready_handler);
    let match_updates_route = warp::path("updates")
        .and(warp::get())
        .and(with_matches(matches.clone()))
        .and(warp::query::<JoinQuery>())
        .and_then(match_ready_updates);
    let end_match_route = warp::path("end_match").and(warp::post()).and_then(end_match_handler);
    let health_route = warp::path("health").and(warp::get()).and_then(health_handler);
    let routes = matches_route
        .or(match_route)
        .or(create_match_route)
        .or(join_match_route)
        .or(cancel_match_route)
        .or(end_match_route)
        .or(ready_route)
        .or(match_updates_route)
        .or(health_route)
        .recover(handle_rejection);

    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());

    // Parse host and port
    let host: Ipv4Addr = host.parse().expect("Invalid HOST address");
    let port: u16 = port.parse().expect("Invalid PORT number");

    println!("Starting server at {}:{}", host, port);

    warp::serve(routes).run((host, port)).await;
}
