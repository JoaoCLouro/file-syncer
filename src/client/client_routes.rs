use axum::{
    Router,
    routing::{get, post},
};
use tokio::sync::watch::Receiver;

pub fn build_client(rx: Receiver) -> Routes {}
