use axum::{
    body::Bytes,
    extract::{Path, State},
    http::StatusCode,
};
use rkyv::{Deserialize, Infallible, check_archived_root};
use sha2::{Digest, Sha256};

use super::super::{User, UserPool};

pub async fn root() -> &'static str {
    "Server serving"
}

pub async fn create_user(
    State(pool): State<UserPool>,
    body: Bytes,
) -> Result<(StatusCode, String), StatusCode> {
    // Validate the body received
    let archive =
        check_archived_root::<(User, User)>(&body).map_err(|_| StatusCode::BAD_REQUEST)?;

    // Deserialize the received content
    let content: (User, User) = archive
        .deserialize(&mut Infallible)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Hatch the received bytes to create a new user on the user pool
    let digest = Sha256::digest(&body);
    let hash: String = digest.iter().map(|b| format!("{b:02x}")).collect();

    match pool.insert(hash.clone(), content) {
        Some(_) => Ok((StatusCode::OK, hash)),
        None => Ok((StatusCode::CREATED, hash)),
    }
}

pub async fn remove_user(
    State(pool): State<UserPool>,
    Path(user_hash): Path<String>,
) -> StatusCode {
    match pool.remove(&user_hash) {
        Some(_) => StatusCode::OK,
        None => StatusCode::NOT_FOUND,
    }
}

pub async fn send_data(State(pool): State<UserPool>, body: Bytes) -> StatusCode {
    unimplemented!("To be completed!");
}
