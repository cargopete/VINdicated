use axum::{routing::get, Router};

use crate::AppState;

mod health;
mod vin;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health::health))
        .route("/vin/:vin", get(vin::lookup_vin))
}
