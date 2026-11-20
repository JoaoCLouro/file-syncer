use dashmap::DashMap;
use dotenv;
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;

use std::{
    net::Ipv6Addr,
    path::PathBuf,
    sync::Arc,
};

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
    // Requires a SERVER_PORT and SERVER_IP environment variable
    let server_ip = dotenv::var("SERVER_IP").unwrap();
    let server_port = dotenv::var("SERVER_PORT").unwrap();
    let pool: UserPool = Arc::new(DashMap::new());

    let app = setup_routes(pool);
    let listener = TcpListener::bind("{server_ip}:{server_port}")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}
