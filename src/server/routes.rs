use axum::{
    Router,
    routing::{get, post},
};

use super::UserPool;
use crud::*;

pub mod crud;

pub fn setup_routes(pool: UserPool) -> Router {
    Router::new()
        .route("/", get(root))
        .route("/users", post(create_user))
        .route("/quit/{hash}", post(remove_user))
        .with_state(pool)
}
