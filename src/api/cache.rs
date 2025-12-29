//! @acp:module "Cache Handler"
//! @acp:summary "Full cache schema endpoint"
//! @acp:domain daemon
//! @acp:layer api

use axum::{extract::State, Json};

use crate::state::AppState;
use acp::cache::Cache;

/// GET /cache - Return full cache JSON
pub async fn get_cache(State(state): State<AppState>) -> Json<Cache> {
    let cache = state.cache_async().await;
    Json(cache.clone())
}
