//! Kadabra Runes MCP Server
//!
//! An MCP (Model Context Protocol) server that bridges LLM applications
//! (like Claude Code) with language servers (like rust-analyzer) to enable
//! semantic code navigation.
//!
//! # Overview
//!
//! This library provides:
//! - MCP server implementation with stdio transport
//! - LSP client for communicating with language servers
//! - Tools for semantic code navigation (goto definition, find references, etc.)
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────┐     stdio      ┌─────────────────┐
//! │   LLM Client    │◄──────────────►│   MCP Server    │
//! │  (Claude Code)  │    (MCP)       │ (kadabra-runes) │
//! └─────────────────┘                └────────┬────────┘
//!                                             │
//!                                      ┌──────▼──────┐
//!                                      │  LSP Client │
//!                                      └──────┬──────┘
//!                                             │ JSON-RPC
//!                                      ┌──────▼────────┐
//!                                      │   Language    │
//!                                      │   Server      │
//!                                      │(rust-analyzer)│
//!                                      └───────────────┘
//! ```
//!
//! # Modules
//!
//! - [`error`] - Error types for the entire application
//! - [`mcp`] - MCP server implementation
//! - [`lsp`] - LSP client implementation
//!
//! # Example
//!
//! ```ignore
//! use code_navigator::{mcp::McpServer, lsp::LspClient};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Initialize LSP client
//!     let lsp_client = LspClient::builder()
//!         .server_command("rust-analyzer")
//!         .workspace_root(".")
//!         .build()
//!         .await?;
//!
//!     // Start MCP server
//!     let server = McpServer::new(lsp_client);
//!     server.run().await?;
//!
//!     Ok(())
//! }
//! ```

// Enforce documentation and other quality attributes
#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
// Allow some pedantic lints that are too strict
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]

pub mod error;
pub mod lsp;
pub mod mcp;

// Re-export commonly used types at the crate root
pub use error::{Error, Result};
