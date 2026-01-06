//! Kadabra Runes MCP Server - Entry Point
//!
//! This is the main entry point for the kadabra-runes MCP server.
//! It sets up logging, parses arguments, and starts the server.

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use rmcp::{ServiceExt, transport::stdio};
use tracing::{Level, info};
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

mod error;
mod lsp;
mod mcp;

use lsp::client::LspClient;
use mcp::KadabraRunes;

/// MCP server for semantic code navigation via language servers.
#[derive(Parser, Debug)]
#[command(name = "kadabra-runes")]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Workspace root directory to navigate.
    #[arg(short, long, default_value = ".")]
    workspace: PathBuf,

    /// Language server command to use.
    #[arg(short, long, default_value = "rust-analyzer")]
    language_server: String,

    /// Arguments to pass to the language server.
    #[arg(long)]
    language_server_args: Vec<String>,

    /// Log level: trace, debug, info, warn, error.
    #[arg(long, default_value = "info")]
    log_level: String,
}

impl Args {
    /// Parses the log level string into a tracing Level.
    fn parse_log_level(&self) -> Result<Level> {
        match self.log_level.to_lowercase().as_str() {
            "trace" => Ok(Level::TRACE),
            "debug" => Ok(Level::DEBUG),
            "info" => Ok(Level::INFO),
            "warn" => Ok(Level::WARN),
            "error" => Ok(Level::ERROR),
            other => anyhow::bail!("invalid log level: {}", other),
        }
    }
}

/// Initializes the tracing subscriber for logging.
fn init_tracing(level: Level) -> Result<()> {
    // Create an env filter that respects RUST_LOG but has a default level
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new(format!(
            "code_navigator={},tower={},async_lsp={}",
            level, level, level
        ))
    });

    // Set up the subscriber
    // Note: We write logs to stderr to keep stdout clean for MCP communication
    tracing_subscriber::registry()
        .with(filter)
        .with(
            fmt::layer()
                .with_writer(std::io::stderr)
                .with_ansi(true)
                .with_target(true)
                .with_thread_ids(false)
                .with_file(true)
                .with_line_number(true),
        )
        .try_init()
        .context("failed to initialize tracing subscriber")?;

    Ok(())
}

/// Main entry point.
#[tokio::main]
async fn main() -> Result<()> {
    // Parse command-line arguments
    let args = Args::parse();

    // Initialize tracing
    let log_level = args.parse_log_level()?;
    init_tracing(log_level)?;

    // Canonicalize the workspace path
    let workspace = args.workspace.canonicalize().context(format!(
        "failed to canonicalize workspace path: {}",
        args.workspace.display()
    ))?;

    info!(
        workspace = %workspace.display(),
        language_server = %args.language_server,
        "starting kadabra-runes MCP server"
    );

    // Create and initialize LSP client
    info!("initializing LSP client");
    let lsp_client = LspClient::builder()
        .server_command(&args.language_server)
        .server_args(args.language_server_args)
        .workspace_root(&workspace)
        .build()
        .await
        .context("failed to start LSP client")?;

    info!("LSP client initialized successfully");

    // Create KadabraRunes instance with LSP client
    let server = KadabraRunes::new(workspace, lsp_client);

    info!("starting MCP server with stdio transport");

    // Start the MCP server with stdio transport
    let service = server
        .serve(stdio())
        .await
        .context("failed to start MCP server")?;

    info!("MCP server started, waiting for messages");

    // Wait for the service to complete (handles graceful shutdown)
    service.waiting().await?;

    info!("MCP server shut down gracefully");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_args_parse_log_level() {
        let args = Args {
            workspace: PathBuf::from("."),
            language_server: "rust-analyzer".to_string(),
            language_server_args: vec![],
            log_level: "debug".to_string(),
        };
        assert_eq!(args.parse_log_level().unwrap(), Level::DEBUG);
    }
}
