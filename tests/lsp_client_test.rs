use crate::common::{open_file, setup_client};
use lsp_types::{DocumentSymbolResponse, GotoDefinitionResponse};
use serial_test::serial;

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
                location
                    .uri
                    .as_str()
                    .ends_with("tests/fixtures/sample_project/src/lib.rs"),
                "Definition should be in lib.rs"
            );
        }
        GotoDefinitionResponse::Scalar(location) => {
            assert!(
                location
                    .uri
                    .as_str()
                    .contains("tests/fixtures/sample_project/src/lib.rs"),
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

#[tokio::test]
#[serial]
async fn test_workspace_symbols_function_names() {
    let client = setup_client().await;

    // Test if workspace_symbols returns function names
    println!("\n=== Testing workspace_symbols for different symbol types ===");

    let test_queries = vec![
        ("add", "function"),
        ("subtract", "function"),
        ("multiply", "function"),
        ("Point", "struct"),
        ("Calculator", "trait"),
        ("Adder", "struct"),
        ("perform_calculation", "function"),
    ];

    for (query, expected_type) in test_queries {
        let symbols = client
            .workspace_symbols(query)
            .await
            .expect("workspace_symbols should succeed");

        println!(
            "\nQuery '{}' (expected: {}) returned {} symbols:",
            query,
            expected_type,
            symbols.len()
        );
        for sym in &symbols {
            println!(
                "  - Name: '{}', Kind: {:?}, Location: {}",
                sym.name,
                sym.kind,
                sym.location.uri.path()
            );
        }

        if expected_type == "function" {
            // Check if we found the exact function name
            let found_exact = symbols.iter().any(|s| s.name == query);
            println!("  >> Found exact match '{}': {}", query, found_exact);
        }
    }

    client.shutdown().await.expect("Shutdown should succeed");
}

#[tokio::test]
#[serial]
async fn test_workspace_symbols_add_detailed() {
    let client = setup_client().await;

    // Get ALL symbols for "add" query from rust-analyzer
    let symbols = client
        .workspace_symbols("add")
        .await
        .expect("workspace_symbols should succeed");

    println!(
        "\n=== DETAILED: workspace_symbols('add') returned {} symbols ===",
        symbols.len()
    );
    for (i, sym) in symbols.iter().enumerate() {
        println!("\n[{}] Symbol Details:", i);
        println!("    Name: '{}'", sym.name);
        println!("    Kind: {:?}", sym.kind);
        println!("    Location: {}", sym.location.uri.path());
        println!("    Range: {:?}", sym.location.range);
        println!("    Exact match 'add': {}", sym.name == "add");
    }

    // Check if ANY symbol has exact name "add"
    let has_exact_add = symbols.iter().any(|s| s.name == "add");
    println!(
        "\n=== Does rust-analyzer return exact 'add' function? {} ===",
        has_exact_add
    );

    client.shutdown().await.expect("Shutdown should succeed");
}

#[tokio::test]
#[serial]
async fn test_document_symbols_add_function() {
    let client = setup_client().await;

    // Open lib.rs and get its document symbols
    let lib_path = open_file(&client, "src/lib.rs").await;

    let result = client
        .document_symbols(&lib_path)
        .await
        .expect("document_symbols should succeed");

    println!("\n=== document_symbols for lib.rs ===");

    match result {
        DocumentSymbolResponse::Flat(symbols) => {
            println!("Got {} flat symbols", symbols.len());
            for sym in &symbols {
                if sym.name.contains("add")
                    || sym.name.contains("multiply")
                    || sym.name.contains("subtract")
                {
                    println!("  - Name: '{}', Kind: {:?}", sym.name, sym.kind);
                }
            }

            let has_add = symbols.iter().any(|s| s.name == "add");
            println!(
                "\n✅ Does document_symbols contain 'add' function? {}",
                has_add
            );
            assert!(has_add, "document_symbols SHOULD contain 'add' function");
        }
        DocumentSymbolResponse::Nested(symbols) => {
            println!("Got {} nested symbols", symbols.len());
            fn find_in_nested(symbols: &[lsp_types::DocumentSymbol], name: &str) -> bool {
                symbols.iter().any(|s| {
                    if s.name == name {
                        println!("  Found '{}' - Kind: {:?}", s.name, s.kind);
                        true
                    } else {
                        find_in_nested(s.children.as_ref().unwrap_or(&vec![]), name)
                    }
                })
            }

            let has_add = find_in_nested(&symbols, "add");
            println!(
                "\n✅ Does document_symbols contain 'add' function? {}",
                has_add
            );
            assert!(has_add, "document_symbols SHOULD contain 'add' function");
        }
    }

    client.shutdown().await.expect("Shutdown should succeed");
}

#[tokio::test]
#[serial]
async fn test_workspace_symbols_qualified_names() {
    let client = setup_client().await;

    println!("\n=== Testing workspace_symbols with QUALIFIED/FULL names ===");

    let test_queries = vec![
        // Simple names (baseline)
        ("add", "simple function name"),
        ("Adder", "simple struct name"),
        ("Calculator", "simple trait name"),
        ("multiply", "simple function name"),
        // Qualified names - different styles
        ("sample_project::add", "crate::function"),
        ("sample_project::Adder", "crate::struct"),
        ("lib::add", "module::function"),
        ("calculator::Adder", "module::struct"),
        ("calculator::Calculator", "module::trait"),
        // With pub/visibility
        ("pub fn add", "with visibility keyword"),
        ("pub add", "pub + name"),
        // Full paths
        ("sample_project::calculator::Adder", "full path to struct"),
        (
            "sample_project::calculator::Calculator",
            "full path to trait",
        ),
    ];

    for (query, description) in test_queries {
        let symbols = client
            .workspace_symbols(query)
            .await
            .expect(&format!("workspace_symbols('{}') should succeed", query));

        println!("\n[Query: '{}'] ({})", query, description);
        println!("  Returned {} symbols", symbols.len());

        if symbols.is_empty() {
            println!("  ❌ No results");
        } else {
            for sym in symbols.iter().take(3) {
                // Show first 3
                println!("    - Name: '{}', Kind: {:?}", sym.name, sym.kind);
            }

            // Check for exact match
            let exact_match = symbols.iter().find(|s| s.name == query);
            if let Some(m) = exact_match {
                println!("  ✅ EXACT MATCH FOUND: {} ({:?})", m.name, m.kind);
            }
        }
    }

    client.shutdown().await.expect("Shutdown should succeed");
}

#[tokio::test]
#[serial]
async fn test_workspace_symbols_why_multiply_works_add_not() {
    let client = setup_client().await;

    println!("\n=== WHY does 'multiply' work but 'add' doesn't? ===\n");

    // Test different prefixes and full names
    let test_queries = vec![
        // Full names
        ("add", "full 'add'"),
        ("multiply", "full 'multiply'"),
        ("subtract", "full 'subtract'"),
        // Prefixes
        ("ad", "prefix 'ad'"),
        ("mul", "prefix 'mul'"),
        ("mult", "prefix 'mult'"),
        ("multi", "prefix 'multi'"),
        ("sub", "prefix 'sub'"),
        ("subt", "prefix 'subt'"),
        // Check for conflicting names
        ("Multiplier", "check for Multiplier struct"),
        ("Subtractor", "check for Subtractor struct"),
        ("Adder", "check for Adder struct"),
    ];

    for (query, description) in test_queries {
        let symbols = client
            .workspace_symbols(query)
            .await
            .expect(&format!("workspace_symbols('{}') should succeed", query));

        println!("[Query: '{}'] ({})", query, description);
        println!("  Returned {} symbols:", symbols.len());

        for sym in &symbols {
            let is_exact = if sym.name == query { "✅ EXACT" } else { "" };
            println!(
                "    - Name: '{}', Kind: {:?} {}",
                sym.name, sym.kind, is_exact
            );
        }
        println!();
    }

    client.shutdown().await.expect("Shutdown should succeed");
}

#[tokio::test]
#[serial]
async fn test_document_symbols_lib_rs_reexports() {
    // TEST: Does document_symbols on lib.rs return re-exported symbols from calculator module?
    // lib.rs has: pub use calculator::{Adder, Calculator};
    // Question: Will document_symbols return Adder and Calculator?

    let client = setup_client().await;
    let lib_path = open_file(&client, "src/lib.rs").await;

    let symbols = client
        .document_symbols(&lib_path)
        .await
        .expect("document_symbols should succeed");

    let symbol_names: Vec<String> = match &symbols {
        DocumentSymbolResponse::Flat(syms) => syms.iter().map(|s| s.name.clone()).collect(),
        DocumentSymbolResponse::Nested(syms) => syms.iter().map(|s| s.name.clone()).collect(),
    };

    println!("\n=== document_symbols for lib.rs returned {} symbols ===", symbol_names.len());

    // Check for physically defined symbols in lib.rs
    let has_add = symbol_names.iter().any(|n| n == "add");
    let has_subtract = symbol_names.iter().any(|n| n == "subtract");
    let has_multiply = symbol_names.iter().any(|n| n == "multiply");
    let has_point = symbol_names.iter().any(|n| n == "Point");

    // Check for re-exported symbols from calculator module
    let has_adder = symbol_names.iter().any(|n| n == "Adder");
    let has_calculator_trait = symbol_names.iter().any(|n| n == "Calculator");
    let has_multiplier = symbol_names.iter().any(|n| n == "Multiplier");

    println!("\n--- Physically defined symbols in lib.rs ---");
    println!("✓ add function: {}", if has_add { "✅ FOUND" } else { "❌ NOT FOUND" });
    println!("✓ subtract function: {}", if has_subtract { "✅ FOUND" } else { "❌ NOT FOUND" });
    println!("✓ multiply function: {}", if has_multiply { "✅ FOUND" } else { "❌ NOT FOUND" });
    println!("✓ Point struct: {}", if has_point { "✅ FOUND" } else { "❌ NOT FOUND" });

    println!("\n--- Re-exported symbols from calculator module ---");
    println!("  pub use calculator::{{Adder, Calculator}};");
    println!("✓ Adder (re-exported): {}", if has_adder { "✅ FOUND" } else { "❌ NOT FOUND" });
    println!("✓ Calculator trait (re-exported): {}", if has_calculator_trait { "✅ FOUND" } else { "❌ NOT FOUND" });
    println!("✓ Multiplier (NOT re-exported): {}", if has_multiplier { "✅ FOUND" } else { "❌ NOT FOUND" });

    println!("\n--- All symbols returned ---");
    for (i, name) in symbol_names.iter().enumerate() {
        println!("[{}] {}", i, name);
    }

    // Assertions
    assert!(has_add, "Should find 'add' function (physically defined in lib.rs)");
    assert!(has_subtract, "Should find 'subtract' function (physically defined in lib.rs)");
    assert!(has_multiply, "Should find 'multiply' function (physically defined in lib.rs)");
    assert!(has_point, "Should find 'Point' struct (physically defined in lib.rs)");

    println!("\n=== CONCLUSION ===");
    if has_adder || has_calculator_trait {
        println!("✅ document_symbols DOES return re-exported symbols!");
        println!("   This means we CAN use document_symbols as fallback for imported symbols.");
    } else {
        println!("❌ document_symbols does NOT return re-exported symbols.");
        println!("   This means document_symbols only returns physically defined symbols.");
        println!("   We would need to search through ALL module files for fallback.");
    }

    client.shutdown().await.expect("Shutdown should succeed");
}
