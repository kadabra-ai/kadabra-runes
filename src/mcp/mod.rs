//! MCP (Model Context Protocol) server module.
//!
//! This module implements the MCP server that exposes code navigation tools
//! to LLM applications like Claude Code. The server uses stdio transport
//! to communicate with clients.
//!
//! # Architecture
//!
//! The MCP module is organized into:
//! - `transport`: Handles stdio-based JSON-RPC communication
//! - `tools`: Defines and implements the navigation tools
//!
//! # Usage
//!
//! ```ignore
//! use code_navigator::mcp::McpServer;
//!
//! let server = McpServer::new(lsp_client);
//! server.run().await?;
//! ```

pub mod server;
pub mod tools;
// pub mod transport;

// Re-export the KadabraRunes for convenient access
pub use server::KadabraRunes;

// TODO: Phase 2 - Implement MCP server
//
// The MCP server will:
// 1. Use rmcp crate for protocol handling
// 2. Expose tools defined in the tools module
// 3. Route tool calls to the LSP client
// 4. Return formatted responses suitable for LLM consumption
//
// Key components to implement:
// - McpServer struct holding LSP client reference
// - ServerHandler implementation for rmcp
// - Tool registration and routing
// - Error handling and response formatting

use crate::error::McpError;

/// Result type for MCP operations.
#[allow(dead_code)]
pub type McpResult<T> = std::result::Result<T, McpError>;
