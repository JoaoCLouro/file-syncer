use axum::{body::Bytes, extract::State, http::StatusCode};
use rkyv::{AlignedVec, from_bytes};

use super::super::{AppState, RequestAction};

pub async fn load_request(State(state): State<AppState<RequestAction>>, body: Bytes) -> StatusCode {
    // Consumes the request body received
    // Requires alignment for rkyv serialization
    let request = if body.as_ptr().align_offset(16) == 0 {
        from_bytes::<RequestAction>(&body)
    } else {
        let mut aligned = AlignedVec::with_capacity(body.len());
        aligned.extend_from_slice(&body);
        from_bytes::<RequestAction>(&aligned)
    };

    let Ok(request) = request else {
        return StatusCode::BAD_REQUEST;
    };

    state.requests.lock().unwrap().push_back(request);
    StatusCode::ACCEPTED
}

pub async fn send_request(State(state): State<AppState<RequestAction>>) -> StatusCode {
    // Fetches requests in a locked scope for mutex safety
    let next_req = state.requests.lock().unwrap().pop_back();

    let Some(req) = next_req else {
        return StatusCode::NO_CONTENT;
    };

    match state.tx.send(req).await {
        Ok(()) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR, // Receiver was dropped
    }
}
