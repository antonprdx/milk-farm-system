mod analytics;
mod animals;
mod auth;
mod bulk_tank;
mod contacts;
mod feed;
mod fitness;
mod grazing;
mod locations;
mod milk;
mod reports;
mod reproduction;
mod settings;

use axum::Router;

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .merge(auth::routes())
        .merge(animals::routes())
        .merge(milk::routes())
        .merge(reproduction::routes())
        .merge(feed::routes())
        .merge(fitness::routes())
        .merge(grazing::routes())
        .merge(contacts::routes())
        .merge(locations::routes())
        .merge(reports::routes())
        .merge(settings::routes())
        .merge(bulk_tank::routes())
        .merge(analytics::routes())
}
