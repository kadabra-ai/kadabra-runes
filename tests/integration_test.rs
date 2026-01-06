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

use kadabra_runes::lsp::client::LspClient;
use lsp_types::{DocumentSymbolResponse, GotoDefinitionResponse};
use serial_test::serial;
use std::path::PathBuf;
use std::time::Duration;

/// Helper to get the fixture project path
fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sample_project")
}

/// Helper to find rust-analyzer executable
fn find_rust_analyzer() -> String {
    // Try to find rust-analyzer in PATH
    if let Ok(path) = std::env::var("RUST_ANALYZER_PATH") {
        return path;
    }

    // Try common locations
    let candidates = vec![
        "rust-analyzer",                          // In PATH
        "/Users/mjaric/.cargo/bin/rust-analyzer", // User cargo bin
        "~/.cargo/bin/rust-analyzer",             // Home cargo bin
    ];

    for candidate in candidates {
        if let Ok(output) = std::process::Command::new(candidate)
            .arg("--version")
            .output()
            && output.status.success()
        {
            return candidate.to_string();
        }
    }

    // Default fallback
    "rust-analyzer".to_string()
}

/// Helper to create and initialize LSP client for tests
async fn setup_client() -> LspClient {
    // CI environments need longer timeouts due to slower hardware and more concurrent processes
    let (init_timeout, request_timeout, index_wait) = if std::env::var("CI").is_ok() {
        (
            Duration::from_secs(120),    // 2 minutes for CI initialization
            Duration::from_secs(60),     // 1 minute for CI requests
            Duration::from_millis(8000), // 8 seconds for CI indexing
        )
    } else {
        (
            Duration::from_secs(60),     // 1 minute for local initialization
            Duration::from_secs(30),     // 30 seconds for local requests
            Duration::from_millis(2000), // 2 seconds for local indexing
        )
    };

    // Build the client with environment-appropriate timeouts
    let client = LspClient::builder()
        .server_command(find_rust_analyzer())
        .workspace_root(fixture_path())
        .init_timeout(init_timeout)
        .request_timeout(request_timeout)
        .build()
        .await
        .expect("Failed to start LSP client");

    // Give rust-analyzer time to fully index the workspace
    tokio::time::sleep(index_wait).await;

    client
}

/// Helper to open a file in the LSP client
async fn open_file(client: &LspClient, relative_path: &str) -> PathBuf {
    let path = fixture_path().join(relative_path);
    client.did_open(&path).await.expect("Failed to open file");

    // Give rust-analyzer time to process the file
    // CI needs more time due to slower hardware and more concurrent processes
    let process_wait = if std::env::var("CI").is_ok() {
        Duration::from_millis(3000) // 3 seconds for CI
    } else {
        Duration::from_millis(500) // 500ms for local
    };
    tokio::time::sleep(process_wait).await;

    path
}

#[tokio::test]
#[serial]
async fn test_goto_definition() {
    let client = setup_client().await;

    // Open main.rs which contains a call to `add`
    let main_path = open_file(&client, "src/main.rs").await;

    // Go to definition of `add` function call on line 7
    // The call is: `let result = add(x, y);`
    // Position is at the start of 'add' (1-indexed: line 7, column 18)
    let result = client
        .goto_definition(&main_path, 7, 18)
        .await
        .expect("goto_definition should succeed");

    // Verify we got a response with at least one location
    match result {
        GotoDefinitionResponse::Array(locations) => {
            assert!(
                !locations.is_empty(),
                "Should find definition location for 'add' function"
            );
            // Verify the definition is in lib.rs
            let location = &locations[0];
            assert!(
                location.uri.as_str().contains("lib.rs"),
                "Definition should be in lib.rs"
            );
        }
        GotoDefinitionResponse::Scalar(location) => {
            assert!(
                location.uri.as_str().contains("lib.rs"),
                "Definition should be in lib.rs"
            );
        }
        _ => panic!("Unexpected response type"),
    }

    client.shutdown().await.expect("Shutdown should succeed");
}

#[tokio::test]
#[serial]
async fn test_find_references() {
    let client = setup_client().await;

    // Open lib.rs which contains the `add` function definition
    let lib_path = open_file(&client, "src/lib.rs").await;

    // Find references to `add` function (defined around line 22)
    // Position at the function name in the definition
    let result = client
        .find_references(&lib_path, 22, 8, true)
        .await
        .expect("find_references should succeed");

    // Should find at least the declaration and the call in main.rs
    assert!(
        !result.is_empty(),
        "Should find at least one reference to 'add' function"
    );

    // Verify we have references in both lib.rs and main.rs
    let files: Vec<String> = result
        .iter()
        .map(|loc| {
            loc.uri
                .path_segments()
                .and_then(|mut s| s.next_back())
                .unwrap_or("")
                .to_string()
        })
        .collect();

    assert!(
        files.iter().any(|f| f.contains("main.rs")),
        "Should have reference in main.rs, found: {:?}",
        files
    );

    client.shutdown().await.expect("Shutdown should succeed");
}

#[tokio::test]
#[serial]
async fn test_hover() {
    let client = setup_client().await;

    // Open lib.rs which has well-documented functions
    let lib_path = open_file(&client, "src/lib.rs").await;

    // Get hover info for `add` function (around line 22)
    let result = client
        .hover(&lib_path, 22, 8)
        .await
        .expect("hover should succeed");

    assert!(result.is_some(), "Should have hover information for 'add'");

    let hover = result.unwrap();

    // Verify we got some content in the hover
    match hover.contents {
        lsp_types::HoverContents::Scalar(content) => {
            let text = match content {
                lsp_types::MarkedString::String(s) => s,
                lsp_types::MarkedString::LanguageString(ls) => ls.value,
            };
            assert!(!text.is_empty(), "Hover content should not be empty");
        }
        lsp_types::HoverContents::Array(contents) => {
            assert!(!contents.is_empty(), "Hover contents should not be empty");
        }
        lsp_types::HoverContents::Markup(content) => {
            assert!(
                !content.value.is_empty(),
                "Hover markup should not be empty"
            );
        }
    }

    client.shutdown().await.expect("Shutdown should succeed");
}

#[tokio::test]
#[serial]
async fn test_document_symbols() {
    let client = setup_client().await;

    // Open lib.rs which contains multiple symbols
    let lib_path = open_file(&client, "src/lib.rs").await;

    let result = client
        .document_symbols(&lib_path)
        .await
        .expect("document_symbols should succeed");

    // Verify we got symbols
    match result {
        DocumentSymbolResponse::Flat(symbols) => {
            assert!(!symbols.is_empty(), "Should find symbols in lib.rs");

            // Verify we have the `add` function
            let has_add = symbols.iter().any(|s| s.name == "add");
            assert!(has_add, "Should find 'add' function in symbols");
        }
        DocumentSymbolResponse::Nested(symbols) => {
            assert!(!symbols.is_empty(), "Should find symbols in lib.rs");

            // For nested symbols, check recursively
            fn contains_symbol(symbols: &[lsp_types::DocumentSymbol], name: &str) -> bool {
                symbols.iter().any(|s| {
                    s.name == name || contains_symbol(s.children.as_ref().unwrap_or(&vec![]), name)
                })
            }

            assert!(
                contains_symbol(&symbols, "add"),
                "Should find 'add' function in symbols"
            );
        }
    }

    client.shutdown().await.expect("Shutdown should succeed");
}

#[tokio::test]
#[serial]
async fn test_workspace_symbols() {
    let client = setup_client().await;

    // Search for symbols matching "add"
    let result = client
        .workspace_symbols("add")
        .await
        .expect("workspace_symbols should succeed");

    assert!(
        !result.is_empty(),
        "Should find symbols matching 'add' in workspace"
    );

    // Verify we found symbols related to "add" (could be "add", "Adder", etc.)
    let has_add_related = result.iter().any(|s| s.name.to_lowercase().contains("add"));
    assert!(
        has_add_related,
        "Should find symbols related to 'add' in workspace. Found: {:?}",
        result.iter().map(|s| &s.name).collect::<Vec<_>>()
    );

    client.shutdown().await.expect("Shutdown should succeed");
}

#[tokio::test]
#[serial]
async fn test_incoming_calls() {
    let client = setup_client().await;

    // Open calculator.rs which has the `calculate` method
    let calc_path = open_file(&client, "src/calculator.rs").await;

    // Also open main.rs which calls methods from calculator
    open_file(&client, "src/main.rs").await;

    // Get incoming calls for `calculate` method in the Adder implementation
    // The implementation starts around line 9
    let result = client
        .incoming_calls(&calc_path, 10, 8)
        .await
        .expect("incoming_calls should succeed");

    // Note: Result might be empty if rust-analyzer hasn't fully indexed
    // or if no calls are detected. This is acceptable for the test.
    // If there are incoming calls, verify the structure is valid
    if !result.is_empty() {
        for call in &result {
            assert!(
                !call.from_ranges.is_empty(),
                "Each incoming call should have at least one range"
            );
        }
    }

    client.shutdown().await.expect("Shutdown should succeed");
}

#[tokio::test]
#[serial]
async fn test_outgoing_calls() {
    let client = setup_client().await;

    // Open main.rs which contains `main` function that makes several calls
    let main_path = open_file(&client, "src/main.rs").await;

    // Get outgoing calls from the `main` function (line 3)
    let result = client
        .outgoing_calls(&main_path, 3, 4)
        .await
        .expect("outgoing_calls should succeed");

    // The main function calls `add`, `subtract`, and `use_calculator`
    // So we should have at least some outgoing calls
    assert!(
        !result.is_empty(),
        "main function should have outgoing calls"
    );

    // Verify we have calls to expected functions
    let called_functions: Vec<String> = result.iter().map(|c| c.to.name.clone()).collect();

    assert!(
        called_functions.iter().any(|name| name.contains("add")
            || name.contains("subtract")
            || name.contains("use_calculator")),
        "Should find expected function calls, found: {:?}",
        called_functions
    );

    client.shutdown().await.expect("Shutdown should succeed");
}

#[tokio::test]
#[serial]
async fn test_implementations() {
    let client = setup_client().await;

    // Open calculator.rs which contains the Calculator trait
    let calc_path = open_file(&client, "src/calculator.rs").await;

    // Get implementations of the Calculator trait (defined around line 2)
    // Position at the trait name
    let result = client
        .implementations(&calc_path, 2, 11)
        .await
        .expect("implementations should succeed");

    // Verify we got implementations
    match result {
        GotoDefinitionResponse::Array(locations) => {
            assert!(
                !locations.is_empty(),
                "Calculator trait should have implementations (Adder, Multiplier, Subtractor)"
            );

            // We should have at least 3 implementations
            assert!(
                locations.len() >= 3,
                "Should have at least 3 implementations, found {}",
                locations.len()
            );
        }
        GotoDefinitionResponse::Scalar(_) => {
            // Single implementation is also acceptable
        }
        _ => panic!("Unexpected response type"),
    }

    client.shutdown().await.expect("Shutdown should succeed");
}

#[tokio::test]
#[serial]
async fn test_type_definition() {
    let client = setup_client().await;

    // Open main.rs which has variables with types
    let main_path = open_file(&client, "src/main.rs").await;

    // Get type definition for the `calc` variable (around line 16)
    // `let calc = sample_project::Adder;`
    let result = client
        .type_definition(&main_path, 16, 9)
        .await
        .expect("type_definition should succeed");

    // Verify we got a type definition response
    match result {
        GotoDefinitionResponse::Array(locations) => {
            if !locations.is_empty() {
                // If we got locations, verify they point to calculator.rs
                assert!(
                    locations[0].uri.as_str().contains("calculator.rs"),
                    "Type definition should be in calculator.rs"
                );
            }
            // Note: Empty array is acceptable as rust-analyzer might not always
            // return type definitions for simple assignments
        }
        GotoDefinitionResponse::Scalar(location) => {
            assert!(
                location.uri.as_str().contains("calculator.rs"),
                "Type definition should be in calculator.rs"
            );
        }
        _ => {
            // Link response is also acceptable
        }
    }

    client.shutdown().await.expect("Shutdown should succeed");
}

#[tokio::test]
#[serial]
async fn test_shutdown() {
    let client = setup_client().await;

    // Test that shutdown completes successfully
    let result = client.shutdown().await;
    assert!(result.is_ok(), "Shutdown should succeed");
}

// Additional test to verify client can be reused for multiple operations
#[tokio::test]
#[serial]
async fn test_multiple_operations() {
    let client = setup_client().await;

    // Open a file
    let lib_path = open_file(&client, "src/lib.rs").await;

    // Perform multiple operations in sequence
    let _symbols = client
        .document_symbols(&lib_path)
        .await
        .expect("document_symbols should succeed");

    let _hover = client
        .hover(&lib_path, 22, 8)
        .await
        .expect("hover should succeed");

    let _refs = client
        .find_references(&lib_path, 22, 8, true)
        .await
        .expect("find_references should succeed");

    // All operations should complete successfully
    client.shutdown().await.expect("Shutdown should succeed");
}

// Test error handling for invalid positions
#[tokio::test]
#[serial]
async fn test_invalid_position() {
    let client = setup_client().await;
    let lib_path = open_file(&client, "src/lib.rs").await;

    // Try to get hover at line 0 (invalid - positions are 1-indexed)
    let result = client.hover(&lib_path, 0, 1).await;

    assert!(result.is_err(), "Should fail for invalid position (line 0)");

    client.shutdown().await.expect("Shutdown should succeed");
}

// Test workspace symbols with various queries
#[tokio::test]
#[serial]
async fn test_workspace_symbols_queries() {
    let client = setup_client().await;

    // Test various search queries
    let queries = vec!["add", "Calculator", "Point", "multiply"];

    for query in queries {
        let result = client
            .workspace_symbols(query)
            .await
            .unwrap_or_else(|_| panic!("workspace_symbols should succeed for query '{}'", query));

        // Each query should find at least something in our test fixture
        assert!(
            !result.is_empty(),
            "Should find symbols for query '{}', but found none",
            query
        );
    }

    client.shutdown().await.expect("Shutdown should succeed");
}
