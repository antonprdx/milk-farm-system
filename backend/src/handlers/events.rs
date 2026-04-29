use std::convert::Infallible;
use std::sync::Arc;

use axum::extract::State;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::routing::get;
use axum::Router;
use futures::stream::Stream;
use serde::Serialize;
use tokio_stream::StreamExt as _;

use crate::middleware::auth::ClaimsAllowMustChange;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize)]
pub struct SseMessage {
    pub event_type: String,
    pub data: serde_json::Value,
}

pub struct EventBus {
    tx: tokio::sync::broadcast::Sender<SseMessage>,
}

impl EventBus {
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = tokio::sync::broadcast::channel(capacity);
        Self { tx }
    }

    pub fn publish(&self, event_type: &str, data: serde_json::Value) {
        let msg = SseMessage {
            event_type: event_type.to_string(),
            data,
        };
        if self.tx.send(msg).is_err() {
            tracing::debug!("No SSE subscribers for event");
        }
    }

    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<SseMessage> {
        self.tx.subscribe()
    }
}

pub fn create_event_bus() -> Arc<EventBus> {
    Arc::new(EventBus::new(256))
}

async fn sse_stream(
    State(state): State<AppState>,
    _auth: ClaimsAllowMustChange,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let receiver = state.event_bus.subscribe();

    let stream = tokio_stream::wrappers::BroadcastStream::new(receiver).filter_map(
        |result| match result {
            Ok(msg) => {
                let event = Event::default()
                    .event(&msg.event_type)
                    .data(serde_json::to_string(&msg.data).unwrap_or_default());
                Some(Ok(event))
            }
            Err(_) => None,
        },
    );

    Sse::new(stream).keep_alive(KeepAlive::default())
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/events/stream", get(sse_stream))
}
