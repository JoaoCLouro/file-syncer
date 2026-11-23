use axum::{Router, routing::post};

use super::AppState;

pub fn build_client<T: Send + Clone + 'static>(state: AppState<T>) -> Router {
    Router::new()
        .route("/send", post(send_request))
        .route("/receive", post(execute_action))
        .with_state(state)
}
