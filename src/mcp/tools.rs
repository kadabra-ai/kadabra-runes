//! MCP tool definitions for code navigation.
//!
//! This module defines the tools exposed by the MCP server. Each tool
//! corresponds to a language server capability and provides LLM-friendly
//! access to semantic code navigation.
//!
//! Note: Types here appear unused because they're consumed by proc macros.
//!
//! # Available Tools

// Allow dead code warnings - types are used by #[tool] and #[tool_router] macros
#![allow(dead_code)]
//!
//! ## High Priority (Must Have)
//! - `goto_definition` - Jump to symbol definition
//! - `find_references` - Find all references to a symbol
//! - `hover` - Get type info and documentation
//! - `document_symbols` - List symbols in a file
//! - `workspace_symbols` - Search symbols across workspace
//! - `incoming_calls` - Find callers of a function
//! - `outgoing_calls` - Find functions called by a function
//! - `implementations` - Find implementations of a trait/interface
//! - `type_definition` - Jump to type definition
//!
//! ## Nice to Have (Future)
//! - `diagnostics` - Get errors and warnings
//! - `signature_help` - Get function signature info
//! - `rename_preview` - Preview rename refactoring
//! - `code_actions` - Get available quick fixes

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// TODO: Phase 2 - Implement tool definitions and handlers
//
// Each tool will:
// 1. Define input parameters as a struct
// 2. Define output format as a struct
// 3. Implement handler that calls LSP client
// 4. Format response for LLM consumption
//
// Integration with rmcp:
// Use the #[tool] macro from rmcp to define tools:
// ```ignore
// #[tool]
// async fn goto_definition(params: GotoDefinitionParams) -> ToolResult {
//     // Implementation
// }
// ```

/// Common input for position-based tool calls.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PositionParams {
    /// Path to the file.
    #[schemars(description = "Absolute path to the source file")]
    pub file_path: String,
    /// Line number (1-indexed for user-friendliness).
    #[schemars(description = "Line number (1-indexed)")]
    pub line: u32,
    /// Column number (1-indexed for user-friendliness).
    #[schemars(description = "Column number (1-indexed)")]
    pub column: u32,
}

/// Input for symbol-based queries by name with an optional file path filter.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SymbolNameParams {
    /// The symbol name to search for.
    #[schemars(description = "The symbol name to search for")]
    pub symbol: String,
    /// Optional file path to narrow the search.
    #[schemars(description = "Optional file path to narrow the search scope")]
    pub file_path: Option<String>,
}

/// Input for symbol-based queries that can use either position or symbol name.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", content = "data", rename_all = "camelCase")]
pub enum SymbolQuery {
    /// Query by position in a file.
    Position(PositionParams),
    /// Query by symbol name.
    Name(SymbolNameParams),
}

/// Parameters for the `goto_definition` tool.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GotoDefinitionParams {
    /// The symbol to find the definition of.
    #[schemars(description = "The symbol to find the definition of (by position or name)")]
    pub query: SymbolQuery,
}

/// Parameters for the `find_references` tool.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FindReferencesParams {
    /// The symbol to find references to.
    #[schemars(description = "The symbol to find references to (by position or name)")]
    pub query: SymbolQuery,
    /// Whether to include the declaration in the results.
    #[serde(default)]
    #[schemars(description = "Whether to include the declaration in the results (default: false)")]
    pub include_declaration: bool,
}

/// Parameters for the hover tool.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct HoverParams {
    /// Position to get hover info for.
    #[schemars(description = "Position in the file to get hover info for")]
    pub position: PositionParams,
}

/// Parameters for the `document_symbols` tool.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DocumentSymbolsParams {
    /// Path to the file.
    #[schemars(description = "Absolute path to the source file to list symbols from")]
    pub file_path: String,
}

/// Parameters for the `workspace_symbols` tool.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceSymbolsParams {
    /// Query string to search for.
    #[schemars(description = "Query string to search for symbols across the workspace")]
    pub query: String,
    /// Maximum number of results to return.
    #[serde(default = "default_max_results")]
    #[schemars(description = "Maximum number of results to return (default: 50)")]
    pub max_results: u32,
}

fn default_max_results() -> u32 {
    50
}

/// Parameters for the `incoming_calls` tool.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct IncomingCallsParams {
    /// Position of the function to find callers for.
    #[schemars(description = "Position of the function to find callers for")]
    pub position: PositionParams,
}

/// Parameters for the `outgoing_calls` tool.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct OutgoingCallsParams {
    /// Position of the function to find callees for.
    #[schemars(description = "Position of the function to find callees for")]
    pub position: PositionParams,
}

/// Parameters for the implementations tool.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ImplementationsParams {
    /// The trait/interface to find implementations for.
    #[schemars(
        description = "The trait/interface to find implementations for (by position or name)"
    )]
    pub query: SymbolQuery,
}

/// Parameters for the `type_definition` tool.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TypeDefinitionParams {
    /// Position to get type definition for.
    #[schemars(description = "Position in the file to get type definition for")]
    pub position: PositionParams,
}

/// A location in the source code with context.
/// Note: Currently unused - reserved for future structured JSON responses.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct LocationWithContext {
    /// Path to the file.
    pub file_path: String,
    /// Line number (1-indexed).
    pub line: u32,
    /// Column number (1-indexed).
    pub column: u32,
    /// The source code line at this location.
    pub context: String,
    /// Additional context lines before.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_before: Option<Vec<String>>,
    /// Additional context lines after.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_after: Option<Vec<String>>,
}

/// A symbol with its location.
/// Note: Currently unused - reserved for future structured JSON responses.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SymbolInfo {
    /// The symbol name.
    pub name: String,
    /// The kind of symbol (function, struct, trait, etc.).
    pub kind: String,
    /// Location of the symbol.
    pub location: LocationWithContext,
    /// Container name (e.g., the struct a method belongs to).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container: Option<String>,
}

/// Result of a hover operation.
/// Note: Currently unused - reserved for future structured JSON responses.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct HoverResult {
    /// The type signature or declaration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    /// Documentation for the symbol.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation: Option<String>,
}

/// Information about a call relationship.
/// Note: Currently unused - reserved for future structured JSON responses.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CallInfo {
    /// The function making or receiving the call.
    pub function: SymbolInfo,
    /// Locations where the call occurs within the function.
    pub call_sites: Vec<LocationWithContext>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_params_serialization() {
        let params = PositionParams {
            file_path: "/path/to/file.rs".to_string(),
            line: 10,
            column: 5,
        };
        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("filePath"));
        assert!(json.contains("/path/to/file.rs"));
    }

    #[test]
    fn test_symbol_query_deserialization() {
        // Position query
        let json = r#"{"kind": "position", "data": { "filePath": "/path/to/file.rs", "line": 10, "column": 5} }"#;
        let query: SymbolQuery = serde_json::from_str(json).unwrap();
        assert!(matches!(query, SymbolQuery::Position(_)));

        // Name query
        let json = r#"{"kind": "name", "data": { "symbol": "MyStruct"} }"#;
        let query: SymbolQuery = serde_json::from_str(json).unwrap();
        assert!(matches!(query, SymbolQuery::Name { .. }));
    }
}
