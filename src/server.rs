use dashmap::DashMap;
use dotenv;
use local_ip_address::local_ip;
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;

use std::{net::Ipv6Addr, path::PathBuf, sync::Arc};

use routes::setup_routes;

pub mod routes;

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

type UserPool = Arc<DashMap<u64, User>>;

#[tokio::main(flavor = "multi_thread")]
pub async fn build_server() {
    // Loads the .env file
    let _ = dotenv::dotenv();
    let port: u16 = validate_port(&dotenv::var("SERVER_PORT").unwrap());
    let ip = local_ip().unwrap();

    let pool: UserPool = Arc::new(DashMap::new());
    let app = setup_routes(pool);
    let listener = TcpListener::bind((ip, port)).await.unwrap();

    axum::serve(listener, app).await.unwrap();
}

fn validate_port(s: &str) -> u16 {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return 8080;
    }

    match trimmed.parse::<u16>() {
        Ok(0) => 8080,
        Ok(port) => port,
        Err(_) => 8080,
    }
}
