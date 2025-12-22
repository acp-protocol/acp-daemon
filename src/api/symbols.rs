//! @acp:module "Symbols Handler"
//! @acp:summary "Symbol query endpoints"
//! @acp:domain daemon
//! @acp:layer api

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

use acp::cache::SymbolEntry;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct SymbolQuery {
    /// Filter by file path (contains match)
    file: Option<String>,
    /// Filter by symbol type (function, class, etc.)
    #[serde(rename = "type")]
    symbol_type: Option<String>,
    /// Filter by exported status
    exported: Option<bool>,
    /// Maximum results to return
    limit: Option<usize>,
}

#[derive(Serialize)]
pub struct SymbolListResponse {
    symbols: Vec<SymbolEntry>,
    total: usize,
}

/// GET /symbols - List symbols with optional filtering
pub async fn list_symbols(
    State(state): State<AppState>,
    Query(query): Query<SymbolQuery>,
) -> Json<SymbolListResponse> {
    let cache = state.cache_async().await;

    let mut symbols: Vec<SymbolEntry> = cache.symbols.values()
        .filter(|s| {
            let file_match = query.file.as_ref()
                .map(|f| s.file.contains(f))
                .unwrap_or(true);

            let type_match = query.symbol_type.as_ref()
                .map(|t| format!("{:?}", s.symbol_type).to_lowercase() == t.to_lowercase())
                .unwrap_or(true);

            let exported_match = query.exported
                .map(|e| s.exported == e)
                .unwrap_or(true);

            file_match && type_match && exported_match
        })
        .cloned()
        .collect();

    let total = symbols.len();

    if let Some(limit) = query.limit {
        symbols.truncate(limit);
    }

    Json(SymbolListResponse { symbols, total })
}

/// GET /symbols/:name - Get a specific symbol
pub async fn get_symbol(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<SymbolEntry>, StatusCode> {
    let cache = state.cache_async().await;

    cache.symbols.get(&name)
        .cloned()
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}
