//! Common test helpers and utilities.

#![allow(dead_code)]

pub mod lsp_harness;
pub mod temp_workspace;

use kadabra_runes::lsp::client::LspClient;
use std::path::PathBuf;
use std::time::Duration;

// Re-export for convenience
pub use temp_workspace::TestWorkspace;

/// Helper to get the fixture project path
pub fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sample_project")
}

/// Helper to find rust-analyzer executable
pub fn find_rust_analyzer() -> String {
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
pub async fn setup_client() -> LspClient {
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
pub async fn open_file(client: &LspClient, relative_path: &str) -> PathBuf {
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
