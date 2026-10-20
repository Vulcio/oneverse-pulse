use std::sync::Arc;
use axum::{extract::State, http::header, response::{Html, IntoResponse}, routing::{get, post}, Json, Router};
use serde::Serialize;
use songverse_player_core::{advance, Set};
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
    Json(Snapshot { set: state.set.clone(), index: state.index })
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
async fn health() -> &'static str { "ok" }
async fn screen() -> impl IntoResponse {
    ([(header::CACHE_CONTROL, "no-store")], Html(include_str!("../../../web/renderer/index.html")))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let state = Arc::new(Mutex::new(PlayerState { set: Set::demo(), index: 0 }));
    let app = Router::new()
        .route("/health", get(health))
        .route("/api/state", get(snapshot))
        .route("/api/next", post(next))
        .route("/api/previous", post(previous))
        .route("/screen", get(screen))
        .with_state(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8787").await?;
    println!("Songverse Player: http://127.0.0.1:8787/screen");
    axum::serve(listener, app).await?;
    Ok(())
}
