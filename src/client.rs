use local_ip_address::local_ip;
use rkyv::{Archive, Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use tokio::{
    net::TcpListener,
    sync::mpsc::{Receiver, Sender, channel},
};

pub mod client_routes;
use client_routes::build_client;

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[archive(check_bytes)]
pub enum Action {
    Replace,
    Remove,
    Insert,
}

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[archive(check_bytes)]
pub struct Changes {
    line: u16,
    content: String,
    action: Action,
}

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[archive(check_bytes)]
pub struct RequestAction {
    path: String,
    actions: Vec<Changes>,
}

#[derive(Debug, Clone)]
pub struct AppState<T> {
    tx: Sender<T>,
    rx: Arc<Mutex<Receiver<T>>>,
    requests: Arc<Mutex<VecDeque<RequestAction>>>,
}

const DEFAULT_PORT: u16 = 3000;

async fn serve_client() {
    let port: u16 = validate_port(&dotenv::var("CLIENT_PORT").unwrap());
    let ip = local_ip().unwrap();
    // tokio channel for inner local network connections
    let (tx, rx) = channel::<RequestAction>(32); //tokio::sync::mpcs::channel

    let state = AppState {
        tx: tx,
        rx: Arc::new(Mutex::new(rx)),
        requests: Arc::new(Mutex::new(VecDeque::<RequestAction>::new())),
    };
    // Pass the receiver to the app so it can be used
    let app = build_client(state);
    let listener = TcpListener::bind((ip, port)).await.unwrap();

    axum::serve(listener, app).await.unwrap();
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
