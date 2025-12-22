#![forbid(unsafe_code)]

//! @acp:module "ACP Daemon Entry Point"
//! @acp:summary "Main entry point for the ACP daemon (acpd)"
//! @acp:domain daemon
//! @acp:layer application
//!
//! The ACP daemon provides:
//! - HTTP REST API for cache queries
//! - Schema loading (config, cache, vars)
//! - Variable expansion
//! - Process lifecycle management

use std::net::SocketAddr;
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use tracing::{info, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod server;
mod lifecycle;
mod api;
mod state;

/// ACP Daemon - Background service for codebase intelligence
#[derive(Parser, Debug)]
#[command(name = "acpd")]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Run in foreground mode (don't daemonize)
    #[arg(long, short = 'f')]
    foreground: bool,

    /// HTTP server port
    #[arg(long, default_value = "9222")]
    port: u16,

    /// Project root directory
    #[arg(long, short = 'C')]
    directory: Option<PathBuf>,

    /// Log level (trace, debug, info, warn, error)
    #[arg(long, default_value = "info")]
    log_level: String,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Start the daemon
    Start {
        /// Run in foreground mode
        #[arg(long, short = 'f')]
        foreground: bool,
    },
    /// Stop the daemon
    Stop,
    /// Check daemon status
    Status,
    /// Run daemon in foreground (alias for --foreground)
    Run,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    init_logging(&cli.log_level);

    // Determine project root
    let project_root = cli.directory.unwrap_or_else(|| {
        std::env::current_dir().expect("Failed to get current directory")
    });

    match cli.command {
        Some(Commands::Start { foreground }) => {
            if foreground || cli.foreground {
                run_foreground(project_root, cli.port).await
            } else {
                lifecycle::start_daemon(project_root, cli.port)
            }
        }
        Some(Commands::Stop) => {
            lifecycle::stop_daemon(&project_root)
        }
        Some(Commands::Status) => {
            lifecycle::check_status(&project_root)
        }
        Some(Commands::Run) | None => {
            // Default: run in foreground
            run_foreground(project_root, cli.port).await
        }
    }
}

fn init_logging(level: &str) {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(level));

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .init();
}

async fn run_foreground(project_root: PathBuf, port: u16) -> anyhow::Result<()> {
    info!("Starting ACP daemon in foreground mode");
    info!("Project root: {}", project_root.display());
    info!("HTTP port: {}", port);

    // Load ACP state (config, cache, vars)
    let state = match state::AppState::load(&project_root).await {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to load ACP state: {}", e);
            return Err(e);
        }
    };

    {
        let cache = state.cache_async().await;
        info!(
            "Loaded cache with {} files, {} symbols",
            cache.files.len(),
            cache.symbols.len()
        );
    }

    // Build router
    let app = server::create_router(state);

    // Bind and serve
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    info!("Listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("Daemon stopped");
    Ok(())
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install CTRL+C handler");
    info!("Received shutdown signal");
}
