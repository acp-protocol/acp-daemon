//! @acp:module "Domains Handler"
//! @acp:summary "Domain query endpoints"
//! @acp:domain daemon
//! @acp:layer api

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::Serialize;

use crate::state::AppState;
use acp::cache::DomainEntry;

#[derive(Serialize)]
pub struct DomainListResponse {
    domains: Vec<DomainEntry>,
    total: usize,
}

/// GET /domains - List all domains
pub async fn list_domains(State(state): State<AppState>) -> Json<DomainListResponse> {
    let cache = state.cache_async().await;

    let domains: Vec<DomainEntry> = cache.domains.values().cloned().collect();

    let total = domains.len();

    Json(DomainListResponse { domains, total })
}

/// GET /domains/:name - Get a specific domain
pub async fn get_domain(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<DomainEntry>, StatusCode> {
    let cache = state.cache_async().await;

    cache
        .domains
        .get(&name)
        .cloned()
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}
