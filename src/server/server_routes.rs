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
        .route("/send", post(send_data))
        .with_state(pool)
}

#[cfg(test)]
mod tests {
    use super::super::{NetInfo, User};
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use dashmap::DashMap;
    use http_body_util::BodyExt;
    use std::sync::Arc;
    use tower::ServiceExt;

    fn test_user(ip: &str, port: u16, path: &str) -> User {
        User {
            net: NetInfo {
                ip: ip.to_string(),
                port,
            },
            path: path.to_string(),
        }
    }

    #[tokio::test]
    async fn test_create_and_remove() {
        let pool: UserPool = Arc::new(DashMap::new());
        let app = setup_routes(pool.clone());

        // Mock a user pair connection
        let pair = (
            test_user("127.0.0.1", 8001, "/src"),
            test_user("127.0.0.1", 8002, "/dest"),
        );
        let payload = rkyv::to_bytes::<_, 256>(&pair).unwrap();

        // Create the first pair on the pool
        // Expects a `CREATED` status code
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/users")
                    .body(Body::from(payload.to_vec()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);

        // Extract hash to test a /quit request
        let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
        let hash = String::from_utf8(body_bytes.to_vec()).unwrap();
        assert_eq!(pool.len(), 1);

        let quit_response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/quit/{hash}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(quit_response.status(), StatusCode::OK);
        assert!(pool.is_empty());
    }

    #[tokio::test]
    async fn test_duplicated_connection_is_changed() {
        let pool: UserPool = Arc::new(DashMap::new());
        let app = setup_routes(pool.clone());

        // Mock a user pair connection
        let pair = (
            test_user("127.0.0.1", 8001, "/src"),
            test_user("127.0.0.1", 8002, "/dest"),
        );
        let payload = rkyv::to_bytes::<_, 256>(&pair).unwrap();

        // Create the first pair on the pool
        // Expects a `CREATED` status code
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/users")
                    .body(Body::from(payload.to_vec()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);

        // Duplicated request should output an OK
        let duplicated = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/users")
                    .body(Body::from(payload.to_vec()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(duplicated.status(), StatusCode::OK);
    }
}
