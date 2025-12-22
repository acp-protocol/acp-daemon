//! @acp:module "Files Handler"
//! @acp:summary "File query endpoints"
//! @acp:domain daemon
//! @acp:layer api

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

use acp::cache::FileEntry;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct FileQuery {
    /// Filter by language (rust, typescript, python, etc.)
    language: Option<String>,
    /// Filter by domain (contains match)
    domain: Option<String>,
    /// Filter by layer
    layer: Option<String>,
    /// Maximum results to return
    limit: Option<usize>,
}

#[derive(Serialize)]
pub struct FileListResponse {
    files: Vec<FileEntry>,
    total: usize,
}

/// GET /files - List files with optional filtering
pub async fn list_files(
    State(state): State<AppState>,
    Query(query): Query<FileQuery>,
) -> Json<FileListResponse> {
    let cache = state.cache_async().await;

    let mut files: Vec<FileEntry> = cache.files.values()
        .filter(|f| {
            let lang_match = query.language.as_ref()
                .map(|l| format!("{:?}", f.language).to_lowercase() == l.to_lowercase())
                .unwrap_or(true);

            let domain_match = query.domain.as_ref()
                .map(|d| f.domains.iter().any(|fd| fd.contains(d)))
                .unwrap_or(true);

            let layer_match = query.layer.as_ref()
                .map(|l| f.layer.as_ref().map(|fl| fl == l).unwrap_or(false))
                .unwrap_or(true);

            lang_match && domain_match && layer_match
        })
        .cloned()
        .collect();

    let total = files.len();

    if let Some(limit) = query.limit {
        files.truncate(limit);
    }

    Json(FileListResponse { files, total })
}

/// GET /files/*path - Get a specific file by path
pub async fn get_file(
    State(state): State<AppState>,
    Path(path): Path<String>,
) -> Result<Json<FileEntry>, StatusCode> {
    let cache = state.cache_async().await;

    // Try exact match first
    if let Some(file) = cache.files.get(&path) {
        return Ok(Json(file.clone()));
    }

    // Try with leading slash removed
    let path_no_slash = path.trim_start_matches('/');
    if let Some(file) = cache.files.get(path_no_slash) {
        return Ok(Json(file.clone()));
    }

    // Try partial match (ends with)
    for (file_path, file) in &cache.files {
        if file_path.ends_with(&path) || file_path.ends_with(path_no_slash) {
            return Ok(Json(file.clone()));
        }
    }

    Err(StatusCode::NOT_FOUND)
}
