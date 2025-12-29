//! @acp:module "Constraints Handler"
//! @acp:summary "Constraint query endpoints"
//! @acp:domain daemon
//! @acp:layer api

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::Serialize;

use crate::state::AppState;
use acp::constraints::Constraints;

#[derive(Serialize)]
pub struct ConstraintResponse {
    path: String,
    constraints: Option<Constraints>,
    lock_level: Option<String>,
}

/// GET /constraints/*path - Get constraints for a file path
pub async fn get_constraints(
    State(state): State<AppState>,
    Path(path): Path<String>,
) -> Result<Json<ConstraintResponse>, StatusCode> {
    let cache = state.cache_async().await;

    let path_normalized = path.trim_start_matches('/');

    // Get constraints from the constraint index
    let (constraints, lock_level) = if let Some(constraint_index) = &cache.constraints {
        let file_constraints = constraint_index.by_file.get(path_normalized).cloned();

        // Check lock level
        let lock = if constraint_index
            .by_lock_level
            .get("frozen")
            .map(|files| files.iter().any(|f| f == path_normalized))
            .unwrap_or(false)
        {
            Some("frozen".to_string())
        } else if constraint_index
            .by_lock_level
            .get("restricted")
            .map(|files| files.iter().any(|f| f == path_normalized))
            .unwrap_or(false)
        {
            Some("restricted".to_string())
        } else {
            None
        };

        (file_constraints, lock)
    } else {
        (None, None)
    };

    Ok(Json(ConstraintResponse {
        path: path_normalized.to_string(),
        constraints,
        lock_level,
    }))
}
