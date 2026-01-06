//! Additional type definitions for LSP operations.
//!
//! This module provides helper types and conversions for working with
//! LSP types in the context of code navigation.
//!
//! Note: Functions here are used by the MCP server layer but may appear
//! unused due to macro-generated code.

// Allow dead code warnings for functions used by MCP layer
#![allow(dead_code)]

use lsp_types::{Position, Url};
use std::path::Path;

use crate::error::LspError;

use super::LspResult;

/// Converts a path to an LSP file:// URI.
///
/// This handles both absolute and relative paths, converting them to
/// properly formatted file:// URIs that LSP servers expect.
/// ## Errors
pub fn path_to_url(path: &Path) -> LspResult<Url> {
    // Make path absolute
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|e| {
                LspError::DocumentNotFound(format!("failed to get current directory: {e}"))
            })?
            .join(path)
    };

    // Canonicalize to resolve symlinks and normalize path
    let canonical = absolute.canonicalize().map_err(|e| {
        LspError::DocumentNotFound(format!(
            "failed to canonicalize path '{}': {}",
            path.display(),
            e
        ))
    })?;

    // Use Url::from_file_path which handles platform-specific details
    Url::from_file_path(&canonical)
        .map_err(|()| LspError::DocumentNotFound(format!("invalid path: {}", canonical.display())))
}

/// Extension trait for converting paths to LSP Url.
pub trait PathToUri {
    /// Converts a path to an LSP Url.
    /// ## Errors
    /// `LspError`
    fn to_lsp_uri(&self) -> LspResult<Url>;
}

impl PathToUri for Path {
    fn to_lsp_uri(&self) -> LspResult<Url> {
        path_to_url(self)
    }
}

/// Converts user-facing 1-indexed position to LSP 0-indexed position.
///
/// # Arguments
///
/// * `line` - 1-indexed line number
/// * `column` - 1-indexed column number
///
/// # Returns
///
/// LSP Position (0-indexed)
///
/// # Errors
///
/// Returns error if line or column is 0.
pub fn to_lsp_position(line: u32, column: u32) -> LspResult<Position> {
    if line == 0 {
        return Err(LspError::InvalidPosition { line, column });
    }
    if column == 0 {
        return Err(LspError::InvalidPosition { line, column });
    }
    Ok(Position {
        line: line - 1,
        character: column - 1,
    })
}

/// Converts LSP 0-indexed position to user-facing 1-indexed position.
///
/// # Arguments
///
/// * `position` - LSP Position (0-indexed)
///
/// # Returns
///
/// Tuple of (line, column) both 1-indexed
pub fn from_lsp_position(position: Position) -> (u32, u32) {
    (position.line + 1, position.character + 1)
}

/// Converts an LSP symbol kind to a human-readable string.
pub fn symbol_kind_to_string(kind: lsp_types::SymbolKind) -> &'static str {
    use lsp_types::SymbolKind;
    match kind {
        SymbolKind::FILE => "file",
        SymbolKind::MODULE => "module",
        SymbolKind::NAMESPACE => "namespace",
        SymbolKind::PACKAGE => "package",
        SymbolKind::CLASS => "class",
        SymbolKind::METHOD => "method",
        SymbolKind::PROPERTY => "property",
        SymbolKind::FIELD => "field",
        SymbolKind::CONSTRUCTOR => "constructor",
        SymbolKind::ENUM => "enum",
        SymbolKind::INTERFACE => "interface",
        SymbolKind::FUNCTION => "function",
        SymbolKind::VARIABLE => "variable",
        SymbolKind::CONSTANT => "constant",
        SymbolKind::STRING => "string",
        SymbolKind::NUMBER => "number",
        SymbolKind::BOOLEAN => "boolean",
        SymbolKind::ARRAY => "array",
        SymbolKind::OBJECT => "object",
        SymbolKind::KEY => "key",
        SymbolKind::NULL => "null",
        SymbolKind::ENUM_MEMBER => "enum_member",
        SymbolKind::STRUCT => "struct",
        SymbolKind::EVENT => "event",
        SymbolKind::OPERATOR => "operator",
        SymbolKind::TYPE_PARAMETER => "type_parameter",
        _ => "unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_lsp_position() {
        let pos = to_lsp_position(1, 1).unwrap();
        assert_eq!(pos.line, 0);
        assert_eq!(pos.character, 0);

        let pos = to_lsp_position(10, 5).unwrap();
        assert_eq!(pos.line, 9);
        assert_eq!(pos.character, 4);
    }

    #[test]
    fn test_to_lsp_position_invalid() {
        assert!(to_lsp_position(0, 1).is_err());
        assert!(to_lsp_position(1, 0).is_err());
    }

    #[test]
    fn test_from_lsp_position() {
        let (line, col) = from_lsp_position(Position {
            line: 0,
            character: 0,
        });
        assert_eq!(line, 1);
        assert_eq!(col, 1);

        let (line, col) = from_lsp_position(Position {
            line: 9,
            character: 4,
        });
        assert_eq!(line, 10);
        assert_eq!(col, 5);
    }

    #[test]
    fn test_symbol_kind_to_string() {
        assert_eq!(
            symbol_kind_to_string(lsp_types::SymbolKind::FUNCTION),
            "function"
        );
        assert_eq!(
            symbol_kind_to_string(lsp_types::SymbolKind::STRUCT),
            "struct"
        );
        assert_eq!(
            symbol_kind_to_string(lsp_types::SymbolKind::METHOD),
            "method"
        );
    }

    #[test]
    fn test_path_to_uri() {
        // Create a temporary file for testing
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("kadabra_test.rs");

        // Create the file
        std::fs::write(&temp_file, "// test file").expect("Failed to create temp file");

        // Test the conversion
        let uri = temp_file.to_lsp_uri().unwrap();
        let uri_str = uri.as_str();
        assert!(uri_str.starts_with("file://"));
        assert!(uri_str.contains("kadabra_test.rs"));

        // Clean up
        let _ = std::fs::remove_file(&temp_file);
    }
}
