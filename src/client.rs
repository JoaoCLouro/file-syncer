use std::time::Duration;
use tokio::{macros::select, net::TcpListener, select, sync::mpsc};

pub mod router;
use routes::build_client;

const DEFAULT_PORT: u16 = 3000;

async fn serve_client() {
    let port: u16 = validate_port(&dotenv::var("CLIENT_PORT").unwrap());
    let ip = local_ip().unwrap();
    // tokio channel for inner local network connections
    let (tx, mut rx) = mpsc::channel(32);

    // Pass the receiver to the app so it can be used
    let app = build_client(rx.clone());
    let listener = TcpListener::bind((ip, port)).await.unwrap();

    // Race the ip connection to the tokio channel to find witch connection is being used
    tokio::select! {
        // Should complement actions
        Some(r) = axum::serve(listener, app).await.unwrap();
        Some(r) = rx.recv().await.unwrap();
        // Could potentially race against a sleeper timeout
    }
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
