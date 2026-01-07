//! Integration tests for the Kadabra Runes MCP server.
//!
//! These tests verify that all 9 LSP tools work correctly with rust-analyzer
//! using a realistic test fixture project.
//!
//! To run these tests:
//! - Standard run: `cargo test --test integration_test`
//! - With debug output: `RUST_LOG=debug cargo test --test integration_test`
//! - Run single test: `cargo test --test integration_test test_goto_definition`
//!
//! Note: Tests are automatically serialized using the `serial_test` crate to avoid
//! conflicts between multiple rust-analyzer instances accessing the same fixture project.
//! You no longer need to use `--test-threads=1`.
//!
//! Note: These tests require rust-analyzer to be installed (via `rustup component add rust-analyzer`).

mod common;
mod lsp_client_test;
mod mcp_tool_test;
