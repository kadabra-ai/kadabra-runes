//! MCP server implementation for kadabra-runes.
//!
//! This module contains the `KadabraRunes` struct that implements the MCP server
//! with code navigation tools powered by the Language Server Protocol.
#[allow(dead_code)]
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::lsp::client::LspClient;
use crate::lsp::types::{from_lsp_position, symbol_kind_to_string};
use lsp_types::{DocumentSymbolResponse, GotoDefinitionResponse};
use rmcp::handler::server::tool::ToolRouter;
use rmcp::{
    ErrorData as McpError, ServerHandler,
    handler::server::wrapper::Parameters,
    model::{
        CallToolResult, Content, ErrorCode, Implementation, ProtocolVersion, ServerCapabilities,
        ServerInfo,
    },
    tool, tool_handler, tool_router,
};

use super::tools::{
    DocumentSymbolsParams, FindReferencesParams, GotoDefinitionParams, HoverParams,
    ImplementationsParams, IncomingCallsParams, OutgoingCallsParams, SymbolNameParams, SymbolQuery,
    TypeDefinitionParams, WorkspaceSymbolsParams,
};

/// MCP server for semantic code navigation.
///
/// This struct implements the MCP server that exposes code navigation tools
/// to LLM applications via LSP integration.
#[derive(Clone)]
pub struct KadabraRunes {
    /// Root directory of the workspace to navigate.
    workspace_root: PathBuf,
    /// LSP client for semantic code navigation.
    lsp_client: Arc<LspClient>,
    #[allow(dead_code)]
    tool_router: ToolRouter<KadabraRunes>,
}

impl KadabraRunes {
    /// Creates a new `KadabraRunes` instance.
    ///
    /// # Arguments
    ///
    /// * `workspace_root` - Root directory of the workspace to navigate.
    /// * `lsp_client` - LSP client instance for code navigation.
    #[allow(dead_code)]
    pub fn new(workspace_root: PathBuf, lsp_client: LspClient) -> Self {
        Self {
            workspace_root,
            lsp_client: Arc::new(lsp_client),
            tool_router: Self::tool_router(),
        }
    }

    /// Returns the workspace root path.
    #[allow(dead_code)]
    pub fn workspace_root(&self) -> &PathBuf {
        &self.workspace_root
    }
}

// Helper functions for formatting LSP responses
// Note: These are called by the #[tool_router] macro-generated code,
// but the compiler's dead code analysis doesn't see through macros.

/// Reads context lines around a specific line in a file.
///
/// Returns a formatted string with line numbers and a marker for the target line.
#[allow(dead_code)]
fn read_context_lines(path: &Path, line: u32, context: usize) -> Result<String, std::io::Error> {
    let file_content = std::fs::read_to_string(path)?;
    let lines: Vec<_> = file_content.lines().collect();
    let line_idx = line.saturating_sub(1) as usize; // Convert to 0-indexed

    let start = line_idx.saturating_sub(context);
    let end = (line_idx + context + 1).min(lines.len());

    let mut result = String::new();
    for (idx, line_text) in lines[start..end].iter().enumerate() {
        let line_num = start + idx + 1;
        let marker = if line_num == (line_idx + 1) { ">" } else { " " };
        let _ = writeln!(result, "{marker} {line_num:4} | {line_text}");
    }
    Ok(result)
}

/// Formats a single LSP location with source context.
#[allow(dead_code)]
fn format_location(loc: &lsp_types::Location, context_lines: usize) -> Result<String, McpError> {
    let file_path = loc
        .uri
        .to_file_path()
        .map_err(|()| McpError::new(ErrorCode::INTERNAL_ERROR, "invalid file URI", None))?;

    let (line, column) = from_lsp_position(loc.range.start);

    let context = read_context_lines(&file_path, line, context_lines).map_err(|e| {
        McpError::new(
            ErrorCode::INTERNAL_ERROR,
            format!("failed to read file: {e}"),
            None,
        )
    })?;

    Ok(format!(
        "{}:{}:{}\n{}",
        file_path.display(),
        line,
        column,
        context
    ))
}

/// Formats multiple LSP locations with context.
#[allow(dead_code)]
fn format_locations(
    locations: &[lsp_types::Location],
    context_lines: usize,
) -> Result<String, McpError> {
    if locations.is_empty() {
        return Ok("No results found.".to_string());
    }

    let results: Result<Vec<String>, McpError> = locations
        .iter()
        .map(|loc| format_location(loc, context_lines))
        .collect();

    Ok(results?.join("\n\n---\n\n"))
}

/// Converts `GotoDefinitionResponse` to a list of locations.
#[allow(dead_code)]
fn goto_response_to_locations(response: GotoDefinitionResponse) -> Vec<lsp_types::Location> {
    match response {
        GotoDefinitionResponse::Scalar(loc) => vec![loc],
        GotoDefinitionResponse::Array(locs) => locs,
        GotoDefinitionResponse::Link(links) => links
            .into_iter()
            .map(|link| lsp_types::Location {
                uri: link.target_uri,
                range: link.target_range,
            })
            .collect(),
    }
}

/// Extracts markdown text from `MarkupContent` or string.
#[allow(dead_code)]
fn extract_hover_text(content: lsp_types::HoverContents) -> String {
    match content {
        lsp_types::HoverContents::Scalar(marked_string) => match marked_string {
            lsp_types::MarkedString::String(s) => s,
            lsp_types::MarkedString::LanguageString(ls) => {
                format!("```{}\n{}\n```", ls.language, ls.value)
            }
        },
        lsp_types::HoverContents::Array(marked_strings) => marked_strings
            .into_iter()
            .map(|ms| match ms {
                lsp_types::MarkedString::String(s) => s,
                lsp_types::MarkedString::LanguageString(ls) => {
                    format!("```{}\n{}\n```", ls.language, ls.value)
                }
            })
            .collect::<Vec<_>>()
            .join("\n\n"),
        lsp_types::HoverContents::Markup(markup) => markup.value,
    }
}

/// Formats document symbols recursively.
#[allow(dead_code)]
fn format_document_symbols(symbols: &[lsp_types::DocumentSymbol], indent: usize) -> String {
    let mut result = String::new();
    let indent_str = "  ".repeat(indent);

    for symbol in symbols {
        let kind = symbol_kind_to_string(symbol.kind);
        let (line, _) = from_lsp_position(symbol.selection_range.start);
        let _ = writeln!(
            result,
            "{}[{}] {} (line {})",
            indent_str, kind, symbol.name, line
        );

        if let Some(children) = &symbol.children {
            result.push_str(&format_document_symbols(children, indent + 1));
        }
    }

    result
}

/// Formats flat symbol information.
#[allow(dead_code)]
fn format_symbol_information(symbols: &[lsp_types::SymbolInformation]) -> String {
    let mut result = String::new();

    for symbol in symbols {
        let kind = symbol_kind_to_string(symbol.kind);
        let file_path = symbol.location.uri.to_file_path().map_or_else(
            |()| symbol.location.uri.to_string(),
            |p| p.display().to_string(),
        );
        let (line, _) = from_lsp_position(symbol.location.range.start);

        let container = symbol
            .container_name
            .as_ref()
            .map_or_else(Default::default, |c| format!(" (in {c})"));

        let _ = writeln!(
            result,
            "[{}] {}{} - {}:{}",
            kind, symbol.name, container, file_path, line
        );
    }

    result
}

/// Tool implementations for `KadabraRunes`.
#[tool_router]
impl KadabraRunes {
    /// Jump to the definition of a symbol at a given position or by name.
    #[tool(
        description = "Jump to where a symbol is defined. Essential for tracing imports and understanding implementations."
    )]
    async fn goto_definition(
        &self,
        Parameters(params): Parameters<GotoDefinitionParams>,
    ) -> Result<CallToolResult, McpError> {
        // Extract position from params
        let (file_path, line, column) = match &params.query {
            SymbolQuery::Position(pos) => (PathBuf::from(&pos.file_path), pos.line, pos.column),
            SymbolQuery::Name(SymbolNameParams { symbol, .. }) => {
                // For symbol name queries, we need to search first
                return Err(McpError::new(
                    ErrorCode::INVALID_PARAMS,
                    format!(
                        "Symbol name queries not yet implemented. Use workspace_symbols to find '{symbol}' first, then use position-based query."
                    ),
                    None,
                ));
            }
        };

        // Ensure the document is open
        self.lsp_client.did_open(&file_path).await.map_err(|e| {
            McpError::new(
                ErrorCode::INTERNAL_ERROR,
                format!("failed to open document: {e}"),
                None,
            )
        })?;

        // Call LSP client
        let response = self
            .lsp_client
            .goto_definition(&file_path, line, column)
            .await
            .map_err(|e| {
                McpError::new(
                    ErrorCode::INTERNAL_ERROR,
                    format!("goto_definition failed: {e}"),
                    None,
                )
            })?;

        // Convert response to locations
        let locations = goto_response_to_locations(response);

        // Format locations with context
        let formatted = format_locations(locations.as_slice(), 2)?;

        Ok(CallToolResult::success(vec![Content::text(formatted)]))
    }

    /// Find all references to a symbol in the workspace.
    #[tool(
        description = "Find all usages of a symbol. Reveals dependencies, call sites, and impact of changes."
    )]
    async fn find_references(
        &self,
        Parameters(params): Parameters<FindReferencesParams>,
    ) -> Result<CallToolResult, McpError> {
        // Extract position from params
        let (file_path, line, column) = match &params.query {
            SymbolQuery::Position(pos) => (PathBuf::from(&pos.file_path), pos.line, pos.column),
            SymbolQuery::Name(SymbolNameParams { symbol, .. }) => {
                return Err(McpError::new(
                    ErrorCode::INVALID_PARAMS,
                    format!(
                        "Symbol name queries not yet implemented. Use workspace_symbols to find '{symbol}' first, then use position-based query."
                    ),
                    None,
                ));
            }
        };

        // Ensure the document is open
        self.lsp_client.did_open(&file_path).await.map_err(|e| {
            McpError::new(
                ErrorCode::INTERNAL_ERROR,
                format!("failed to open document: {e}"),
                None,
            )
        })?;

        // Call LSP client
        let locations = self
            .lsp_client
            .find_references(&file_path, line, column, params.include_declaration)
            .await
            .map_err(|e| {
                McpError::new(
                    ErrorCode::INTERNAL_ERROR,
                    format!("find_references failed: {e}"),
                    None,
                )
            })?;

        // Format locations with context
        let formatted = format_locations(locations.as_slice(), 2)?;

        Ok(CallToolResult::success(vec![Content::text(formatted)]))
    }

    /// Get type information and documentation for a symbol.
    #[tool(
        description = "Get type signature and docs. Quick way to understand what something is without navigating away."
    )]
    async fn hover(
        &self,
        Parameters(params): Parameters<HoverParams>,
    ) -> Result<CallToolResult, McpError> {
        let file_path = PathBuf::from(&params.position.file_path);
        let line = params.position.line;
        let column = params.position.column;

        // Ensure the document is open
        self.lsp_client.did_open(&file_path).await.map_err(|e| {
            McpError::new(
                ErrorCode::INTERNAL_ERROR,
                format!("failed to open document: {e}"),
                None,
            )
        })?;

        // Call LSP client
        let hover_result = self
            .lsp_client
            .hover(&file_path, line, column)
            .await
            .map_err(|e| {
                McpError::new(
                    ErrorCode::INTERNAL_ERROR,
                    format!("hover failed: {e}"),
                    None,
                )
            })?;

        // Format hover information
        let formatted = match hover_result {
            Some(hover) => {
                let text = extract_hover_text(hover.contents);
                if text.is_empty() {
                    "No hover information available.".to_string()
                } else {
                    text
                }
            }
            None => "No hover information available.".to_string(),
        };

        Ok(CallToolResult::success(vec![Content::text(formatted)]))
    }

    /// List all symbols defined in a file.
    #[tool(
        description = "List all symbols in a file. Get a structural overview: functions, types, constants, etc."
    )]
    async fn document_symbols(
        &self,
        Parameters(params): Parameters<DocumentSymbolsParams>,
    ) -> Result<CallToolResult, McpError> {
        let file_path = PathBuf::from(&params.file_path);

        // Ensure the document is open
        self.lsp_client.did_open(&file_path).await.map_err(|e| {
            McpError::new(
                ErrorCode::INTERNAL_ERROR,
                format!("failed to open document: {e}"),
                None,
            )
        })?;

        // Call LSP client
        let response = self
            .lsp_client
            .document_symbols(&file_path)
            .await
            .map_err(|e| {
                McpError::new(
                    ErrorCode::INTERNAL_ERROR,
                    format!("document_symbols failed: {e}"),
                    None,
                )
            })?;

        // Format symbols
        let formatted = match response {
            DocumentSymbolResponse::Flat(symbols) => {
                if symbols.is_empty() {
                    "No symbols found in document.".to_string()
                } else {
                    format_symbol_information(&symbols)
                }
            }
            DocumentSymbolResponse::Nested(symbols) => {
                if symbols.is_empty() {
                    "No symbols found in document.".to_string()
                } else {
                    format_document_symbols(&symbols, 0)
                }
            }
        };

        Ok(CallToolResult::success(vec![Content::text(formatted)]))
    }

    /// Search for symbols across the entire workspace.
    #[tool(
        description = "Search symbols by name across the workspace. Find types, functions, or modules without knowing their location."
    )]
    async fn workspace_symbols(
        &self,
        Parameters(params): Parameters<WorkspaceSymbolsParams>,
    ) -> Result<CallToolResult, McpError> {
        // Call LSP client
        let symbols = self
            .lsp_client
            .workspace_symbols(&params.query)
            .await
            .map_err(|e| {
                McpError::new(
                    ErrorCode::INTERNAL_ERROR,
                    format!("workspace_symbols failed: {e}"),
                    None,
                )
            })?;

        // Limit results if specified
        let limited_symbols: Vec<_> = symbols
            .into_iter()
            .take(params.max_results as usize)
            .collect();

        // Format symbols
        let formatted = if limited_symbols.is_empty() {
            format!("No symbols found matching '{}'.", params.query)
        } else {
            format_symbol_information(&limited_symbols)
        };

        Ok(CallToolResult::success(vec![Content::text(formatted)]))
    }

    /// Find all functions that call the function at the given position.
    #[tool(
        description = "Find callers of a function. Build upward call graphs, trace who depends on this code."
    )]
    async fn incoming_calls(
        &self,
        Parameters(params): Parameters<IncomingCallsParams>,
    ) -> Result<CallToolResult, McpError> {
        let file_path = PathBuf::from(&params.position.file_path);
        let line = params.position.line;
        let column = params.position.column;

        // Ensure the document is open
        self.lsp_client.did_open(&file_path).await.map_err(|e| {
            McpError::new(
                ErrorCode::INTERNAL_ERROR,
                format!("failed to open document: {e}"),
                None,
            )
        })?;

        // Call LSP client
        let calls = self
            .lsp_client
            .incoming_calls(&file_path, line, column)
            .await
            .map_err(|e| {
                McpError::new(
                    ErrorCode::INTERNAL_ERROR,
                    format!("incoming_calls failed: {e}"),
                    None,
                )
            })?;

        // Format call hierarchy
        let mut formatted = String::new();
        if calls.is_empty() {
            formatted = "No incoming calls found.".to_string();
        } else {
            for call in calls {
                let caller_name = &call.from.name;
                let kind = symbol_kind_to_string(call.from.kind);
                let file_path = call
                    .from
                    .uri
                    .to_file_path()
                    .map_or_else(|()| call.from.uri.to_string(), |p| p.display().to_string());
                let (line, _) = from_lsp_position(call.from.selection_range.start);

                let _ = writeln!(formatted, "\n[{kind}] {caller_name} - {file_path}:{line}");

                // List call sites
                for range in &call.from_ranges {
                    let (call_line, call_col) = from_lsp_position(range.start);
                    let _ = writeln!(
                        formatted,
                        "  Call site: line {call_line}, column {call_col}"
                    );
                }
            }
        }

        Ok(CallToolResult::success(vec![Content::text(formatted)]))
    }

    /// Find all functions called by the function at the given position.
    #[tool(
        description = "Find callees of a function. Build downward call graphs, trace execution flow."
    )]
    async fn outgoing_calls(
        &self,
        Parameters(params): Parameters<OutgoingCallsParams>,
    ) -> Result<CallToolResult, McpError> {
        let file_path = PathBuf::from(&params.position.file_path);
        let line = params.position.line;
        let column = params.position.column;

        // Ensure the document is open
        self.lsp_client.did_open(&file_path).await.map_err(|e| {
            McpError::new(
                ErrorCode::INTERNAL_ERROR,
                format!("failed to open document: {e}"),
                None,
            )
        })?;

        // Call LSP client
        let calls = self
            .lsp_client
            .outgoing_calls(&file_path, line, column)
            .await
            .map_err(|e| {
                McpError::new(
                    ErrorCode::INTERNAL_ERROR,
                    format!("outgoing_calls failed: {e}"),
                    None,
                )
            })?;

        // Format call hierarchy
        let mut formatted = String::new();
        if calls.is_empty() {
            formatted = "No outgoing calls found.".to_string();
        } else {
            for call in calls {
                let callee_name = &call.to.name;
                let kind = symbol_kind_to_string(call.to.kind);
                let file_path = call
                    .to
                    .uri
                    .to_file_path()
                    .map_or_else(|()| call.to.uri.to_string(), |p| p.display().to_string());
                let (line, _) = from_lsp_position(call.to.selection_range.start);
                let _ = writeln!(formatted, "\n[{kind}] {callee_name} - {file_path}:{line}");

                // List call sites
                for range in &call.from_ranges {
                    let (call_line, call_col) = from_lsp_position(range.start);
                    let _ = writeln!(
                        formatted,
                        "  Call site: line {call_line}, column {call_col}"
                    );
                }
            }
        }

        Ok(CallToolResult::success(vec![Content::text(formatted)]))
    }

    /// Find all implementations of a trait or interface.
    #[tool(
        description = "Find trait/interface implementations. Discover concrete types, understand polymorphism."
    )]
    async fn implementations(
        &self,
        Parameters(params): Parameters<ImplementationsParams>,
    ) -> Result<CallToolResult, McpError> {
        // Extract position from params
        let (file_path, line, column) = match &params.query {
            SymbolQuery::Position(pos) => (PathBuf::from(&pos.file_path), pos.line, pos.column),
            SymbolQuery::Name(SymbolNameParams { symbol, .. }) => {
                return Err(McpError::new(
                    ErrorCode::INVALID_PARAMS,
                    format!(
                        "Symbol name queries not yet implemented. Use workspace_symbols to find '{symbol}' first, then use position-based query."
                    ),
                    None,
                ));
            }
        };

        // Ensure the document is open
        self.lsp_client.did_open(&file_path).await.map_err(|e| {
            McpError::new(
                ErrorCode::INTERNAL_ERROR,
                format!("failed to open document: {e}"),
                None,
            )
        })?;

        // Call LSP client
        let response = self
            .lsp_client
            .implementations(&file_path, line, column)
            .await
            .map_err(|e| {
                McpError::new(
                    ErrorCode::INTERNAL_ERROR,
                    format!("implementations failed: {e}"),
                    None,
                )
            })?;

        // Convert response to locations
        let locations = goto_response_to_locations(response);

        // Format locations with context
        let formatted = format_locations(locations.as_slice(), 2)?;

        Ok(CallToolResult::success(vec![Content::text(formatted)]))
    }

    /// Jump to the type definition of a symbol.
    #[tool(
        description = "Jump to a symbol's type definition. Understand what type a variable or expression has."
    )]
    async fn type_definition(
        &self,
        Parameters(params): Parameters<TypeDefinitionParams>,
    ) -> Result<CallToolResult, McpError> {
        let file_path = PathBuf::from(&params.position.file_path);
        let line = params.position.line;
        let column = params.position.column;

        // Ensure the document is open
        self.lsp_client.did_open(&file_path).await.map_err(|e| {
            McpError::new(
                ErrorCode::INTERNAL_ERROR,
                format!("failed to open document: {e}"),
                None,
            )
        })?;

        // Call LSP client
        let response = self
            .lsp_client
            .type_definition(&file_path, line, column)
            .await
            .map_err(|e| {
                McpError::new(
                    ErrorCode::INTERNAL_ERROR,
                    format!("type_definition failed: {e}"),
                    None,
                )
            })?;

        // Convert response to locations
        let locations = goto_response_to_locations(response);

        // Format locations with context
        let formatted = format_locations(locations.as_slice(), 2)?;

        Ok(CallToolResult::success(vec![Content::text(formatted)]))
    }
}

#[tool_handler]
impl ServerHandler for KadabraRunes {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::LATEST,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "kadabra-runes".into(),
                version: env!("CARGO_PKG_VERSION").into(),
                ..Default::default()
            },
            instructions: Some(
                "Semantic code intelligence via LSP. Enables: reverse engineering unfamiliar code, \
                 building call graphs, tracing dependencies, understanding type hierarchies, \
                 and exploring how code flows through a system. Works with any LSP-compatible \
                 language server (rust-analyzer, typescript-language-server, etc.)."
                    .into(),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    // Tests require a real LSP client instance, which is complex to mock.
    // Integration tests will be added separately.

    #[test]
    fn test_helper_functions() {
        // Test read_context_lines would require creating a test file
        // Test format functions would require creating test LSP responses
        // These will be covered in integration tests
    }
}
