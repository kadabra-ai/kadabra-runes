# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Kadabra Runes is an MCP (Model Context Protocol) server that bridges LLM applications with language servers (like rust-analyzer) to enable semantic code navigation. It provides 9 LSP-powered tools for intelligent code understanding: `goto_definition`, `find_references`, `hover`, `document_symbols`, `workspace_symbols`, `incoming_calls`, `outgoing_calls`, `implementations`, and `type_definition`.

## Build and Development Commands

### Building
```bash
# Debug build
cargo build

# Release build with optimizations
cargo build --release

# Check without building
cargo check

# Install locally to ~/.cargo/bin
cargo install --path .
```

### Testing

Tests are organized into multiple test files with shared utilities. The `serial_test` crate automatically serializes tests that need it:

```bash
# Run all tests (unit + integration)
cargo test

# Run LSP client tests
cargo test --test lsp_client_test

# Run MCP tool tests
cargo test --test mcp_tool_test

# Run specific test
cargo test --test lsp_client_test test_goto_definition

# With debug logging
RUST_LOG=debug cargo test --test lsp_client_test -- --nocapture
```

### Running the Server

```bash
# Run for current directory (default serve command)
cargo run

# Run for specific workspace
cargo run -- --workspace /path/to/rust/project

# With debug logging
cargo run -- --workspace . --log-level debug

# Using installed binary
kadabra-runes --workspace /path/to/project

# Create .mcp.json configuration in current directory
cargo run -- config
# Or with installed binary:
kadabra-runes config
```

## Architecture

### High-Level Design

The system has three main layers that communicate via stdio/JSON-RPC:

```
LLM Client (Claude Code) ←→ MCP Server (kadabra-runes) ←→ LSP Client ←→ Language Server (rust-analyzer)
         (MCP protocol)              (bridges MCP to LSP)        (LSP protocol)
```

### Module Structure

**`src/main.rs`**: CLI entry point - parses args, initializes logging to stderr (stdout is reserved for MCP), sets up LSP client, starts MCP server with stdio transport.

**`src/lib.rs`**: Library root that re-exports the main modules.

**`src/error.rs`**: Centralized error types (`LspError`, `McpError`) using `thiserror`.

**`src/config.rs`**: Configuration helper to create/update `.mcp.json` files for MCP client setup.

**`src/lsp/` (LSP Client Layer)**:
- `client.rs`: Manages language server lifecycle - spawns process, handles initialization handshake, sends requests, correlates responses
- `types.rs`: Helper types and conversions for LSP operations

**`src/mcp/` (MCP Server Layer)**:
- `server.rs`: Implements MCP server using `rmcp` crate - exposes tools, routes calls to LSP client, formats responses for LLM consumption
- `tools.rs`: Defines tool parameter types and response schemas using `schemars` for JSON Schema generation
- `transport.rs`: Abstractions for stdio-based JSON-RPC communication

### Key Dependencies

- **`rmcp`**: Official Rust MCP SDK - provides server framework, stdio transport, JSON Schema macros
- **`async-lsp`**: LSP client/server framework - chosen over `tower-lsp` because it supports client role
- **`lsp-types`**: LSP protocol type definitions - version MUST match `async-lsp` to avoid type conflicts
- **`tokio`**: Async runtime with full features
- **`async-process`**: Process management for spawning language servers
- **`clap`**: Command-line argument parsing with derive macros
- **`schemars`**: JSON Schema generation for MCP tool parameters

### Critical Design Decisions

1. **Stdio Transport**: Both MCP (client↔server) and LSP (server↔language server) use stdio, so logs MUST go to stderr
2. **Response Formatting**: Responses are formatted for LLM consumption with context (source code snippets, line numbers) rather than raw LSP JSON
3. **Single Workspace**: One language server instance per MCP server - multiple workspaces require multiple server instances
4. **Async Throughout**: Fully async design using tokio for concurrent request handling
5. **1-Indexed Positions**: LSP uses 0-indexed positions internally, but tools accept 1-indexed (human-readable) positions

### Testing Architecture

Tests are split across multiple files with shared utilities:

**`tests/common/`** - Shared test infrastructure:
- `mod.rs`: Common helpers for fixtures, rust-analyzer detection, LSP client setup
- `lsp_harness.rs`: LSP client spawning with CI-aware timeouts
- `temp_workspace.rs`: `TestWorkspace` builder pattern for creating test fixtures with:
  - Automatic temporary directory management
  - Fixture parsing with cursor markers (`$0`)
  - LSP client lifecycle management
  - Automatic file opening via `did_open`

**`tests/lsp_client_test.rs`**: Direct LSP client tests:
- Tests all 9 navigation tools (goto_definition, find_references, hover, etc.)
- Uses `TestWorkspace::builder()` pattern
- No explicit serialization needed - tests are independent

**`tests/mcp_tool_test.rs`**: MCP server tool interface tests:
- Tests MCP layer by invoking tools through `KadabraRunes` server
- Validates JSON-RPC responses and formatting
- Ensures LLM-friendly output format

**Test Timeouts** (CI-aware via `std::env::var("CI")`):
- Local: 60s init, 30s request, 2s indexing, 500ms file processing
- CI: 120s init, 60s request, 8s indexing, 3s file processing

## Response Format Conventions

All tool responses follow consistent LLM-friendly patterns:

**Location results** (goto_definition, find_references):
```
/path/to/file.rs:42:5
   40 | fn example() {
   41 |     let x = 10;
>  42 |     process(x);
   43 |     println!("Done");
```

**Symbol listings** (workspace_symbols):
```
[function] main - /path/to/main.rs:15
[struct] Config - /path/to/config.rs:8
```

**Hierarchical symbols** (document_symbols):
```
[struct] MyStruct (line 29)
  [field] value (line 31)
  [method] new (line 43)
```

## Common Patterns

### Writing Tests

Use the `TestWorkspace` builder pattern for integration tests:

```rust
#[tokio::test]
async fn test_goto_definition() {
    let ws = TestWorkspace::builder()
        .fixture(&common::comprehensive_fixture())
        .open_all_files()  // Automatically calls did_open on all files
        .build()
        .await;

    let result = ws.lsp()
        .goto_definition(&ws.apath("src/main.rs"), 7, 18)
        .await
        .expect("should succeed");

    // Assert on result...

    ws.lsp().shutdown().await.expect("shutdown should succeed");
}
```

**Fixture Format**:
- Fixtures use `//- /path` markers to define files
- `$0` marks cursor position for tests
- Paths are relative to workspace root
- Example: `//- /src/main.rs` followed by file content

### Error Handling
- Use `anyhow::Result` for main/CLI code with context
- Use domain-specific error types (`LspError`, `McpError`) for library code
- Graceful degradation when language server is unavailable
- Clear error messages with file paths and positions for debugging

### Logging
- ALL logs go to stderr via `tracing` (stdout is for MCP communication)
- Use structured logging: `info!(field = value, "message")`
- Default level is `info`, configurable via `--log-level` or `RUST_LOG`

### LSP Client Lifecycle
- Builder pattern for configuration (`LspClient::builder()`)
- Async initialization with timeout (default 30s, tests use 60s local / 120s CI)
- Cleanup on Drop - gracefully shuts down language server
- Document tracking via `did_open`/`did_close` notifications
- CI environments automatically get longer timeouts for reliable indexing

## Important Constraints

1. **Test Organization**: Tests are split into `lsp_client_test.rs` and `mcp_tool_test.rs` with shared `common/` module
2. **Test Fixtures**: Use `TestWorkspace::builder()` pattern with fixture strings containing `$0` cursor markers
3. **Workspace Paths**: Must be absolute paths, canonicalized before use (macOS symlink resolution: `/var` → `/private/var`)
4. **Stdout Reserved**: Never write to stdout except for MCP protocol messages (all logs go to stderr)
5. **LSP Version Compatibility**: `lsp-types` version must match `async-lsp` dependency
6. **Line/Column Indexing**: Tools accept 1-indexed (human) but convert to 0-indexed for LSP
7. **Config Command**: `kadabra-runes config` creates `.mcp.json` with proper MCP server configuration

## CI/CD Pipeline

### GitHub Actions Workflows

**`.github/workflows/ci.yml`** - Full CI pipeline:
- Runs on push to main/develop and all PRs
- Tests on Linux, macOS, and Windows
- Runs formatting checks, clippy, builds, and all tests
- Generates code coverage reports

**`.github/workflows/quick-test.yml`** - Fast feedback:
- Runs on all branches except main
- Quick checks: format, clippy, build, test
- Faster feedback for development branches

**`.github/workflows/release.yml`** - Release automation:
- Triggers on version tags (v*.*.*)
- Builds binaries for 6 platforms (Linux x64/ARM64, macOS x64/ARM64, Windows x64)
- Creates GitHub Release with pre-filled installation instructions
- Uploads release artifacts
- Optionally publishes to crates.io

### Creating a Release

Releases are automated using cargo-release:

1. Update CHANGELOG.md [Unreleased] section with changes
2. Commit CHANGELOG: `git commit -am "Update CHANGELOG for release"`
3. Push to main: `git push origin main`
4. Run cargo-release: `cargo release patch --execute` (or `minor`/`major`)

cargo-release will:
- Bump version in Cargo.toml
- Update CHANGELOG.md (move Unreleased → versioned)
- Commit, tag, and push
- Publish to crates.io

GitHub Actions will automatically:
- Build platform binaries
- Create GitHub Release

See PUBLISHING.md for detailed instructions.

### Required Secrets

For full release automation, configure these GitHub secrets:
- `CARGO_TOKEN` - For publishing to crates.io (optional)

## Known Limitations

1. **workspace_symbols Behavior**: rust-analyzer's workspace_symbols has quirks:
   - Works reliably for structs, enums, and traits (100% match rate)
   - Function names may not always resolve in workspace_symbols queries
   - Use `document_symbols` for reliable function discovery in specific files
   - `goto_definition` works with symbols currently in opened documents

2. **Symbol Resolution**: goto_definition is limited to symbols in currently opened documents. For best results, ensure files are opened via `did_open` before querying.

## Future Enhancements (Documented in README)

- Support for TypeScript/JavaScript (typescript-language-server)
- Support for Python (pylsp/pyright)
- Support for Go (gopls)
- Improved symbol name-based queries across unopened files
- Diagnostics tool (compiler errors/warnings)
- Code actions (quick fixes, refactorings)
- Response caching for performance
- Batch operations
