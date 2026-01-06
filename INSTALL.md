# Installation Guide

This guide covers how to install and configure the Kadabra Runes MCP server with various LLM applications.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Building the Server](#building-the-server)
- [Claude Desktop](#claude-desktop)
- [Claude Code (CLI)](#claude-code-cli)
- [ChatGPT Desktop](#chatgpt-desktop)
- [Gemini CLI](#gemini-cli)
- [Codex CLI](#codex-cli)
- [Verification](#verification)
- [Troubleshooting](#troubleshooting)

## Prerequisites

Before installing, ensure you have:

1. **Rust toolchain** (1.70 or later)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **rust-analyzer** language server
   ```bash
   rustup component add rust-analyzer
   ```

3. **Kadabra Runes** binary (see [Building the Server](#building-the-server))

## Building the Server

### Option 1: Install from Source

```bash
# Clone the repository
git clone https://github.com/kadabra-ai/kadabra-runes.git
cd kadabra-runes

# Build and install
cargo install --path .
```

The binary will be installed to `~/.cargo/bin/kadabra-runes` (ensure this is in your PATH).

### Option 2: Build for Development

```bash
# Build in release mode
cargo build --release

# Binary will be at: target/release/kadabra-runes
# You can copy it to your PATH or use the full path in configurations
```

### Verify Installation

```bash
kadabra-runes --version
# Should output: kadabra-runes 0.1.0
```

---

## Claude Desktop

[Claude Desktop](https://claude.ai/download) supports MCP servers via configuration file.

### macOS

1. **Locate the config file:**
   ```bash
   # Config location:
   ~/Library/Application Support/Claude/claude_desktop_config.json
   ```

2. **Edit the configuration:**
   ```bash
   # Create directory if it doesn't exist
   mkdir -p ~/Library/Application\ Support/Claude

   # Edit the config file
   nano ~/Library/Application\ Support/Claude/claude_desktop_config.json
   ```

3. **Add Kadabra Runes server:**
   ```json
   {
     "mcpServers": {
       "kadabra-runes": {
         "command": "kadabra-runes",
         "args": [
           "--workspace",
           "/path/to/your/rust/project",
           "--log-level",
           "info"
         ]
       }
     }
   }
   ```

   **For multiple projects:**
   ```json
   {
     "mcpServers": {
       "kadabra-runes-myproject": {
         "command": "kadabra-runes",
         "args": ["--workspace", "/Users/you/projects/myproject"]
       },
       "kadabra-runes-otherproject": {
         "command": "kadabra-runes",
         "args": ["--workspace", "/Users/you/projects/otherproject"]
       }
     }
   }
   ```

4. **Restart Claude Desktop**

5. **Verify in Claude:**
   - Look for the 🔌 (plug) icon indicating MCP servers are connected
   - Type a message asking Claude to use the kadabra runes
   - Example: "Use the kadabra-runes tool to find the definition of `main` function in src/main.rs"

### Windows

1. **Config location:**
   ```
   %APPDATA%\Claude\claude_desktop_config.json
   ```

2. **Edit configuration:**
   ```powershell
   notepad %APPDATA%\Claude\claude_desktop_config.json
   ```

3. **Add server (use Windows path format):**
   ```json
   {
     "mcpServers": {
       "kadabra-runes": {
         "command": "kadabra-runes",
         "args": [
           "--workspace",
           "C:\\Users\\YourName\\projects\\rust-project"
         ]
       }
     }
   }
   ```

### Linux

1. **Config location:**
   ```
   ~/.config/Claude/claude_desktop_config.json
   ```

2. **Follow the same steps as macOS** with appropriate Linux paths.

---

## Claude Code (CLI)

[Claude Code](https://github.com/anthropics/claude-code) is Anthropic's CLI tool for AI-assisted development.

### Installation

1. **Create or edit MCP settings:**
   ```bash
   # Settings location:
   ~/.claude/settings.json
   ```

2. **Add Kadabra Runes:**
   ```json
   {
     "mcpServers": {
       "kadabra-runes": {
         "command": "kadabra-runes",
         "args": ["--workspace", "${workspaceFolder}"]
       }
     }
   }
   ```

   The `${workspaceFolder}` variable will be automatically replaced with the current working directory.

3. **Alternative: Per-project configuration**

   Create `.claude/settings.local.json` in your project root:
   ```json
   {
     "mcpServers": {
       "kadabra-runes": {
         "command": "kadabra-runes",
         "args": ["--workspace", "."]
       }
     }
   }
   ```

### Usage

```bash
# Start Claude Code in your Rust project
cd /path/to/your/rust/project
claude

# The MCP server will start automatically
# Ask Claude to navigate your code:
> Find all references to the Config struct
> Show me the implementation of the calculate function
> What calls the initialize function?
```

---

## ChatGPT Desktop

ChatGPT Desktop is adding MCP support. Configuration varies by version.

### Current Status

As of January 2025, ChatGPT Desktop's MCP support is in beta. Check [OpenAI's documentation](https://help.openai.com/) for the latest.

### Expected Configuration (Beta)

1. **Config location (macOS):**
   ```
   ~/Library/Application Support/ChatGPT/mcp_config.json
   ```

2. **Add Kadabra Runes:**
   ```json
   {
     "servers": {
       "kadabra-runes": {
         "command": "kadabra-runes",
         "args": [
           "--workspace",
           "/path/to/your/project"
         ],
         "transport": "stdio"
       }
     }
   }
   ```

3. **Restart ChatGPT Desktop**

### Alternative: Manual Mode

If native MCP isn't available yet:

1. **Run server manually:**
   ```bash
   kadabra-runes --workspace /path/to/project
   ```

2. **Use ChatGPT's function calling** (if supported) to interact with the server via stdio

---

## Gemini CLI

Google's Gemini CLI tool with MCP support.

### Installation

1. **Check Gemini CLI version:**
   ```bash
   gemini --version
   # Ensure MCP support is available
   ```

2. **Create Gemini MCP config:**
   ```bash
   # Config location (may vary):
   ~/.config/gemini/mcp.json
   ```

3. **Add Kadabra Runes:**
   ```json
   {
     "mcp_servers": {
       "kadabra-runes": {
         "type": "stdio",
         "command": "kadabra-runes",
         "arguments": [
           "--workspace",
           "${CWD}"
         ],
         "description": "Semantic code navigation for Rust projects"
       }
     }
   }
   ```

### Usage

```bash
cd /path/to/rust/project
gemini

# In Gemini CLI:
> Use kadabra-runes to find the definition of handle_request
```

---

## Codex CLI

GitHub Codex CLI with MCP integration.

### Installation

1. **Codex config location:**
   ```bash
   ~/.codex/config.yaml
   ```

2. **Add MCP server:**
   ```yaml
   mcp:
     servers:
       kadabra-runes:
         command: kadabra-runes
         args:
           - "--workspace"
           - "${workspace}"
         transport: stdio
         enabled: true
   ```

3. **Alternative: JSON config** (if using `config.json`):
   ```json
   {
     "mcp": {
       "servers": {
         "kadabra-runes": {
           "command": "kadabra-runes",
           "args": ["--workspace", "${workspace}"],
           "transport": "stdio",
           "enabled": true
         }
       }
     }
   }
   ```

### Usage

```bash
# Start Codex in your project
cd /path/to/rust/project
codex

# Use code navigation
> @kadabra-runes find references to MyStruct
```

---

## Verification

### Test the MCP Server Directly

You can test the server manually via stdio:

1. **Start the server:**
   ```bash
   kadabra-runes --workspace /path/to/rust/project --log-level debug
   ```

2. **Send a test MCP request** (via stdin):
   ```json
   {
     "jsonrpc": "2.0",
     "id": 1,
     "method": "initialize",
     "params": {
       "protocolVersion": "2024-11-05",
       "capabilities": {},
       "clientInfo": {
         "name": "test-client",
         "version": "1.0.0"
       }
     }
   }
   ```

3. **Expected response:**
   ```json
   {
     "jsonrpc": "2.0",
     "id": 1,
     "result": {
       "protocolVersion": "2024-11-05",
       "capabilities": {
         "tools": {}
       },
       "serverInfo": {
         "name": "kadabra-runes",
         "version": "0.1.0"
       }
     }
   }
   ```

### Check Server Logs

Monitor logs to ensure the server is running:

```bash
# Logs are written to stderr
kadabra-runes --workspace . --log-level debug 2> navigator.log

# In another terminal:
tail -f navigator.log
```

---

## Troubleshooting

### Server Not Found

**Problem:** `command not found: kadabra-runes`

**Solution:**
```bash
# Ensure ~/.cargo/bin is in your PATH
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc

# Or use full path in config:
{
  "command": "/Users/yourname/.cargo/bin/kadabra-runes"
}
```

### rust-analyzer Not Found

**Problem:** Server fails with "rust-analyzer not found"

**Solution:**
```bash
# Install rust-analyzer
rustup component add rust-analyzer

# Or specify full path:
kadabra-runes --language-server /usr/local/bin/rust-analyzer
```

### MCP Server Not Loading in Claude Desktop

**Problem:** Server doesn't appear in Claude Desktop

**Solutions:**

1. **Check config file syntax:**
   ```bash
   # Validate JSON
   cat ~/Library/Application\ Support/Claude/claude_desktop_config.json | jq
   ```

2. **Check file permissions:**
   ```bash
   chmod 644 ~/Library/Application\ Support/Claude/claude_desktop_config.json
   ```

3. **View Claude Desktop logs:**
   ```bash
   # macOS
   ~/Library/Logs/Claude/mcp*.log

   # Check for errors
   tail -f ~/Library/Logs/Claude/mcp*.log
   ```

4. **Restart Claude Desktop completely:**
   - Quit Claude Desktop (not just close window)
   - On macOS: Cmd+Q to fully quit
   - Restart the application

### Server Starts But No Results

**Problem:** Server runs but returns empty results

**Possible Causes:**

1. **Wrong workspace path:**
   ```bash
   # Ensure path is absolute and contains Cargo.toml
   ls /path/to/workspace/Cargo.toml
   ```

2. **rust-analyzer still indexing:**
   - Wait 10-30 seconds after startup
   - Check logs for "indexing complete"

3. **Invalid position parameters:**
   - Ensure line/column are 1-indexed
   - Ensure line/column are within file bounds

### Permission Denied

**Problem:** `Permission denied` when starting server

**Solution:**
```bash
# Make binary executable
chmod +x ~/.cargo/bin/kadabra-runes

# Or if using local build:
chmod +x target/release/kadabra-runes
```

### Multiple rust-analyzer Instances

**Problem:** Multiple servers conflicting

**Solution:**
- Each MCP server instance starts its own rust-analyzer
- Use separate workspace configurations
- Ensure each project has one server instance

---

## Advanced Configuration

### Custom rust-analyzer Settings

Pass arguments to rust-analyzer:

```json
{
  "command": "kadabra-runes",
  "args": [
    "--workspace", "/path/to/project",
    "--language-server", "rust-analyzer",
    "--language-server-args", "--log-file=/tmp/ra.log"
  ]
}
```

### Environment Variables

Set environment variables for the MCP server:

```json
{
  "command": "kadabra-runes",
  "args": ["--workspace", "/path/to/project"],
  "env": {
    "RUST_LOG": "debug",
    "RUST_BACKTRACE": "1"
  }
}
```

### Multiple Workspaces

Configure multiple servers for different projects:

```json
{
  "mcpServers": {
    "nav-backend": {
      "command": "kadabra-runes",
      "args": ["--workspace", "/projects/backend"]
    },
    "nav-frontend": {
      "command": "kadabra-runes",
      "args": ["--workspace", "/projects/frontend"]
    },
    "nav-shared": {
      "command": "kadabra-runes",
      "args": ["--workspace", "/projects/shared-lib"]
    }
  }
}
```

---

## Platform-Specific Notes

### macOS

- Use full paths in configuration
- Code signing may require allowing the binary in Security & Privacy settings
- Use `open -a Claude` to restart Claude Desktop from terminal

### Windows

- Use backslashes or forward slashes in paths: `C:\\projects\\...` or `C:/projects/...`
- PowerShell may require execution policy changes
- Use Task Manager to fully close applications before restart

### Linux

- Ensure `$HOME/.cargo/bin` is in PATH
- Some distros require `libssl-dev` for building
- Use `killall claude` to ensure clean restart

---

## Getting Help

If you encounter issues:

1. **Check server logs** with `--log-level debug`
2. **Verify rust-analyzer** works: `rust-analyzer --version`
3. **Test server manually** using stdio
4. **Open an issue** at [github.com/kadabra-ai/kadabra-runes/issues](https://github.com/kadabra-ai/kadabra-runes/issues)

Include in your report:
- Operating system and version
- LLM application and version
- Kadabra Runes version
- Full error messages
- Configuration file contents (sanitized)

---

**Happy Coding with Semantic Navigation!** 🚀
