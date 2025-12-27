//! @acp:module "Map Handler"
//! @acp:summary "Directory structure endpoint"
//! @acp:domain daemon
//! @acp:layer api

use std::collections::BTreeMap;
use axum::{
    extract::{Query, State},
    Json,
};
use serde::{Deserialize, Serialize};

use crate::state::AppState;

#[derive(Deserialize)]
pub struct MapQuery {
    /// Maximum depth to traverse (default: 3)
    depth: Option<usize>,
}

#[derive(Serialize)]
pub struct MapNode {
    name: String,
    #[serde(rename = "type")]
    node_type: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    children: Option<Vec<MapNode>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    symbols: Option<usize>,
}

#[derive(Serialize)]
pub struct MapResponse {
    tree: MapNode,
    total_files: usize,
    total_dirs: usize,
}

/// GET /map - Get directory structure
pub async fn get_map(
    State(state): State<AppState>,
    Query(query): Query<MapQuery>,
) -> Json<MapResponse> {
    let cache = state.cache_async().await;
    let max_depth = query.depth.unwrap_or(3);

    // Build tree from file paths
    let mut dir_tree: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut file_symbols: BTreeMap<String, usize> = BTreeMap::new();

    for (path, file_entry) in &cache.files {
        // Count symbols per file
        file_symbols.insert(path.clone(), file_entry.exports.len());

        // Build directory hierarchy
        let parts: Vec<&str> = path.split('/').collect();
        for i in 0..parts.len() {
            let dir_path = if i == 0 {
                ".".to_string()
            } else {
                parts[..i].join("/")
            };

            let child = if i + 1 == parts.len() {
                path.clone()
            } else {
                parts[..=i].join("/")
            };

            dir_tree.entry(dir_path).or_default().push(child);
        }
    }

    // Deduplicate children
    for children in dir_tree.values_mut() {
        children.sort();
        children.dedup();
    }

    // Count directories
    let total_dirs = dir_tree.len();

    // Build tree structure
    fn build_node(
        name: &str,
        path: &str,
        dir_tree: &BTreeMap<String, Vec<String>>,
        file_symbols: &BTreeMap<String, usize>,
        depth: usize,
        max_depth: usize,
    ) -> MapNode {
        if let Some(symbols) = file_symbols.get(path) {
            // It's a file
            MapNode {
                name: name.to_string(),
                node_type: "file",
                children: None,
                symbols: Some(*symbols),
            }
        } else if let Some(children) = dir_tree.get(path) {
            // It's a directory
            let child_nodes = if depth < max_depth {
                Some(
                    children
                        .iter()
                        .map(|child| {
                            let child_name = child.rsplit('/').next().unwrap_or(child);
                            build_node(child_name, child, dir_tree, file_symbols, depth + 1, max_depth)
                        })
                        .collect(),
                )
            } else {
                None
            };

            MapNode {
                name: name.to_string(),
                node_type: "directory",
                children: child_nodes,
                symbols: None,
            }
        } else {
            MapNode {
                name: name.to_string(),
                node_type: "unknown",
                children: None,
                symbols: None,
            }
        }
    }

    let tree = build_node(".", ".", &dir_tree, &file_symbols, 0, max_depth);

    Json(MapResponse {
        tree,
        total_files: cache.files.len(),
        total_dirs,
    })
}
