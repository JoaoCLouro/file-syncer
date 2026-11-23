use local_ip_address::local_ip;
use rkyv::{Archive, Deserialize, Serialize};
use std::collections::VecDeque;
use tokio::{net::TcpListener, sync::mpsc};

pub mod client_routes;
use client_routes::build_client;

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
pub struct Changes {
    line: u16,
    content: String,
}

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
pub struct RequestAction {
    path: String,
    actions: Vec<Changes>,
}

const DEFAULT_PORT: u16 = 3000;

async fn serve_client() {
    let port: u16 = validate_port(&dotenv::var("CLIENT_PORT").unwrap());
    let ip = local_ip().unwrap();
    // tokio channel for inner local network connections
    let (tx, mut rx) = mpsc::channel(32);
    let requests = VecDeque::<RequestAction>::new();

    // Pass the receiver to the app so it can be used
    let app = build_client(rx.clone());
    let listener = TcpListener::bind((ip, port)).await.unwrap();
}

fn validate_port(s: &str) -> u16 {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return DEFAULT_PORT;
    }

    match trimmed.parse::<u16>() {
        Ok(0) => DEFAULT_PORT,
        Ok(port) => port,
        Err(_) => DEFAULT_PORT,
    }
}
