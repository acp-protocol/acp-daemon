//! @acp:module "Stats Handler"
//! @acp:summary "Project statistics endpoint"
//! @acp:domain daemon
//! @acp:layer api

use axum::{extract::State, Json};
use serde::Serialize;

use crate::state::AppState;

#[derive(Serialize)]
pub struct StatsResponse {
    files: usize,
    symbols: usize,
    lines: usize,
    annotation_coverage: f64,
    domains: usize,
}

/// GET /stats - Get project statistics
pub async fn get_stats(State(state): State<AppState>) -> Json<StatsResponse> {
    let cache = state.cache_async().await;

    Json(StatsResponse {
        files: cache.stats.files,
        symbols: cache.stats.symbols,
        lines: cache.stats.lines,
        annotation_coverage: cache.stats.annotation_coverage,
        domains: cache.domains.len(),
    })
}
