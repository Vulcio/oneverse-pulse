//! Mock routes, enabled only by `pulsed --dev`.
use axum::{
    extract::State,
    http::header,
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use pulse_core::{advance, Set};
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::Mutex;

struct PlayerState {
    set: Set,
    index: usize,
}
type Shared = Arc<Mutex<PlayerState>>;

#[derive(Serialize)]
struct Snapshot {
    set: Set,
    index: usize,
}

async fn snapshot(State(state): State<Shared>) -> Json<Snapshot> {
    let state = state.lock().await;
    Json(Snapshot {
        set: state.set.clone(),
        index: state.index,
    })
}
async fn next(State(state): State<Shared>) -> Json<usize> {
    let mut state = state.lock().await;
    state.index = advance(state.index, state.set.slides.len(), 1);
    Json(state.index)
}
async fn previous(State(state): State<Shared>) -> Json<usize> {
    let mut state = state.lock().await;
    state.index = advance(state.index, state.set.slides.len(), -1);
    Json(state.index)
}
async fn screen() -> impl IntoResponse {
    (
        [(header::CACHE_CONTROL, "no-store")],
        Html(include_str!("../../../web/renderer/index.html")),
    )
}

/// Only constructed when the operator explicitly opts into mock data.
pub fn routes() -> Router {
    let state = Arc::new(Mutex::new(PlayerState {
        set: Set::demo(),
        index: 0,
    }));
    Router::new()
        .route("/api/state", get(snapshot))
        .route("/api/next", post(next))
        .route("/api/previous", post(previous))
        .route("/screen", get(screen))
        .with_state(state)
}
