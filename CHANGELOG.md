# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial release of Kadabra Runes MCP server
- Support for 9 LSP-powered navigation tools
- Integration with rust-analyzer
- Stdio transport for MCP communication
- Comprehensive integration tests
- GitHub Actions CI/CD pipeline

### Changed
- N/A

### Deprecated
- N/A

### Removed
- N/A

### Fixed
- N/A

### Security
- N/A

## [0.1.0] - TBD

### Added
- Initial implementation of MCP server
- LSP client with full rust-analyzer integration
- Nine code navigation tools:
  - `goto_definition` - Jump to symbol definitions
  - `find_references` - Find all references to symbols
  - `hover` - Get type information and documentation
  - `document_symbols` - List all symbols in a file
  - `workspace_symbols` - Search symbols across workspace
  - `incoming_calls` - Find functions that call a given function
  - `outgoing_calls` - Find functions called by a given function
  - `implementations` - Find trait/interface implementations
  - `type_definition` - Jump to type definitions
- Formatted, LLM-friendly responses
- Comprehensive error handling
- Integration tests with real rust-analyzer
- Documentation and installation guides

[Unreleased]: https://github.com/kadabra-ai/kadabra-runes/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/kadabra-ai/kadabra-runes/releases/tag/v0.1.0
