use warp::{Filter, Rejection, Reply};

use crate::error::UnauthorizedError;

pub struct User {
    pub id: u64,
    pub username: String,
    pub auth_token: String,
    pub balance: u64,
}
pub fn with_user() -> impl Filter<Extract = (User,), Error = Rejection> + Clone {
    warp::header::optional("Authorization")
        .and_then(|auth_header: Option<String>| async move {
            if let Some(auth) = auth_header {
                if auth.starts_with("Bearer ") {
                    // TODO: Add actual auth
                    // Extract the token by removing "Bearer " prefix
                    let token = auth.strip_prefix("Bearer ").unwrap().to_string();
                    let user = User {
                        id: 1,
                        username: token.clone(),
                        balance: 1000,
                        auth_token: token,
                    };
                    Ok(user)
                    // add back auth here
                    // if token == "valid_token" {
                    //     let user = User {
                    //         id: 1,
                    //         username: "Alice".to_string(),
                    //         balance: 1000,
                    //         auth_token: token,
                    //     };
                    //     Ok(user)
                    // } else {
                    //     Err(warp::reject::custom(UnauthorizedError)) // Custom error if token is invalid
                    // }
                } else {
                    println!("Invalid auth");
                    Err(warp::reject::custom(UnauthorizedError))
                }
            } else {
                println!("No auth");
                Err(warp::reject::custom(UnauthorizedError))
            }
        })
        .boxed()
}
