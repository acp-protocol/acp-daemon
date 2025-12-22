//! @acp:module "Health Check Handler"
//! @acp:summary "Health check endpoint for daemon status"
//! @acp:domain daemon
//! @acp:layer api

use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct HealthResponse {
    status: String,
    version: String,
}

/// GET /health - Health check endpoint
pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}
