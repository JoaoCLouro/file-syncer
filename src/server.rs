use dashmap::DashMap;
use dotenv;
use local_ip_address::local_ip;
use rkyv::{Archive, Deserialize, Serialize};
use tokio::net::TcpListener;

use std::{net::Ipv6Addr, sync::Arc};

use crate::types::EventLog;
use routes::setup_routes;

pub mod routes;

#[derive(Debug, Clone, Hash, PartialEq, Eq, Archive, Serialize, Deserialize)]
#[archive(check_bytes)]
pub struct User {
    pub net: NetInfo,
    pub path: String,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Archive, Serialize, Deserialize)]
#[archive(check_bytes)]
pub struct NetInfo {
    pub ip: Ipv6Addr,
    pub port: u16,
}

#[derive(Debug, Clone, Hash, Archive, Serialize, Deserialize)]
#[archive(check_bytes)]
pub struct RequestInfo {
    pub hash: String,
    pub event: EventLog,
}

type UserPool = Arc<DashMap<String, (User, User)>>;

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
