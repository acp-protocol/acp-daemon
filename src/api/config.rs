//! @acp:module "Config Handler"
//! @acp:summary "Config schema endpoint"
//! @acp:domain daemon
//! @acp:layer api

use axum::{
    extract::State,
    Json,
};

use acp::config::Config;
use crate::state::AppState;

/// GET /config - Return config JSON
pub async fn get_config(
    State(state): State<AppState>,
) -> Json<Config> {
    let config = state.config().await;
    Json(config.clone())
}
