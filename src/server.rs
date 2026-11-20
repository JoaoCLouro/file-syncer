use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use dashmap::DashMap;
use dotenv;
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;

use std::{
    hash::{DefaultHasher, Hash, Hasher},
    net::Ipv6Addr,
    path::PathBuf,
    sync::Arc,
};

type UserPool = Arc<DashMap<u64, User>>;

#[derive(Debug, Clone, PartialEq, Hash, Eq, Deserialize, Serialize)]
pub struct User {
    pub net: NetInfo,
    pub path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Hash, Eq, Deserialize, Serialize)]
pub struct NetInfo {
    pub ip: Ipv6Addr,
    pub port: u16,
}

#[derive(Debug, Clone)]
pub struct SyncInfo {
    pub source: User,
    pub dest: User,
}

pub async fn build_server() {
    todo!("Add validation to the env vars and server creation");

    // Requires a SERVER_PORT and SERVER_IP environment variable
    let server_ip = dotenv::var("SERVER_IP").unwrap();
    let server_port = dotenv::var("SERVER_PORT").unwrap();
    let pool: UserPool = Arc::new(DashMap::new());

    let app = Router::new()
        .route("/", get(root))
        .route("/users", post(create_user))
        .route("/quit/{hash}", post(remove_user))
        .with_state(pool);
    let listener = TcpListener::bind("{server_ip}:{server_port}")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "Server serving"
}

async fn create_user(
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

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

async fn remove_user(State(pool): State<UserPool>, Path(user_hash): Path<u64>) -> StatusCode {
    match pool.remove(&user_hash) {
        Some(_) => StatusCode::OK,
        None => StatusCode::NOT_FOUND,
    }
}
