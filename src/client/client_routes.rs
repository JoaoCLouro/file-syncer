use axum::{Router, routing::post};

use super::{AppState, RequestAction};
use client_crud::*;

pub mod client_crud;

pub fn build_client(state: AppState<RequestAction>) -> Router {
    Router::new()
        .route("/send", post(send_request))
        .route("/load", post(load_request))
        .route("/receive", post(execute_action))
        .with_state(state)
}
