//! Error types for the kadabra-runes MCP server.
//!
//! This module defines all error types used throughout the application,
//! organized by subsystem: LSP, MCP, Transport, and Tools.
//!
//! Note: Error variants defined for comprehensive error handling and future use.

// Allow dead code warnings - error types are for comprehensive coverage
#![allow(dead_code)]

use thiserror::Error;

/// Errors related to LSP client operations.
#[derive(Debug, Error)]
pub enum LspError {
    /// The language server process failed to start.
    #[error("failed to start language server: {0}")]
    ServerStartFailed(String),

    /// The language server process exited unexpectedly.
    #[error("language server exited unexpectedly: {0}")]
    ServerExited(String),

    /// Failed to initialize the language server.
    #[error("language server initialization failed: {0}")]
    InitializationFailed(String),

    /// The language server returned an error response.
    #[error("language server error: {message} (code: {code})")]
    ServerError {
        /// The error code from the language server.
        code: i32,
        /// The error message from the language server.
        message: String,
    },

    /// A request to the language server timed out.
    #[error("language server request timed out after {0:?}")]
    Timeout(std::time::Duration),

    /// The language server is not initialized.
    #[error("language server not initialized")]
    NotInitialized,

    /// Failed to send a request to the language server.
    #[error("failed to send request to language server: {0}")]
    RequestFailed(String),

    /// Failed to parse the response from the language server.
    #[error("failed to parse language server response: {0}")]
    ParseError(String),

    /// The requested capability is not supported by the language server.
    #[error("capability not supported: {0}")]
    CapabilityNotSupported(String),

    /// Invalid position in document.
    #[error("invalid position: line {line}, column {column}")]
    InvalidPosition {
        /// The line number.
        line: u32,
        /// The column number.
        column: u32,
    },

    /// Document not found or not open.
    #[error("document not found: {0}")]
    DocumentNotFound(String),
}

/// Errors related to MCP server operations.
#[derive(Debug, Error)]
pub enum McpError {
    /// Failed to parse an MCP request.
    #[error("failed to parse MCP request: {0}")]
    ParseError(String),

    /// The requested method is not supported.
    #[error("method not found: {0}")]
    MethodNotFound(String),

    /// Invalid parameters in the request.
    #[error("invalid parameters: {0}")]
    InvalidParams(String),

    /// Internal server error during request processing.
    #[error("internal error: {0}")]
    InternalError(String),

    /// The server is shutting down.
    #[error("server is shutting down")]
    ShuttingDown,

    /// Tool execution failed.
    #[error("tool error: {0}")]
    ToolError(#[from] ToolError),

    /// Protocol version mismatch.
    #[error("protocol version mismatch: expected {expected}, got {actual}")]
    ProtocolVersionMismatch {
        /// The expected protocol version.
        expected: String,
        /// The actual protocol version received.
        actual: String,
    },
}

/// Errors related to transport layer operations.
#[derive(Debug, Error)]
pub enum TransportError {
    /// Failed to read from stdin.
    #[error("stdin read error: {0}")]
    StdinReadError(String),

    /// Failed to write to stdout.
    #[error("stdout write error: {0}")]
    StdoutWriteError(String),

    /// Connection was closed unexpectedly.
    #[error("connection closed")]
    ConnectionClosed,

    /// Failed to serialize a message.
    #[error("serialization error: {0}")]
    SerializationError(String),

    /// Failed to deserialize a message.
    #[error("deserialization error: {0}")]
    DeserializationError(String),

    /// Invalid message format.
    #[error("invalid message format: {0}")]
    InvalidFormat(String),

    /// IO error during transport operations.
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Errors related to tool execution.
#[derive(Debug, Error)]
pub enum ToolError {
    /// The requested tool was not found.
    #[error("tool not found: {0}")]
    NotFound(String),

    /// Invalid arguments provided to the tool.
    #[error("invalid tool arguments: {0}")]
    InvalidArguments(String),

    /// The tool execution failed.
    #[error("tool execution failed: {0}")]
    ExecutionFailed(String),

    /// The file specified in the tool arguments was not found.
    #[error("file not found: {0}")]
    FileNotFound(String),

    /// Failed to read the file content.
    #[error("failed to read file: {0}")]
    FileReadError(String),

    /// The symbol was not found at the specified location.
    #[error("symbol not found at position")]
    SymbolNotFound,

    /// LSP error during tool execution.
    #[error("LSP error: {0}")]
    LspError(#[from] LspError),
}

/// A unified error type for the entire application.
#[derive(Debug, Error)]
pub enum Error {
    /// LSP-related error.
    #[error("LSP error: {0}")]
    Lsp(#[from] LspError),

    /// MCP-related error.
    #[error("MCP error: {0}")]
    Mcp(#[from] McpError),

    /// Transport-related error.
    #[error("transport error: {0}")]
    Transport(#[from] TransportError),

    /// Tool-related error.
    #[error("tool error: {0}")]
    Tool(#[from] ToolError),

    /// Configuration error.
    #[error("configuration error: {0}")]
    Config(String),

    /// Generic IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// A specialized Result type for kadabra-runes operations.
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lsp_error_display() {
        let err = LspError::ServerStartFailed("connection refused".to_string());
        assert_eq!(
            err.to_string(),
            "failed to start language server: connection refused"
        );
    }

    #[test]
    fn test_error_conversion() {
        let lsp_err = LspError::NotInitialized;
        let err: Error = lsp_err.into();
        assert!(matches!(err, Error::Lsp(LspError::NotInitialized)));
    }

    #[test]
    fn test_tool_error_from_lsp_error() {
        let lsp_err = LspError::DocumentNotFound("/path/to/file.rs".to_string());
        let tool_err: ToolError = lsp_err.into();
        assert!(matches!(tool_err, ToolError::LspError(_)));
    }
}
