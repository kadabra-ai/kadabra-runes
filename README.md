# Kadabra Runes MCP Server

A [Model Context Protocol (MCP)](https://modelcontextprotocol.io/) server that provides semantic code navigation for Rust projects through the Language Server Protocol (LSP). Enables LLM applications like Claude Code to intelligently navigate codebases using rust-analyzer.

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Quick Start

```bash
# Install
cargo install --path .

# Run for your Rust project
kadabra-runes --workspace /path/to/your/project
```

**🔌 Want to use with Claude Desktop, ChatGPT, or other LLM apps?** See **[INSTALL.md](INSTALL.md)** for step-by-step integration guides.

## Features

### 🎯 Semantic Code Navigation

Provides 9 powerful navigation tools backed by rust-analyzer:

- **goto_definition** - Jump from symbol usage to its definition
- **find_references** - Find all references to a symbol across the workspace
- **hover** - Get type information, signatures, and documentation
- **document_symbols** - List all symbols in a file (functions, structs, traits, etc.)
- **workspace_symbols** - Search for symbols across the entire workspace
- **incoming_calls** - Find all functions that call a given function
- **outgoing_calls** - Find all functions called by a given function
- **implementations** - Find all implementations of a trait or interface
- **type_definition** - Jump to the type definition of a symbol

### 🚀 Key Capabilities

- **LLM-Optimized Responses** - Returns concise, context-rich results perfect for LLM consumption
- **Real-time Semantic Analysis** - Leverages rust-analyzer's powerful type system understanding
- **Zero Configuration** - Works out of the box with any Rust project
- **Async/Non-blocking** - Handles multiple concurrent requests efficiently
- **Robust Error Handling** - Graceful degradation with helpful error messages

## Architecture

```
┌─────────────────────────┐
│   LLM Application       │
│   (Claude Code, etc.)   │
└───────────┬─────────────┘
            │ MCP Protocol (stdio)
            │ JSON-RPC
            ▼
┌─────────────────────────┐
│  Kadabra Runes Server   │
│  - MCP Tool Router      │
│  - Response Formatter   │
└───────────┬─────────────┘
            │ LSP Protocol (stdio)
            │ JSON-RPC
            ▼
┌─────────────────────────┐
│     rust-analyzer       │
│  - Type Inference       │
│  - Symbol Resolution    │
│  - Call Graph Analysis  │
└───────────┬─────────────┘
            │
            ▼
┌─────────────────────────┐
│     Rust Codebase       │
└─────────────────────────┘
```

## Installation

> 📖 **For detailed installation instructions for Claude Desktop, Claude Code, ChatGPT Desktop, Gemini CLI, and Codex CLI, see [INSTALL.md](INSTALL.md)**

### Prerequisites

- Rust 1.70 or later
- rust-analyzer (installed automatically with `rustup component add rust-analyzer`)

### From Source

```bash
git clone https://github.com/kadabra-ai/kadabra-runes.git
cd kadabra-runes
cargo build --release
```

The binary will be available at `target/release/kadabra-runes`.

### Install via Cargo

```bash
cargo install --path .
```

## Quick Setup

After installing kadabra-runes, configure your MCP client with one command:

```bash
cd /path/to/your/rust/project
kadabra-runes config
```

This creates `.mcp.json` in your project root with the kadabra-runes configuration. The file uses the standard MCP configuration format that works with:
- **Claude Code (CLI)**
- **Gemini CLI**
- **Other MCP clients** that support project-level `.mcp.json`

> **Tip**: Commit `.mcp.json` to git to share the configuration with your team!

## Usage

### Basic Usage

Start the MCP server for a Rust workspace:

```bash
kadabra-runes --workspace /path/to/your/rust/project
```

The server will:
1. Start rust-analyzer on your workspace
2. Wait for initialization and indexing
3. Listen for MCP requests on stdin
4. Send MCP responses on stdout

### CLI Options

```
kadabra-runes [OPTIONS]

Options:
  -w, --workspace <PATH>
          Workspace root directory to navigate
          [default: .]

  -l, --language-server <CMD>
          Language server command to use
          [default: rust-analyzer]

      --language-server-args <ARGS>...
          Arguments to pass to the language server

      --log-level <LEVEL>
          Log level: trace, debug, info, warn, error
          [default: info]

  -h, --help
          Print help information

  -V, --version
          Print version information
```

### Example Commands

```bash
# Navigate current directory
kadabra-runes

# Specify workspace
kadabra-runes --workspace ~/projects/my-rust-app

# Enable debug logging
kadabra-runes --log-level debug

# Use custom rust-analyzer
kadabra-runes --language-server /usr/local/bin/rust-analyzer
```

### MCP Integration

#### With Claude Code

Add to your MCP configuration:

```json
{
  "mcpServers": {
    "kadabra-runes": {
      "command": "kadabra-runes",
      "args": ["--workspace", "/path/to/workspace"],
      "type": "stdio"
    }
  }
}
```

#### MCP Tool Examples

**Find Definition:**
```json
{
  "name": "goto_definition",
  "arguments": {
    "file_path": "/path/to/src/main.rs",
    "line": 42,
    "column": 15
  }
}
```

**Search Symbols:**
```json
{
  "name": "workspace_symbols",
  "arguments": {
    "query": "HashMap",
    "max_results": 20
  }
}
```

**Get Hover Info:**
```json
{
  "name": "hover",
  "arguments": {
    "file_path": "/path/to/src/lib.rs",
    "line": 10,
    "column": 5
  }
}
```

## Response Format

Responses are formatted for optimal LLM consumption:

### Location Results (goto_definition, find_references, etc.)

```
/path/to/file.rs:42:5
   40 | fn example() {
   41 |     let x = 10;
>  42 |     process(x);
   43 |     println!("Done");
   44 | }
```

### Symbol Listings (workspace_symbols)

```
[function] main - /path/to/main.rs:15
[struct] Config - /path/to/config.rs:8
[method] new (in Config) - /path/to/config.rs:12
```

### Hierarchical Symbols (document_symbols)

```
[struct] KadabraRunes (line 29)
  [field] workspace_root (line 31)
  [field] lsp_client (line 33)
  [method] new (line 43)
  [method] workspace_root (line 51)
```

### Hover Information

````markdown
```rust
pub fn goto_definition(&self, path: &Path, line: u32, column: u32) -> LspResult<GotoDefinitionResponse>
```

Gets the definition location(s) for the symbol at the given position.

# Arguments
* `path` - File path containing the symbol
* `line` - Line number (1-indexed)
* `column` - Column number (1-indexed)
````

## Development

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Check without building
cargo check
```

### Testing

#### Run All Tests

```bash
cargo test
```

#### Run Integration Tests

```bash
# Integration tests MUST run single-threaded
cargo test --test integration_test -- --test-threads=1
```

#### Run Specific Test

```bash
cargo test --test integration_test test_goto_definition -- --test-threads=1
```

#### Test with Debug Logging

```bash
RUST_LOG=debug cargo test --test integration_test -- --test-threads=1 --nocapture
```

### Project Structure

```
kadabra-runes/
├── src/
│   ├── main.rs              # CLI entry point
│   ├── lib.rs               # Library exports
│   ├── error.rs             # Error types
│   ├── mcp/
│   │   ├── mod.rs           # MCP module
│   │   ├── server.rs        # MCP server and tool implementations
│   │   ├── tools.rs         # Tool parameter and response types
│   │   └── transport.rs     # Transport abstractions
│   └── lsp/
│       ├── mod.rs           # LSP module
│       ├── client.rs        # LSP client implementation
│       └── types.rs         # Helper types and conversions
├── tests/
│   ├── integration_test.rs  # Integration tests
│   └── fixtures/            # Test Rust project
├── Cargo.toml               # Dependencies and metadata
├── REQUIREMENTS.md          # Detailed requirements
└── README.md                # This file
```

## Performance

- **Startup Time:** ~2-5 seconds (rust-analyzer initialization)
- **Query Latency:** 10-200ms depending on operation complexity
- **Memory Usage:** ~200-500MB (rust-analyzer overhead)
- **Concurrent Requests:** Fully async, handles multiple simultaneous requests

## Limitations

- **Rust Only:** Currently only supports Rust via rust-analyzer (other languages planned)
- **Symbol Name Queries:** Direct symbol search not yet implemented; use `workspace_symbols` instead
- **No Caching:** Each request queries rust-analyzer directly (caching planned)
- **Single Workspace:** One workspace per server instance

## Troubleshooting

### rust-analyzer Not Found

Ensure rust-analyzer is installed:
```bash
rustup component add rust-analyzer
```

Or install manually:
```bash
# macOS/Linux
brew install rust-analyzer

# Or download from GitHub releases
```

### Server Hangs on Startup

rust-analyzer may be indexing a large workspace. Check logs:
```bash
kadabra-runes --log-level debug
```

### No Results for Queries

1. Ensure the file path is absolute
2. Check that the workspace is a valid Rust project with `Cargo.toml`
3. Verify line/column numbers are within file bounds (1-indexed)
4. Wait for rust-analyzer to finish indexing

### Integration Tests Fail

Make sure to run with single thread:
```bash
cargo test --test integration_test -- --test-threads=1
```

## Future Enhancements

- [ ] Support for TypeScript/JavaScript (via typescript-language-server)
- [ ] Support for Python (via pylsp/pyright)
- [ ] Support for Go (via gopls)
- [ ] Symbol name-based queries (search then goto)
- [ ] Diagnostics tool (compiler errors/warnings)
- [ ] Code actions (quick fixes, refactorings)
- [ ] Response caching for better performance
- [ ] Batch operations (multiple queries in one request)
- [ ] Multiple simultaneous language servers

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes with tests
4. Ensure all tests pass (`cargo test`)
5. Commit your changes (`git commit -m 'Add amazing feature'`)
6. Push to the branch (`git push origin feature/amazing-feature`)
7. Open a Pull Request

### Development Guidelines

- Follow Rust idioms and conventions
- Add tests for new features
- Update documentation
- Keep response formats LLM-friendly
- Handle errors gracefully

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [Model Context Protocol](https://modelcontextprotocol.io/) - MCP specification
- [rust-analyzer](https://rust-analyzer.github.io/) - Rust language server
- [async-lsp](https://github.com/oxalica/async-lsp) - Async LSP client/server library
- [rmcp](https://github.com/modelcontextprotocol/rust-sdk) - Rust MCP SDK

## Contact

- **Project:** [github.com/kadabra-ai/kadabra-runes](https://github.com/kadabra-ai/kadabra-runes)
- **Issues:** [github.com/kadabra-ai/kadabra-runes/issues](https://github.com/kadabra-ai/kadabra-runes/issues)

---

**Built with ❤️ for the Rust and LLM communities**
