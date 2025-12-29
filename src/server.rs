//! @acp:module "HTTP Server"
//! @acp:summary "Axum HTTP server with REST API routes"
//! @acp:domain daemon
//! @acp:layer transport
//!
//! Provides the HTTP server configuration and router for the daemon API.

use axum::{routing::get, Router};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::api;
use crate::state::AppState;

/// Create the main application router
pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Health check
        .route("/health", get(api::health::health_check))
        // Full schema endpoints
        .route("/cache", get(api::cache::get_cache))
        .route("/config", get(api::config::get_config))
        .route("/vars", get(api::vars::get_vars))
        // Symbol queries
        .route("/symbols/{name}", get(api::symbols::get_symbol))
        .route("/symbols", get(api::symbols::list_symbols))
        // File queries
        .route("/files/{*path}", get(api::files::get_file))
        .route("/files", get(api::files::list_files))
        // Graph queries
        .route("/callers/{symbol}", get(api::graph::get_callers))
        .route("/callees/{symbol}", get(api::graph::get_callees))
        // Domain queries
        .route("/domains/{name}", get(api::domains::get_domain))
        .route("/domains", get(api::domains::list_domains))
        // Constraint queries
        .route(
            "/constraints/{*path}",
            get(api::constraints::get_constraints),
        )
        // Variable expansion
        .route("/vars/{name}/expand", get(api::vars::expand_variable))
        // Aggregate endpoints
        .route("/stats", get(api::stats::get_stats))
        .route("/map", get(api::map::get_map))
        .route("/primer", get(api::primer::get_primer))
        // Add middleware
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(state)
}
