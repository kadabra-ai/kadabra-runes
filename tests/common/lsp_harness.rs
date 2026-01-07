use kadabra_runes::lsp::client::LspClient;
use std::path::Path;
use std::time::Duration;

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

/// Spawns a new LSP client with the given workspace directory
/// ## Panics
pub async fn spawn_lsp(workspace: &Path) -> LspClient {
    let (init_timeout, request_timeout, index_wait) = (
        Duration::from_secs(120), // 2 minutes for CI initialization
        Duration::from_secs(60),  // 1 minute for CI requests
        Duration::from_secs(8),   // 8 seconds for CI indexing
    );
    let client = LspClient::builder()
        .server_command(find_rust_analyzer())
        .workspace_root(workspace.to_path_buf())
        .init_timeout(init_timeout)
        .request_timeout(request_timeout)
        .build()
        .await
        .expect("Failed to start LSP client");
    // Give rust-analyzer time to fully index the workspace
    tokio::time::sleep(index_wait).await;
    client
}
