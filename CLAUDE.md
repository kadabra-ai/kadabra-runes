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

Integration tests use `serial_test` crate to automatically run one at a time (no manual flags needed):

```bash
# Run all tests
cargo test --test integration_test

# Run specific test
cargo test --test integration_test test_goto_definition

# With debug logging
RUST_LOG=debug cargo test --test integration_test -- --nocapture

# Run all tests (including unit tests)
cargo test
```

### Running the Server

```bash
# Run for current directory
cargo run

# Run for specific workspace
cargo run -- --workspace /path/to/rust/project

# With debug logging
cargo run -- --workspace . --log-level debug

# Using installed binary
kadabra-runes --workspace /path/to/project
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

**`src/lib.rs`**: Library root that re-exports the three main modules.

**`src/error.rs`**: Centralized error types (`LspError`, `McpError`) using `thiserror`.

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

`tests/integration_test.rs`: End-to-end tests that:
- Spawn real rust-analyzer instances
- Use `tests/fixtures/sample_project` as test workspace
- Automatically serialized via `serial_test` crate (using `#[serial]` attribute) to prevent conflicts
- CI-aware timeouts to handle slower CI environments:
  - Local: 60s init, 30s request, 2s indexing wait, 500ms file processing
  - CI: 120s init, 60s request, 8s indexing wait, 3s file processing
- Test all 9 navigation tools with realistic scenarios
- Detect CI via `std::env::var("CI").is_ok()` for automatic timeout adjustment

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

1. **Integration Tests Serialization**: Tests use `#[serial]` attribute from `serial_test` crate to run sequentially
2. **Workspace Paths**: Must be absolute paths, canonicalized before use
3. **Stdout Reserved**: Never write to stdout except for MCP protocol messages
4. **LSP Version Compatibility**: `lsp-types` version must match `async-lsp` dependency
5. **Line/Column Indexing**: Tools accept 1-indexed (human) but convert to 0-indexed for LSP

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

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md` with release notes
3. Commit changes: `git commit -am "Release v0.1.0"`
4. Create and push tag: `git tag v0.1.0 && git push origin v0.1.0`
5. GitHub Actions will automatically build and create the release

### Required Secrets

For full release automation, configure these GitHub secrets:
- `CARGO_TOKEN` - For publishing to crates.io (optional)

## Future Enhancements (Documented in README)

- Support for TypeScript/JavaScript (typescript-language-server)
- Support for Python (pylsp/pyright)
- Support for Go (gopls)
- Symbol name-based queries (currently position-based only)
- Diagnostics tool (compiler errors/warnings)
- Code actions (quick fixes, refactorings)
- Response caching for performance
- Batch operations
