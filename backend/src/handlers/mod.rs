pub(crate) mod analytics;
pub(crate) mod animals;
pub(crate) mod auth;
pub(crate) mod bulk_tank;
pub(crate) mod contacts;
pub(crate) mod feed;
pub(crate) mod fitness;
pub(crate) mod grazing;
pub(crate) mod lely;
pub(crate) mod locations;
pub(crate) mod milk;
pub(crate) mod reports;
pub(crate) mod reproduction;
pub(crate) mod settings;
pub(crate) mod tasks;
pub(crate) mod vet;

use axum::Router;

use crate::state::AppState;

pub use analytics::TrendQuery;
pub use reports::ReportFilter;
pub use settings::{ChangePasswordRequest, UpdateRoleRequest};

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
        .merge(lely::routes())
        .merge(vet::routes())
        .merge(tasks::routes())
}
