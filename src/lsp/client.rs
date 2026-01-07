//! LSP client implementation.
//!
//! This module provides the core LSP client that spawns and communicates
//! with language servers using the Language Server Protocol.
//!
//! Note: Methods appear unused because they're called by MCP tool implementations.
//!
//! # Example

// Allow dead code warnings - methods are used by MCP server tools
#![allow(dead_code)]
//!
//! ```ignore
//! use code_navigator::lsp::client::LspClient;
//! use std::path::Path;
//!
//! let client = LspClient::builder()
//!     .server_command("rust-analyzer")
//!     .workspace_root(Path::new("/path/to/project"))
//!     .build()
//!     .await?;
//!
//! let definition = client.goto_definition("src/main.rs", 10, 5).await?;
//! client.shutdown().await?;
//! ```

use std::collections::HashSet;
use std::ops::ControlFlow;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;

use async_lsp::concurrency::ConcurrencyLayer;
use async_lsp::panic::CatchUnwindLayer;
use async_lsp::router::Router;
use async_lsp::tracing::TracingLayer;
use async_lsp::{LanguageServer, ServerSocket};
use lsp_types::{
    CallHierarchyIncomingCall, CallHierarchyIncomingCallsParams, CallHierarchyItem,
    CallHierarchyOutgoingCall, CallHierarchyOutgoingCallsParams, CallHierarchyPrepareParams,
    ClientCapabilities, ClientInfo, CompletionClientCapabilities, CompletionItemCapability,
    DidChangeTextDocumentParams, DidChangeWatchedFilesClientCapabilities,
    DidCloseTextDocumentParams, DidOpenTextDocumentParams, DocumentSymbolClientCapabilities,
    DocumentSymbolParams, DocumentSymbolResponse, DynamicRegistrationClientCapabilities,
    GotoCapability, GotoDefinitionParams, GotoDefinitionResponse, Hover, HoverClientCapabilities,
    HoverParams, InitializeParams, InitializedParams, Location, MarkupKind, PartialResultParams,
    ReferenceContext, ReferenceParams, ServerCapabilities, SymbolInformation,
    TextDocumentClientCapabilities, TextDocumentContentChangeEvent, TextDocumentIdentifier,
    TextDocumentItem, TextDocumentPositionParams, TextDocumentSyncClientCapabilities, TraceValue,
    Url, VersionedTextDocumentIdentifier, WindowClientCapabilities, WorkDoneProgressParams,
    WorkspaceClientCapabilities, WorkspaceEditClientCapabilities, WorkspaceFolder,
    WorkspaceSymbolClientCapabilities, WorkspaceSymbolParams, WorkspaceSymbolResponse,
    notification, request,
};
use tokio::sync::Mutex;
use tower::ServiceBuilder;

use crate::error::LspError;

use super::LspResult;
use super::types::{path_to_url, to_lsp_position};

/// State for handling LSP client notifications.
///
/// This struct maintains the state needed to handle notifications
/// from the language server.
#[derive(Debug, Clone)]
struct ClientState {
    // Track diagnostics, progress, etc. if needed in the future
}

impl ClientState {
    fn new() -> Self {
        Self {}
    }
}

/// Configuration for building an LSP client.
#[derive(Debug, Clone)]
pub struct LspClientConfig {
    /// Command to start the language server.
    pub server_command: String,
    /// Arguments to pass to the language server.
    pub server_args: Vec<String>,
    /// Root directory of the workspace.
    pub workspace_root: PathBuf,
    /// Timeout for initialization.
    pub init_timeout: std::time::Duration,
    /// Timeout for requests.
    pub request_timeout: std::time::Duration,
}

impl Default for LspClientConfig {
    fn default() -> Self {
        Self {
            server_command: "rust-analyzer".to_string(),
            server_args: Vec::new(),
            workspace_root: PathBuf::from("."),
            init_timeout: std::time::Duration::from_secs(30),
            request_timeout: std::time::Duration::from_secs(10),
        }
    }
}

/// Builder for constructing an LSP client.
#[derive(Debug, Default)]
pub struct LspClientBuilder {
    config: LspClientConfig,
}

impl LspClientBuilder {
    /// Creates a new builder with default configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the server command.
    #[must_use]
    pub fn server_command(mut self, command: impl Into<String>) -> Self {
        self.config.server_command = command.into();
        self
    }

    /// Sets the server arguments.
    #[must_use]
    pub fn server_args(mut self, args: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.config.server_args = args.into_iter().map(Into::into).collect();
        self
    }

    /// Sets the workspace root.
    #[must_use]
    pub fn workspace_root(mut self, path: impl Into<PathBuf>) -> Self {
        self.config.workspace_root = path.into();
        self
    }

    /// Sets the initialization timeout.
    #[must_use]
    pub fn init_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.config.init_timeout = timeout;
        self
    }

    /// Sets the request timeout.
    #[must_use]
    pub fn request_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.config.request_timeout = timeout;
        self
    }

    /// Builds the LSP client.
    ///
    /// This will spawn the language server process and perform initialization.
    /// ## Errors
    #[allow(clippy::too_many_lines)]
    pub async fn build(self) -> LspResult<LspClient> {
        let workspace_root = self.config.workspace_root.clone();
        let workspace_root = workspace_root.canonicalize().map_err(|e| {
            LspError::InitializationFailed(format!("failed to canonicalize workspace root: {e}"))
        })?;

        // Spawn the language server process
        let mut cmd = async_process::Command::new(&self.config.server_command);
        cmd.args(&self.config.server_args)
            .current_dir(&workspace_root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .kill_on_drop(true);

        let mut child = cmd.spawn().map_err(|e| {
            LspError::ServerStartFailed(format!(
                "failed to spawn '{}': {}",
                self.config.server_command, e
            ))
        })?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| LspError::ServerStartFailed("failed to capture stdout".to_string()))?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| LspError::ServerStartFailed("failed to capture stdin".to_string()))?;

        // Create the mainloop with router for notifications
        let (mainloop, server) = async_lsp::MainLoop::new_client(|_client| {
            let mut router = Router::new(ClientState::new());

            // Handle progress notifications
            router.notification::<notification::Progress>(|_this, _prog| {
                // Can log or track progress here if needed
                ControlFlow::Continue(())
            });

            // Handle publish diagnostics notifications
            router.notification::<notification::PublishDiagnostics>(|_this, _diag| {
                // Can collect diagnostics here if needed
                ControlFlow::Continue(())
            });

            // Build the service with layers
            ServiceBuilder::new()
                .layer(TracingLayer::default())
                .layer(CatchUnwindLayer::default())
                .layer(ConcurrencyLayer::default())
                .service(router)
        });

        // Spawn the mainloop to handle communication
        let mainloop_handle = tokio::spawn(async move {
            mainloop.run_buffered(stdout, stdin).await.ok();
        });

        // Prepare initialization parameters
        let workspace_uri = Url::from_file_path(&workspace_root).map_err(|()| {
            LspError::InitializationFailed(format!(
                "invalid workspace root path: {}",
                workspace_root.display()
            ))
        })?;

        let init_params = InitializeParams {
            process_id: Some(std::process::id()),
            workspace_folders: Some(vec![WorkspaceFolder {
                uri: workspace_uri,
                name: workspace_root
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(".runes")
                    .to_string(),
            }]),
            initialization_options: None,
            capabilities: ClientCapabilities {
                workspace: Some(WorkspaceClientCapabilities {
                    apply_edit: Some(true),
                    workspace_edit: Some(WorkspaceEditClientCapabilities {
                        document_changes: Some(true),
                        ..Default::default()
                    }),
                    did_change_configuration: Some(DynamicRegistrationClientCapabilities {
                        dynamic_registration: Some(false),
                    }),
                    did_change_watched_files: Some(DidChangeWatchedFilesClientCapabilities {
                        dynamic_registration: Some(false),
                        relative_pattern_support: None,
                    }),
                    symbol: Some(WorkspaceSymbolClientCapabilities {
                        dynamic_registration: Some(false),
                        ..Default::default()
                    }),
                    execute_command: Some(DynamicRegistrationClientCapabilities {
                        dynamic_registration: Some(false),
                    }),
                    ..Default::default()
                }),
                text_document: Some(TextDocumentClientCapabilities {
                    synchronization: Some(TextDocumentSyncClientCapabilities {
                        dynamic_registration: Some(false),
                        will_save: Some(false),
                        will_save_wait_until: Some(false),
                        did_save: Some(false),
                    }),
                    completion: Some(CompletionClientCapabilities {
                        dynamic_registration: Some(false),
                        completion_item: Some(CompletionItemCapability {
                            snippet_support: Some(false),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }),
                    hover: Some(HoverClientCapabilities {
                        dynamic_registration: Some(false),
                        content_format: Some(vec![MarkupKind::Markdown, MarkupKind::PlainText]),
                    }),
                    definition: Some(GotoCapability {
                        dynamic_registration: Some(false),
                        link_support: Some(false),
                    }),
                    references: Some(DynamicRegistrationClientCapabilities {
                        dynamic_registration: Some(false),
                    }),
                    document_symbol: Some(DocumentSymbolClientCapabilities {
                        dynamic_registration: Some(false),
                        hierarchical_document_symbol_support: Some(true),
                        ..Default::default()
                    }),
                    type_definition: Some(GotoCapability {
                        dynamic_registration: Some(false),
                        link_support: Some(false),
                    }),
                    implementation: Some(GotoCapability {
                        dynamic_registration: Some(false),
                        link_support: Some(false),
                    }),
                    call_hierarchy: Some(DynamicRegistrationClientCapabilities {
                        dynamic_registration: Some(false),
                    }),
                    ..Default::default()
                }),
                window: Some(WindowClientCapabilities {
                    work_done_progress: Some(true),
                    ..Default::default()
                }),
                experimental: Some(true.into()),
                ..Default::default()
            },
            trace: Some(TraceValue::Off),
            client_info: Some(ClientInfo {
                name: "kadabra-runes".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
            locale: None,
            work_done_progress_params: WorkDoneProgressParams::default(),
            ..Default::default()
        };

        // Wrap server in Arc<Mutex<>> for shared mutable access
        let server = Arc::new(Mutex::new(server));

        // Send initialize request
        let init_result = tokio::time::timeout(
            self.config.init_timeout,
            server.lock().await.initialize(init_params),
        )
        .await
        .map_err(|_| LspError::Timeout(self.config.init_timeout))?
        .map_err(|e| LspError::InitializationFailed(format!("initialize request failed: {e:?}")))?;

        let capabilities = Arc::new(init_result.capabilities);

        // Send initialized notification
        server
            .lock()
            .await
            .initialized(InitializedParams {})
            .map_err(|e| {
                LspError::InitializationFailed(format!("initialized notification failed: {e:?}"))
            })?;

        Ok(LspClient {
            config: self.config,
            server,
            _mainloop_handle: mainloop_handle,
            capabilities,
            open_documents: Arc::new(Mutex::new(HashSet::new())),
            _child_process: Arc::new(Mutex::new(child)),
        })
    }
}

/// LSP client for communicating with language servers.
///
/// This client manages the lifecycle of a language server process and
/// provides methods for all LSP operations needed by the MCP tools.
#[derive(Debug)]
pub struct LspClient {
    /// Configuration used to create this client.
    config: LspClientConfig,
    /// The language server handle for making requests.
    server: Arc<Mutex<ServerSocket>>,
    /// Handle to the mainloop task.
    _mainloop_handle: tokio::task::JoinHandle<()>,
    /// Server capabilities from initialization.
    capabilities: Arc<ServerCapabilities>,
    /// Set of currently open documents.
    open_documents: Arc<Mutex<HashSet<Url>>>,
    /// The language server process handle (kept alive to prevent kill-on-drop).
    _child_process: Arc<Mutex<async_process::Child>>,
}

// impl std::fmt::Debug for LspClient {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("LspClient")
//             .field("config", &self.config)
//             .field("capabilities", &self.capabilities)
//             .finish()
//     }
// }

impl LspClient {
    /// Creates a new builder for constructing an LSP client.
    pub fn builder() -> LspClientBuilder {
        LspClientBuilder::new()
    }

    /// Shuts down the language server gracefully.
    /// ## Errors
    pub async fn shutdown(&self) -> LspResult<()> {
        // Send shutdown request
        self.server
            .lock()
            .await
            .shutdown(())
            .await
            .map_err(|e| LspError::RequestFailed(format!("shutdown request failed: {e:?}")))?;

        // Send exit notification
        self.server
            .lock()
            .await
            .exit(())
            .map_err(|e| LspError::RequestFailed(format!("exit notification failed: {e:?}")))?;

        Ok(())
    }

    /// Opens a document in the language server.
    ///
    /// This sends a `textDocument/didOpen` notification and tracks the document as open.
    /// ## Errors
    pub async fn did_open(&self, path: &Path) -> LspResult<()> {
        let uri = path_to_url(path)?;

        // Read file content
        let content = tokio::fs::read_to_string(path).await.map_err(|e| {
            LspError::DocumentNotFound(format!("failed to read '{}': {}", path.display(), e))
        })?;

        // Determine language ID from file extension
        let language_id = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map_or_else(
                || "plaintext",
                |ext| match ext {
                    "rs" => "rust",
                    "py" => "python",
                    "js" => "javascript",
                    "ts" => "typescript",
                    "go" => "go",
                    "c" => "c",
                    "cpp" | "cc" | "cxx" => "cpp",
                    "java" => "java",
                    _ => "plaintext",
                },
            )
            .to_string();

        let params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id,
                version: 0,
                text: content,
            },
        };

        self.server
            .lock()
            .await
            .did_open(params)
            .map_err(|e| LspError::RequestFailed(format!("didOpen notification failed: {e:?}")))?;

        // Track as open
        self.open_documents.lock().await.insert(uri);

        Ok(())
    }

    /// Notifies the language server about document changes.
    /// ## Errors
    pub async fn did_change(&self, path: &Path, content: &str) -> LspResult<()> {
        let uri = path_to_url(path)?;

        // Check if document is open
        if !self.open_documents.lock().await.contains(&uri) {
            return Err(LspError::DocumentNotFound(format!(
                "document not open: {}",
                path.display()
            )));
        }

        let params = DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier {
                uri,
                version: 1, // Simplified versioning
            },
            content_changes: vec![TextDocumentContentChangeEvent {
                range: None,
                range_length: None,
                text: content.to_string(),
            }],
        };

        self.server.lock().await.did_change(params).map_err(|e| {
            LspError::RequestFailed(format!("didChange notification failed: {e:?}"))
        })?;

        Ok(())
    }

    /// Closes a document in the language server.
    /// ## Errors
    pub async fn did_close(&self, path: &Path) -> LspResult<()> {
        let uri = path_to_url(path)?;

        let params = DidCloseTextDocumentParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
        };

        self.server
            .lock()
            .await
            .did_close(params)
            .map_err(|e| LspError::RequestFailed(format!("didClose notification failed: {e:?}")))?;

        // Remove from tracking
        self.open_documents.lock().await.remove(&uri);

        Ok(())
    }

    // Navigation methods

    /// Gets the definition location(s) for the symbol at the given position.
    /// ## Errors
    pub async fn goto_definition(
        &self,
        path: &Path,
        line: u32,
        column: u32,
    ) -> LspResult<GotoDefinitionResponse> {
        let uri = path_to_url(path)?;
        let position = to_lsp_position(line, column)?;

        let params = GotoDefinitionParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri },
                position,
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        };

        let result = tokio::time::timeout(
            self.config.request_timeout,
            self.server.lock().await.definition(params),
        )
        .await
        .map_err(|_| LspError::Timeout(self.config.request_timeout))?
        .map_err(|e| LspError::RequestFailed(format!("goto_definition failed: {e:?}")))?;

        Ok(result.unwrap_or(GotoDefinitionResponse::Array(vec![])))
    }

    /// Finds all references to the symbol at the given position.
    /// ## Errors
    pub async fn find_references(
        &self,
        path: &Path,
        line: u32,
        column: u32,
        include_declaration: bool,
    ) -> LspResult<Vec<Location>> {
        let uri = path_to_url(path)?;
        let position = to_lsp_position(line, column)?;

        let params = ReferenceParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri },
                position,
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
            context: ReferenceContext {
                include_declaration,
            },
        };

        let result = tokio::time::timeout(
            self.config.request_timeout,
            self.server.lock().await.references(params),
        )
        .await
        .map_err(|_| LspError::Timeout(self.config.request_timeout))?
        .map_err(|e| LspError::RequestFailed(format!("references failed: {e:?}")))?;

        Ok(result.unwrap_or_default())
    }

    /// Gets hover information for the symbol at the given position.
    /// ## Errors
    pub async fn hover(&self, path: &Path, line: u32, column: u32) -> LspResult<Option<Hover>> {
        let uri = path_to_url(path)?;
        let position = to_lsp_position(line, column)?;

        let params = HoverParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri },
                position,
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
        };

        let result = tokio::time::timeout(
            self.config.request_timeout,
            self.server.lock().await.hover(params),
        )
        .await
        .map_err(|_| LspError::Timeout(self.config.request_timeout))?
        .map_err(|e| LspError::RequestFailed(format!("hover failed: {e:?}")))?;

        Ok(result)
    }

    /// Gets all symbols in a document.
    /// ## Errors
    pub async fn document_symbols(&self, path: &Path) -> LspResult<DocumentSymbolResponse> {
        let uri = path_to_url(path)?;

        let params = DocumentSymbolParams {
            text_document: TextDocumentIdentifier { uri },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        };

        let result = tokio::time::timeout(
            self.config.request_timeout,
            self.server.lock().await.document_symbol(params),
        )
        .await
        .map_err(|_| LspError::Timeout(self.config.request_timeout))?
        .map_err(|e| LspError::RequestFailed(format!("document_symbol failed: {e:?}")))?;

        Ok(result.unwrap_or(DocumentSymbolResponse::Flat(vec![])))
    }

    /// Searches for symbols across the workspace.
    /// ## Errors
    pub async fn workspace_symbols(&self, query: &str) -> LspResult<Vec<SymbolInformation>> {
        let params = WorkspaceSymbolParams {
            query: query.to_string(),
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        };

        let result = tokio::time::timeout(
            self.config.request_timeout,
            self.server.lock().await.symbol(params),
        )
        .await
        .map_err(|_| LspError::Timeout(self.config.request_timeout))?
        .map_err(|e| LspError::RequestFailed(format!("symbol failed: {e:?}")))?;

        // Convert WorkspaceSymbolResponse to Vec<SymbolInformation>
        match result {
            Some(WorkspaceSymbolResponse::Flat(symbols)) => Ok(symbols),
            Some(WorkspaceSymbolResponse::Nested(_nested)) => {
                // For nested symbols, we'd need to flatten them
                // For now, return empty
                Ok(vec![])
            }
            None => Ok(vec![]),
        }
    }

    /// Gets incoming calls (callers) for the function at the given position.
    /// ## Errors
    pub async fn incoming_calls(
        &self,
        path: &Path,
        line: u32,
        column: u32,
    ) -> LspResult<Vec<CallHierarchyIncomingCall>> {
        // First, prepare call hierarchy
        let items = self.prepare_call_hierarchy(path, line, column).await?;

        if items.is_empty() {
            return Ok(vec![]);
        }

        // Use the first item for incoming calls
        let params = CallHierarchyIncomingCallsParams {
            item: items[0].clone(),
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        };

        let result = tokio::time::timeout(
            self.config.request_timeout,
            self.server.lock().await.incoming_calls(params),
        )
        .await
        .map_err(|_| LspError::Timeout(self.config.request_timeout))?
        .map_err(|e| LspError::RequestFailed(format!("incoming_calls failed: {e:?}")))?;

        Ok(result.unwrap_or_default())
    }

    /// Gets outgoing calls (callees) for the function at the given position.
    /// ## Errors
    pub async fn outgoing_calls(
        &self,
        path: &Path,
        line: u32,
        column: u32,
    ) -> LspResult<Vec<CallHierarchyOutgoingCall>> {
        // First, prepare call hierarchy
        let items = self.prepare_call_hierarchy(path, line, column).await?;

        if items.is_empty() {
            return Ok(vec![]);
        }

        // Use the first item for outgoing calls
        let params = CallHierarchyOutgoingCallsParams {
            item: items[0].clone(),
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        };

        let result = tokio::time::timeout(
            self.config.request_timeout,
            self.server.lock().await.outgoing_calls(params),
        )
        .await
        .map_err(|_| LspError::Timeout(self.config.request_timeout))?
        .map_err(|e| LspError::RequestFailed(format!("outgoing_calls failed: {e:?}")))?;

        Ok(result.unwrap_or_default())
    }

    /// Prepares call hierarchy items for the given position.
    async fn prepare_call_hierarchy(
        &self,
        path: &Path,
        line: u32,
        column: u32,
    ) -> LspResult<Vec<CallHierarchyItem>> {
        let uri = path_to_url(path)?;
        let position = to_lsp_position(line, column)?;

        let params = CallHierarchyPrepareParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri },
                position,
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
        };

        let result = tokio::time::timeout(
            self.config.request_timeout,
            self.server.lock().await.prepare_call_hierarchy(params),
        )
        .await
        .map_err(|_| LspError::Timeout(self.config.request_timeout))?
        .map_err(|e| LspError::RequestFailed(format!("prepare_call_hierarchy failed: {e:?}")))?;

        Ok(result.unwrap_or_default())
    }

    /// Gets implementations for the trait/interface at the given position.
    /// ## Errors
    pub async fn implementations(
        &self,
        path: &Path,
        line: u32,
        column: u32,
    ) -> LspResult<GotoDefinitionResponse> {
        let uri = path_to_url(path)?;
        let position = to_lsp_position(line, column)?;

        let params = request::GotoImplementationParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri },
                position,
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        };

        let result = tokio::time::timeout(
            self.config.request_timeout,
            self.server.lock().await.implementation(params),
        )
        .await
        .map_err(|_| LspError::Timeout(self.config.request_timeout))?
        .map_err(|e| LspError::RequestFailed(format!("goto_implementation failed: {e:?}")))?;

        Ok(result.unwrap_or(GotoDefinitionResponse::Array(vec![])))
    }

    /// Gets the type definition for the symbol at the given position.
    /// ## Errors
    pub async fn type_definition(
        &self,
        path: &Path,
        line: u32,
        column: u32,
    ) -> LspResult<GotoDefinitionResponse> {
        let uri = path_to_url(path)?;
        let position = to_lsp_position(line, column)?;

        let params = request::GotoTypeDefinitionParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri },
                position,
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        };

        let result = tokio::time::timeout(
            self.config.request_timeout,
            self.server.lock().await.type_definition(params),
        )
        .await
        .map_err(|_| LspError::Timeout(self.config.request_timeout))?
        .map_err(|e| LspError::RequestFailed(format!("goto_type_definition failed: {e:?}")))?;

        Ok(result.unwrap_or(GotoDefinitionResponse::Array(vec![])))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_default() {
        let builder = LspClientBuilder::new();
        assert_eq!(builder.config.server_command, "rust-analyzer");
    }

    #[test]
    fn test_builder_configuration() {
        let builder = LspClientBuilder::new()
            .server_command("pylsp")
            .server_args(["--verbose"])
            .workspace_root("/home/user/project")
            .init_timeout(std::time::Duration::from_secs(60));

        assert_eq!(builder.config.server_command, "pylsp");
        assert_eq!(builder.config.server_args, vec!["--verbose"]);
        assert_eq!(
            builder.config.workspace_root,
            PathBuf::from("/home/user/project")
        );
        assert_eq!(
            builder.config.init_timeout,
            std::time::Duration::from_secs(60)
        );
    }
}
