use axum::extract::State;
use axum::response::Json;
use axum::Router;
use axum::routing::get;
use serde::Serialize;

use crate::state::AppState;

#[derive(Serialize)]
pub struct DemoStatus {
    demo_mode: bool,
}

async fn status(State(state): State<AppState>) -> Json<DemoStatus> {
    Json(DemoStatus {
        demo_mode: state.config.demo_mode,
    })
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/demo/status", get(status))
}
