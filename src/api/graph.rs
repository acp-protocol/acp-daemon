//! @acp:module "Graph Handler"
//! @acp:summary "Call graph query endpoints"
//! @acp:domain daemon
//! @acp:layer api

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::Serialize;

use crate::state::AppState;

#[derive(Serialize)]
pub struct GraphResponse {
    symbol: String,
    relationships: Vec<String>,
    count: usize,
}

/// GET /callers/:symbol - Get callers of a symbol (reverse graph)
pub async fn get_callers(
    State(state): State<AppState>,
    Path(symbol): Path<String>,
) -> Result<Json<GraphResponse>, StatusCode> {
    let cache = state.cache_async().await;

    // Look up in reverse graph
    if let Some(graph) = &cache.graph {
        if let Some(callers) = graph.reverse.get(&symbol) {
            return Ok(Json(GraphResponse {
                symbol: symbol.clone(),
                relationships: callers.clone(),
                count: callers.len(),
            }));
        }
    }

    // Symbol exists but no callers
    if cache.symbols.contains_key(&symbol) {
        return Ok(Json(GraphResponse {
            symbol,
            relationships: vec![],
            count: 0,
        }));
    }

    Err(StatusCode::NOT_FOUND)
}

/// GET /callees/:symbol - Get callees of a symbol (forward graph)
pub async fn get_callees(
    State(state): State<AppState>,
    Path(symbol): Path<String>,
) -> Result<Json<GraphResponse>, StatusCode> {
    let cache = state.cache_async().await;

    // Look up in forward graph
    if let Some(graph) = &cache.graph {
        if let Some(callees) = graph.forward.get(&symbol) {
            return Ok(Json(GraphResponse {
                symbol: symbol.clone(),
                relationships: callees.clone(),
                count: callees.len(),
            }));
        }
    }

    // Symbol exists but no callees
    if cache.symbols.contains_key(&symbol) {
        return Ok(Json(GraphResponse {
            symbol,
            relationships: vec![],
            count: 0,
        }));
    }

    Err(StatusCode::NOT_FOUND)
}
