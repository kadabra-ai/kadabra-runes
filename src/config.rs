//! Configuration helper for MCP clients.
//!
//! This module provides functionality to create/update `.mcp.json` in the project root
//! with kadabra-runes configuration.

use anyhow::{Context, Result, bail};
use serde_json::{Value, json};
use std::fs;
use std::path::Path;

/// Configure kadabra-runes by creating/updating .mcp.json in current directory
///
/// ## Errors
/// Returns an error if:
/// - File I/O fails
/// - Existing .mcp.json contains invalid JSON
/// - kadabra-runes is already configured
/// ## Panics
pub fn configure() -> Result<()> {
    let config_file = Path::new(".mcp.json");

    // Read existing config or create new
    let mut config: Value = if config_file.exists() {
        let content = fs::read_to_string(config_file).context("failed to read .mcp.json")?;
        serde_json::from_str(&content).context("failed to parse .mcp.json - invalid JSON")?
    } else {
        json!({})
    };

    // Ensure config is an object
    if !config.is_object() {
        config = json!({});
    }
    let config_obj = config.as_object_mut().unwrap();

    // Ensure mcpServers object exists
    if !config_obj.contains_key("mcpServers") {
        config_obj.insert("mcpServers".to_string(), json!({}));
    }

    // Get or create mcpServers object
    let mcp_servers = config_obj.get_mut("mcpServers").unwrap();
    if !mcp_servers.is_object() {
        *mcp_servers = json!({});
    }

    // Check if kadabra-runes is already configured
    if mcp_servers.get("kadabra-runes").is_some() {
        bail!(
            "kadabra-runes is already configured in .mcp.json\n\n\
            To reconfigure, first remove the existing entry, then run:\n  \
            kadabra-runes config"
        );
    }

    // Add kadabra-runes configuration
    let mcp_servers_obj = mcp_servers.as_object_mut().unwrap();
    mcp_servers_obj.insert(
        "kadabra-runes".to_string(),
        json!({
            "command": "kadabra-runes",
            "args": ["--workspace", "."]
        }),
    );

    // Write atomically (temp file + rename)
    let temp_file = config_file.with_extension("tmp");
    let json_str = serde_json::to_string_pretty(&config).context("failed to serialize JSON")?;

    fs::write(&temp_file, json_str).context("failed to write temporary config file")?;

    fs::rename(&temp_file, config_file).context("failed to rename temporary config file")?;

    // Print success message
    println!("\n{}", "=".repeat(60));
    println!("✓ Created .mcp.json");
    println!("{}", "=".repeat(60));
    println!("\nConfiguration complete! 🎉\n");
    println!("Next steps:");
    println!("  1. The .mcp.json file has been created");
    println!("  2. Restart your MCP client if it's running");
    println!("  3. kadabra-runes will start automatically");
    println!("  4. Try: \"Find the main function in this project\"\n");
    println!("Note: You can commit .mcp.json to git to share this");
    println!("      configuration with your team.\n");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::fs;

    #[test]
    #[serial]
    fn test_configure_creates_new_file() {
        let original_dir = std::env::current_dir().unwrap();
        let temp_dir =
            std::env::temp_dir().join(format!("kadabra-test-new-{}", std::process::id()));

        // Cleanup any leftover test directory
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        std::env::set_current_dir(&temp_dir).unwrap();

        configure().unwrap();

        let content = fs::read_to_string(".mcp.json").unwrap();
        let config: Value = serde_json::from_str(&content).unwrap();

        assert!(config["mcpServers"]["kadabra-runes"].is_object());
        assert_eq!(
            config["mcpServers"]["kadabra-runes"]["command"],
            "kadabra-runes"
        );

        // Restore original directory before cleanup
        std::env::set_current_dir(&original_dir).unwrap();
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    #[serial]
    fn test_configure_fails_if_already_exists() {
        let original_dir = std::env::current_dir().unwrap();
        let temp_dir =
            std::env::temp_dir().join(format!("kadabra-test-exists-{}", std::process::id()));

        // Cleanup any leftover test directory
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        std::env::set_current_dir(&temp_dir).unwrap();

        configure().unwrap();

        let result = configure();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("already configured")
        );

        // Restore original directory before cleanup
        std::env::set_current_dir(&original_dir).unwrap();
        fs::remove_dir_all(&temp_dir).unwrap();
    }
}
