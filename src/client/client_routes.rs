use axum::{Router, routing::post};

use super::{AppState, RequestAction};
use client_crud::*;

pub mod client_crud;

pub fn build_client(state: AppState<RequestAction>) -> Router {
    Router::new()
        .route("/send", post(send_request))
        .route("/load", post(load_request))
        .with_state(state)
}

#[cfg(test)]
mod test {
    use super::super::{Action, AppState, Changes, RequestAction, client_routes::build_client};
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use std::{
        collections::VecDeque,
        sync::{Arc, Mutex},
    };
    use tower::ServiceExt;

    fn test_state() -> AppState<RequestAction> {
        let (tx, rx) = tokio::sync::mpsc::channel::<RequestAction>(32);
        AppState {
            tx,
            rx: Arc::new(Mutex::new(rx)),
            requests: Arc::new(Mutex::new(VecDeque::<RequestAction>::new())),
        }
    }

    #[tokio::test]
    async fn test_send() {
        let state = test_state();
        let app = build_client(state.clone());

        // --------------------
        // Test normal request
        // --------------------

        let request = RequestAction {
            path: "/".to_string(),
            actions: vec![Changes {
                line: 0,
                content: "this is a test".to_string(),
                action: Action::Insert,
            }],
        };

        state.requests.lock().unwrap().push_back(request);

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/send")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // -------------------
        // Test empty request
        // -------------------

        state.requests.lock().unwrap().clear();

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/send")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NO_CONTENT);

        // ----------------------
        // Test receiver dropped
        // ----------------------

        let request = RequestAction {
            path: "/".to_string(),
            actions: vec![Changes {
                line: 0,
                content: "this is a test".to_string(),
                action: Action::Insert,
            }],
        };

        state.requests.lock().unwrap().push_back(request);

        state.rx.lock().unwrap().close();
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/send")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn test_load() {
        let state = test_state();
        let app = build_client(state.clone());

        // ---------------------
        // Test aligned request
        // ---------------------

        let request = RequestAction {
            path: "/".to_string(),
            actions: vec![Changes {
                line: 0,
                content: "This is a test request".to_string(),
                action: Action::Insert,
            }],
        };

        let payload = rkyv::to_bytes::<_, 256>(&request).unwrap();

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/load")
                    .body(Body::from(payload.to_vec()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::ACCEPTED);

        // -----------------------
        // Test unaligned request
        // -----------------------

        let request = RequestAction {
            path: "/".to_string(),
            actions: vec![Changes {
                line: 0,
                content: "This is a test request".to_string(),
                action: Action::Insert,
            }],
        };

        let archived = rkyv::to_bytes::<_, 256>(&request).unwrap();
        // One leading byte shifts every address by 1, so the archive is misaligned
        let mut unaligned = Vec::with_capacity(archived.len() + 1);
        unaligned.push(0u8);
        unaligned.extend_from_slice(&archived);

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/load")
                    .body(Body::from(unaligned))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
