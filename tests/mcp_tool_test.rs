//! End-to-end tests for MCP server tools.
//!
//! These tests validate the complete MCP tool interface by invoking tools
//! through the MCP server and verifying responses.
//!
//! To run these tests:
//! ```bash
//! # Run all MCP tool tests
//! cargo test --test mcp_tool_test
//!
//! # Run with debug output
//! RUST_LOG=debug cargo test --test mcp_tool_test -- --nocapture
//!
//! # Run specific test
//! cargo test --test mcp_tool_test test_mcp_goto_definition_by_name
//! ```
mod common;

use common::temp_workspace::TestWorkspace;
use kadabra_runes::mcp::KadabraRunes;
use kadabra_runes::mcp::tools::PositionParams;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::RawContent;

#[tokio::test]
async fn test_mcp_goto_definition_tool() {
    let ws = TestWorkspace::builder()
        .fixture(&common::comprehensive_fixture())
        .open_all_files()
        .build()
        .await;
    let server = KadabraRunes::new(ws.root.path().into(), ws.lsp());
    // Invoke goto_definition tool with parameters
    let params = PositionParams {
        file_path: ws.root.path().join("src/main.rs").display().to_string(),
        line: 7,
        column: 18,
    };

    let result = server
        .goto_definition(Parameters(params))
        .await
        .expect("goto_definition tool should succeed");

    // Validate response structure
    // CallToolResult::success() sets is_error to Some(false)
    assert_eq!(result.is_error, Some(false), "Should not be an error");
    assert!(!result.content.is_empty(), "Should have content");

    // Extract and validate text content
    // Content is Annotated<RawContent>, need to access the raw field
    let text = match &result.content[0].raw {
        RawContent::Text(text_content) => &text_content.text,
        _ => panic!("Expected Text content, got: {:?}", result.content[0]),
    };

    // Validate formatted output contains expected elements
    assert!(
        text.contains("lib.rs"),
        "Should reference lib.rs, got: {}",
        text
    );
    assert!(
        text.contains("add"),
        "Should show 'add' function, got: {}",
        text
    );
    assert!(text.contains("22"), "Should show line 22, got: {}", text);
    assert!(text.contains(">"), "Should have line marker, got: {}", text);
    assert!(
        text.contains("pub fn"),
        "Should show function definition, got: {}",
        text
    );
}
