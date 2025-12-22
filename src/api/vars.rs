//! @acp:module "Vars Handler"
//! @acp:summary "Variables schema and expansion endpoints"
//! @acp:domain daemon
//! @acp:layer api

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::Serialize;

use acp::vars::VarsFile;
use crate::state::AppState;

/// GET /vars - Return vars JSON
pub async fn get_vars(
    State(state): State<AppState>,
) -> Result<Json<VarsFile>, StatusCode> {
    let vars = state.vars().await;
    match vars.as_ref() {
        Some(v) => Ok(Json(v.clone())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

#[derive(Serialize)]
pub struct ExpandedVariable {
    name: String,
    expanded: String,
    description: Option<String>,
    source: Option<String>,
    lines: Option<[usize; 2]>,
}

/// GET /vars/:name/expand - Expand a variable to full content
pub async fn expand_variable(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<ExpandedVariable>, StatusCode> {
    let vars = state.vars().await;

    match vars.as_ref() {
        Some(v) => {
            // Look up variable in vars.variables
            if let Some(var) = v.variables.get(&name) {
                // For now, return the variable definition
                // Full expansion would resolve $SYM_*, $FILE_*, $DOM_* references
                // and read the source file content
                Ok(Json(ExpandedVariable {
                    name,
                    expanded: var.value.clone(),
                    description: var.description.clone(),
                    source: var.source.clone(),
                    lines: var.lines,
                }))
            } else {
                Err(StatusCode::NOT_FOUND)
            }
        }
        None => Err(StatusCode::NOT_FOUND),
    }
}
