use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json
};

use std::hash::{DefaultHasher, Hash, Hasher};

use super::super::{UserPool, User, NetInfo};

pub async fn root() -> &'static str {
    "Server serving"
}

pub async fn create_user(
    State(pool): State<UserPool>,
    Json(payload): Json<User>,
) -> (StatusCode, Json<u64>) {
    let user = User {
        net: NetInfo {
            ip: (payload.net.ip),
            port: (payload.net.port),
        },
        path: payload.path,
    };
    let hash = calculate_hash(&user);

    match pool.insert(hash, user.clone()) {
        // Already on the map => altered
        Some(_) => (StatusCode::OK, Json(hash)),
        // Not on the map => created
        None => (StatusCode::CREATED, Json(hash)),
    }
}

pub async fn remove_user(State(pool): State<UserPool>, Path(user_hash): Path<u64>) -> StatusCode {
    match pool.remove(&user_hash) {
        Some(_) => StatusCode::OK,
        None => StatusCode::NOT_FOUND,
    }
}

// Helpers

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}
