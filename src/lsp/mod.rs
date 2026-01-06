//! LSP (Language Server Protocol) client module.
//!
//! This module implements the LSP client that communicates with language
//! servers like rust-analyzer to provide semantic code navigation capabilities.
//!
//! # Architecture
//!
//! The LSP module is organized into:
//! - `client`: The main LSP client implementation
//! - `types`: Additional type definitions for LSP operations
//!
//! # Usage
//!
//! ```ignore
//! use code_navigator::lsp::LspClient;
//!
//! let client = LspClient::new("rust-analyzer").await?;
//! let definition = client.goto_definition(file, position).await?;
//! ```

pub mod client;
pub mod types;

// TODO: Phase 3 - Implement LSP client
//
// The LSP client will:
// 1. Spawn and manage language server processes
// 2. Handle LSP initialization handshake
// 3. Send requests and receive responses
// 4. Track document state for didOpen/didChange
// 5. Support multiple language servers (future)
//
// Key components to implement:
// - LspClient struct managing server lifecycle
// - Request/response correlation
// - Notification handling
// - Document synchronization
// - Capability negotiation

use crate::error::LspError;

/// Result type for LSP operations.
pub type LspResult<T> = std::result::Result<T, LspError>;

// Re-export commonly used types from lsp-types
// Note: These re-exports are for future use by the MCP server layer
#[allow(unused_imports)]
pub use lsp_types::{
    CallHierarchyIncomingCall, CallHierarchyOutgoingCall, DocumentSymbol, GotoDefinitionResponse,
    Hover, Location, Position, SymbolInformation, TextDocumentIdentifier,
    TextDocumentPositionParams, Url,
};
