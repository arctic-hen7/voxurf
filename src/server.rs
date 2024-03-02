use axum::{extract::State, http::StatusCode, routing::get, Json, Router};
use serde::Serialize;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::voice::Dictation;

struct AppState {
    dictation: Mutex<Dictation>,
}

pub async fn serve() {
    let dictation = Mutex::new(Dictation::new().unwrap());
    let app_state = Arc::new(AppState { dictation });

    let app = Router::new()
        .route("/start-recording", get(start_recording))
        .route("/end-recording", get(end_recording))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}

async fn start_recording(State(state): State<Arc<AppState>>) -> StatusCode {
    let mut dictation = state.dictation.lock().await;
    dictation.start().unwrap();

    StatusCode::ACCEPTED
}

async fn end_recording(
    State(state): State<Arc<AppState>>,
) -> (StatusCode, Json<serde_json::Value>) {
    let mut dictation = state.dictation.lock().await;

    match dictation.end() {
        Ok(transcription) => {
            log::info!(
                "Recording ended successfully, transcription: {}",
                transcription
            );
            (
                StatusCode::OK,
                Json(json!({ "transcription": transcription })),
            )
        }
        Err(e) => {
            log::error!("Failed to end recording: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to end recording" })),
            )
        }
    }
}
