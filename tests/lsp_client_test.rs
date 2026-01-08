//! End-to-end tests for MCP server tools.
//!
//! These tests validate the complete MCP tool interface by invoking tools
//! through the MCP server and verifying responses.
//!
//! To run these tests:
//! ```bash
//! # Run all MCP tool tests
//! cargo test --test lsp_client_test
//!
//! # Run with debug output
//! RUST_LOG=debug cargo test --test lsp_client_test -- --nocapture
//!
//! # Run specific test
//! cargo test --test lsp_client_test test_goto_definition
//! ```
mod common;
use common::temp_workspace::TestWorkspace;
use lsp_types::{DocumentSymbolResponse, GotoDefinitionResponse, SymbolKind};

#[tokio::test]
async fn goto_definition_add() {
    let ws = TestWorkspace::builder()
        .fixture(&common::comprehensive_fixture())
        .open_all_files()
        .build()
        .await;

    // Go to definition of `add` function call on line 7 in main.rs
    // The call is: `let result = add(x, y);`
    // Position at the start of 'add' (1-indexed: line 7, column 18)
    let result = ws
        .lsp()
        .goto_definition(&ws.apath("src/main.rs"), 7, 18)
        .await
        .expect("goto_definition should succeed");

    // Verify we got a response pointing to lib.rs
    match result {
        GotoDefinitionResponse::Array(locations) => {
            assert!(
                !locations.is_empty(),
                "Should find definition location for 'add' function"
            );
            assert!(
                locations[0].uri.path().ends_with("src/lib.rs"),
                "Definition should be in lib.rs"
            );
        }
        GotoDefinitionResponse::Scalar(location) => {
            assert!(
                location.uri.path().ends_with("src/lib.rs"),
                "Definition should be in lib.rs"
            );
        }
        _ => panic!("Unexpected response type"),
    }

    ws.lsp().shutdown().await.expect("Shutdown should succeed");
}

#[tokio::test]
async fn test_goto_definition() {
    let ws = TestWorkspace::builder()
        .fixture(&common::comprehensive_fixture())
        .open_all_files()
        .build()
        .await;

    // Go to definition of `add` function call on line 7 in main.rs
    let result = ws
        .lsp()
        .goto_definition(&ws.apath("src/main.rs"), 7, 18)
        .await
        .expect("goto_definition should succeed");

    match result {
        GotoDefinitionResponse::Array(locations) => {
            assert!(
                !locations.is_empty(),
                "Should find definition location for 'add' function"
            );
            assert!(
                locations[0].uri.path().ends_with("src/lib.rs"),
                "Definition should be in lib.rs"
            );
        }
        GotoDefinitionResponse::Scalar(location) => {
            assert!(
                location.uri.path().ends_with("src/lib.rs"),
                "Definition should be in lib.rs"
            );
        }
        _ => panic!("Unexpected response type"),
    }

    ws.lsp().shutdown().await.expect("Shutdown should succeed");
}

#[tokio::test]
async fn test_find_references() {
    let ws = TestWorkspace::builder()
        .fixture(&common::comprehensive_fixture())
        .open_all_files()
        .build()
        .await;

    // Find references to `add` function (line 22 in lib.rs: "pub fn add")
    let result = ws
        .lsp()
        .find_references(&ws.apath("src/lib.rs"), 22, 12, true)
        .await
        .expect("find_references should succeed");

    assert!(
        !result.is_empty(),
        "Should find at least one reference to 'add' function"
    );

    // Verify we have at least some references (in lib.rs where it's defined/re-exported)
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
        files.iter().any(|f| f.contains("lib.rs")),
        "Should have references in lib.rs, found: {:?}",
        files
    );

    ws.lsp().shutdown().await.expect("Shutdown should succeed");
}

#[tokio::test]
async fn test_hover() {
    let ws = TestWorkspace::builder()
        .fixture(&common::comprehensive_fixture())
        .open_all_files()
        .build()
        .await;

    // Get hover info for `add` function (line 22: "pub fn add")
    let result = ws
        .lsp()
        .hover(&ws.apath("src/lib.rs"), 22, 12)
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

    ws.lsp().shutdown().await.expect("Shutdown should succeed");
}

#[tokio::test]
async fn test_document_symbols() {
    let ws = TestWorkspace::builder()
        .fixture(&common::comprehensive_fixture())
        .open_all_files()
        .build()
        .await;

    let result = ws
        .lsp()
        .document_symbols(&ws.apath("src/lib.rs"))
        .await
        .expect("document_symbols should succeed");

    // Verify we got symbols
    match result {
        DocumentSymbolResponse::Flat(symbols) => {
            assert!(!symbols.is_empty(), "Should find symbols in lib.rs");
            let has_add = symbols.iter().any(|s| s.name == "add");
            assert!(has_add, "Should find 'add' function in symbols");
        }
        DocumentSymbolResponse::Nested(symbols) => {
            assert!(!symbols.is_empty(), "Should find symbols in lib.rs");

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

    ws.lsp().shutdown().await.expect("Shutdown should succeed");
}

#[tokio::test]
async fn test_workspace_symbols() {
    let ws = TestWorkspace::builder()
        .fixture(&common::comprehensive_fixture())
        .open_all_files()
        .build()
        .await;

    let result = ws
        .lsp()
        .workspace_symbols("add")
        .await
        .expect("workspace_symbols should succeed");

    assert!(
        !result.is_empty(),
        "Should find symbols matching 'add' in workspace"
    );

    let has_add_related = result.iter().any(|s| s.name.to_lowercase().contains("add"));
    assert!(
        has_add_related,
        "Should find symbols related to 'add' in workspace. Found: {:?}",
        result.iter().map(|s| &s.name).collect::<Vec<_>>()
    );

    ws.lsp().shutdown().await.expect("Shutdown should succeed");
}

#[tokio::test]
async fn test_incoming_calls() {
    let ws = TestWorkspace::builder()
        .fixture(&common::comprehensive_fixture())
        .open_all_files()
        .build()
        .await;

    // Get incoming calls for `calculate` method in Adder impl (line 10 in calculator.rs)
    let result = ws
        .lsp()
        .incoming_calls(&ws.apath("src/calculator.rs"), 10, 8)
        .await
        .expect("incoming_calls should succeed");

    // Result might be empty if no calls detected
    if !result.is_empty() {
        for call in &result {
            assert!(
                !call.from_ranges.is_empty(),
                "Each incoming call should have at least one range"
            );
        }
    }

    ws.lsp().shutdown().await.expect("Shutdown should succeed");
}

#[tokio::test]
async fn test_outgoing_calls() {
    let ws = TestWorkspace::builder()
        .fixture(&common::comprehensive_fixture())
        .open_all_files()
        .build()
        .await;

    // Get outgoing calls from `main` function (line 3)
    let result = ws
        .lsp()
        .outgoing_calls(&ws.apath("src/main.rs"), 3, 4)
        .await
        .expect("outgoing_calls should succeed");

    assert!(
        !result.is_empty(),
        "main function should have outgoing calls"
    );

    let called_functions: Vec<String> = result.iter().map(|c| c.to.name.clone()).collect();

    assert!(
        called_functions.iter().any(|name| name.contains("add")
            || name.contains("subtract")
            || name.contains("use_calculator")),
        "Should find expected function calls, found: {:?}",
        called_functions
    );

    ws.lsp().shutdown().await.expect("Shutdown should succeed");
}

#[tokio::test]
async fn test_implementations() {
    let ws = TestWorkspace::builder()
        .fixture(&common::comprehensive_fixture())
        .open_all_files()
        .build()
        .await;

    // Get implementations of Calculator trait (line 2)
    let result = ws
        .lsp()
        .implementations(&ws.apath("src/calculator.rs"), 2, 16)
        .await
        .expect("implementations should succeed");

    match result {
        GotoDefinitionResponse::Array(locations) => {
            assert!(
                !locations.is_empty(),
                "Calculator trait should have implementations"
            );
            assert!(
                locations.len() >= 3,
                "Should have at least 3 implementations, found {}",
                locations.len()
            );
        }
        GotoDefinitionResponse::Scalar(_) => {
            // Single implementation is acceptable
        }
        _ => panic!("Unexpected response type"),
    }

    ws.lsp().shutdown().await.expect("Shutdown should succeed");
}

#[tokio::test]
async fn test_type_definition() {
    let ws = TestWorkspace::builder()
        .fixture(&common::comprehensive_fixture())
        .open_all_files()
        .build()
        .await;

    // Get type definition for the `calc` variable (line 18)
    let result = ws
        .lsp()
        .type_definition(&ws.apath("src/main.rs"), 18, 9)
        .await
        .expect("type_definition should succeed");

    match result {
        GotoDefinitionResponse::Array(locations) => {
            if !locations.is_empty() {
                assert!(
                    locations[0].uri.path().contains("calculator.rs"),
                    "Type definition should be in calculator.rs"
                );
            }
        }
        GotoDefinitionResponse::Scalar(location) => {
            assert!(
                location.uri.path().contains("calculator.rs"),
                "Type definition should be in calculator.rs"
            );
        }
        _ => {}
    }

    ws.lsp().shutdown().await.expect("Shutdown should succeed");
}

#[tokio::test]
async fn test_shutdown() {
    let ws = TestWorkspace::builder()
        .fixture(&common::comprehensive_fixture())
        .build()
        .await;

    let result = ws.lsp().shutdown().await;
    assert!(result.is_ok(), "Shutdown should succeed");
}

#[tokio::test]
async fn test_multiple_operations() {
    let ws = TestWorkspace::builder()
        .fixture(&common::comprehensive_fixture())
        .open_all_files()
        .build()
        .await;

    let lib_path = ws.apath("src/lib.rs");

    let _symbols = ws
        .lsp()
        .document_symbols(&lib_path)
        .await
        .expect("document_symbols should succeed");

    let _hover = ws
        .lsp()
        .hover(&lib_path, 22, 8)
        .await
        .expect("hover should succeed");

    let _refs = ws
        .lsp()
        .find_references(&lib_path, 22, 8, true)
        .await
        .expect("find_references should succeed");

    ws.lsp().shutdown().await.expect("Shutdown should succeed");
}

#[tokio::test]
async fn test_invalid_position() {
    let ws = TestWorkspace::builder()
        .fixture(&common::comprehensive_fixture())
        .open_all_files()
        .build()
        .await;

    let result = ws.lsp().hover(&ws.apath("src/lib.rs"), 0, 1).await;
    assert!(result.is_err(), "Should fail for invalid position (line 0)");

    ws.lsp().shutdown().await.expect("Shutdown should succeed");
}

#[tokio::test]
async fn test_workspace_symbols_queries() {
    let ws = TestWorkspace::builder()
        .fixture(&common::comprehensive_fixture())
        .open_all_files()
        .build()
        .await;

    let queries = vec!["add", "Calculator", "Point", "multiply"];

    for query in queries {
        let result =
            ws.lsp().workspace_symbols(query).await.unwrap_or_else(|_| {
                panic!("workspace_symbols should succeed for query '{}'", query)
            });

        assert!(
            !result.is_empty(),
            "Should find symbols for query '{}', but found none",
            query
        );
    }

    ws.lsp().shutdown().await.expect("Shutdown should succeed");
}

#[tokio::test]
async fn test_workspace_symbols_function_names() {
    let ws = TestWorkspace::builder()
        .fixture(&common::comprehensive_fixture())
        .open_all_files()
        .build()
        .await;

    let test_queries = vec![
        ("subtract", "function", false),
        ("multiply", "function", true),
        ("Point", "struct", true),
        ("Calculator", "trait", true),
        ("Adder", "struct", true),
        ("perform_calculation", "function", true),
    ];

    for (query, expected_type, to_match) in test_queries {
        let symbols = ws
            .lsp()
            .workspace_symbols(query)
            .await
            .expect("workspace_symbols should succeed");

        if expected_type == "function" {
            let found_exact = symbols.iter().any(|s| s.name == query);
            assert_eq!(
                found_exact, to_match,
                "Symbol '{}' does not match expected",
                query
            );
        }
    }

    ws.lsp().shutdown().await.expect("Shutdown should succeed");
}

#[tokio::test]
async fn test_workspace_symbols_add_detailed() {
    let ws = TestWorkspace::builder()
        .fixture(&common::comprehensive_fixture())
        .open_all_files()
        .build()
        .await;

    let symbols = ws
        .lsp()
        .workspace_symbols("add")
        .await
        .expect("workspace_symbols should succeed");

    let has_exact_add = symbols.iter().any(|s| s.name == "add");
    // NOTE: This is absurd, but it works like that
    assert!(!has_exact_add, "Should NOT find exact 'add' symbol");

    ws.lsp().shutdown().await.expect("Shutdown should succeed");
}

#[tokio::test]
async fn test_document_symbols_add_function() {
    let ws = TestWorkspace::builder()
        .fixture(&common::comprehensive_fixture())
        .open_all_files()
        .build()
        .await;

    let result = ws
        .lsp()
        .document_symbols(&ws.apath("src/lib.rs"))
        .await
        .expect("document_symbols should succeed");

    match result {
        DocumentSymbolResponse::Flat(symbols) => {
            let has_add = symbols
                .iter()
                .any(|s| s.name == "add" && s.kind == SymbolKind::FUNCTION);
            assert!(has_add, "document_symbols SHOULD contain 'add' function");
        }
        DocumentSymbolResponse::Nested(symbols) => {
            fn find_in_nested(symbols: &[lsp_types::DocumentSymbol], name: &str) -> bool {
                symbols.iter().any(|s| {
                    if s.name == name {
                        true
                    } else {
                        find_in_nested(s.children.as_ref().unwrap_or(&vec![]), name)
                    }
                })
            }

            let has_add = find_in_nested(&symbols, "add");
            assert!(has_add, "document_symbols SHOULD contain 'add' function");
        }
    }

    ws.lsp().shutdown().await.expect("Shutdown should succeed");
}

#[tokio::test]
async fn test_workspace_symbols_qualified_names() {
    let ws = TestWorkspace::builder()
        .fixture(&common::comprehensive_fixture())
        .open_all_files()
        .build()
        .await;

    let test_queries = vec![
        ("Adder", "simple struct name", true),
        ("Calculator", "simple trait name", true),
        ("multiply", "simple function name", true),
        ("test_project::add", "crate::function", false),
        ("test_project::Adder", "crate::struct", false),
        ("lib::add", "module::function", false),
        ("calculator::Adder", "module::struct", false),
        ("calculator::Calculator", "module::trait", false),
        ("pub fn add", "with visibility keyword", false),
        ("pub add", "pub + name", false),
        (
            "test_project::calculator::Adder",
            "full path to struct",
            false,
        ),
        (
            "test_project::calculator::Calculator",
            "full path to trait",
            false,
        ),
    ];

    for (query, _description, expected) in test_queries {
        let symbols = ws
            .lsp()
            .workspace_symbols(query)
            .await
            .unwrap_or_else(|_| panic!("workspace_symbols('{}') should succeed", query));

        if symbols.is_empty() && expected {
            panic!("Symbol '{}' does not match expected", query);
        } else {
            let exact_match = symbols.iter().any(|s| s.name == query);
            assert_eq!(
                exact_match, expected,
                "No exact match found for query '{}'",
                query
            );
        }
    }

    ws.lsp().shutdown().await.expect("Shutdown should succeed");
}

#[tokio::test]
async fn test_document_symbols_lib_rs_reexports() {
    let ws = TestWorkspace::builder()
        .fixture(&common::comprehensive_fixture())
        .open_all_files()
        .build()
        .await;

    let symbols = ws
        .lsp()
        .document_symbols(&ws.apath("src/lib.rs"))
        .await
        .expect("document_symbols should succeed");

    let symbol_names: Vec<String> = match &symbols {
        DocumentSymbolResponse::Flat(syms) => syms.iter().map(|s| s.name.clone()).collect(),
        DocumentSymbolResponse::Nested(syms) => syms.iter().map(|s| s.name.clone()).collect(),
    };

    // Check for physically defined symbols in lib.rs
    let has_add = symbol_names.iter().any(|n| n == "add");
    let has_subtract = symbol_names.iter().any(|n| n == "subtract");
    let has_multiply = symbol_names.iter().any(|n| n == "multiply");
    let has_point = symbol_names.iter().any(|n| n == "Point");

    // Check for re-exported symbols from calculator module
    let has_adder = symbol_names.iter().any(|n| n == "Adder");
    let has_calculator_trait = symbol_names.iter().any(|n| n == "Calculator");
    let has_multiplier = symbol_names.iter().any(|n| n == "Multiplier");
    assert!(
        !has_adder,
        "Should NOT find 'Adder' (re-exported from calculator module)"
    );
    assert!(
        !has_calculator_trait,
        "Should NOT find 'Calculator' trait (re-exported from calculator module)"
    );
    assert!(
        !has_multiplier,
        "Should NOT find 'Multiplier' (NOT re-exported from calculator module)"
    );
    // Assertions
    assert!(
        has_add,
        "Should find 'add' function (physically defined in lib.rs)"
    );
    assert!(
        has_subtract,
        "Should find 'subtract' function (physically defined in lib.rs)"
    );
    assert!(
        has_multiply,
        "Should find 'multiply' function (physically defined in lib.rs)"
    );
    assert!(
        has_point,
        "Should find 'Point' struct (physically defined in lib.rs)"
    );

    ws.lsp().shutdown().await.expect("Shutdown should succeed");
}
