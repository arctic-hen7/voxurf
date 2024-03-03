use axum::{extract::State, http::{StatusCode, Method}, routing::get, Router};
use tower_http::cors::{CorsLayer, Any};
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::Mutex;

use crate::voice::Dictation;

struct AppState {
    dictation: Mutex<Dictation>,
}

pub async fn serve() {
    let dictation = Mutex::new(Dictation::new().await.unwrap());
    let app_state = Arc::new(AppState { dictation });

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(Any);

    let app = Router::new()
        .route("/start-recording", get(start_recording))
        .route("/end-recording", get(end_recording))
        .layer(cors)
        .with_state(app_state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    log::info!("Starting server at {}", addr);

    axum::serve(listener, app).await.unwrap();
}

async fn start_recording(State(state): State<Arc<AppState>>) -> StatusCode {
    log::info!("Starting recording");

    let mut dictation = state.dictation.lock().await;
    dictation.start().unwrap();

    StatusCode::OK
}

async fn end_recording(State(state): State<Arc<AppState>>) -> (StatusCode, String) {
    log::info!("Ending recording");

    let mut dictation = state.dictation.lock().await;

    match dictation.end() {
        Ok(transcription) => {
            log::info!(
                "Recording ended successfully, transcription: {}",
                transcription
            );
            (StatusCode::OK, transcription)
        }
        Err(e) => {
            let error_msg = format!("Failed to end recoridng: {:?}", e);
            log::error!("{}", error_msg);
            panic!("{}", error_msg);
        }
    }
}
