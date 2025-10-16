//! Utilities for formatting options in a tabular layout with aligned columns.
//!
//! This module provides functionality to display multi-field data (like file listings,
//! server information, or project metadata) with properly aligned columns.
//!
//! # Column Configuration
//!
//! Columns are configured using [`ColumnConfig`], which specifies:
//! - **Alignment**: Left or right alignment for the column content
//! - **Separator**: Text that appears after the column (before the next column)
//!
//! # Separators
//!
//! - **Default separator**: Use `ColumnConfig::new(alignment)` to use a single space `" "`
//! - **Custom separator**: Use `ColumnConfig::new_with_separator(sep, alignment)` for custom separators
//!
//! # Example
//!
//! ```
//! use inquire::tabular::{format_as_table, ColumnConfig, ColumnAlignment};
//!
//! let data = vec![
//!     "file1.txt: 1.2 KB, 2025-01-15".to_string(),
//!     "document.pdf: 3.4 MB, 2025-01-16".to_string(),
//! ];
//!
//! let columns = vec![
//!     ColumnConfig::new_with_separator(": ", ColumnAlignment::Left),  // Filename + ": "
//!     ColumnConfig::new_with_separator(", ", ColumnAlignment::Right), // Size + ", "
//!     ColumnConfig::new(ColumnAlignment::Left),                       // Date (last column)
//! ];
//!
//! let formatted = format_as_table(&data, &columns);
//! // Results in aligned columns:
//! // "file1.txt    : 1.2 KB, 2025-01-15"
//! // "document.pdf : 3.4 MB, 2025-01-16"
//! ```

use unicode_width::UnicodeWidthStr;

/// Column alignment options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnAlignment {
    /// Align column content to the left
    Left,
    /// Align column content to the right
    Right,
}

/// Configuration for a single column in tabular formatting
#[derive(Debug, Clone)]
pub struct ColumnConfig {
    /// The separator that follows this column (appears after the column content)
    pub separator: String,
    /// How to align content within this column
    pub alignment: ColumnAlignment,
}

impl ColumnConfig {
    /// Default separator used between columns
    pub const DEFAULT_SEPARATOR: &'static str = " ";

    /// Creates a new column configuration with default separator (a single space)
    ///
    /// # Arguments
    /// * `alignment` - How to align the column content
    ///
    /// # Example
    /// ```
    /// use inquire::tabular::{ColumnConfig, ColumnAlignment};
    ///
    /// let col = ColumnConfig::new(ColumnAlignment::Left);
    /// // This will use " " as the separator
    /// ```
    #[must_use]
    pub fn new(alignment: ColumnAlignment) -> Self {
        Self {
            separator: Self::DEFAULT_SEPARATOR.to_string(),
            alignment,
        }
    }

    /// Creates a new column configuration with a custom separator
    ///
    /// # Arguments
    /// * `separator` - The text that separates this column from the next (e.g., ": " or ", ")
    /// * `alignment` - How to align the column content
    ///
    /// # Example
    /// ```
    /// use inquire::tabular::{ColumnConfig, ColumnAlignment};
    ///
    /// let col = ColumnConfig::new_with_separator(": ", ColumnAlignment::Left);
    /// ```
    pub fn new_with_separator(separator: impl Into<String>, alignment: ColumnAlignment) -> Self {
        Self {
            separator: separator.into(),
            alignment,
        }
    }

    /// Sets or replaces the separator
    ///
    /// This method allows you to set or change the separator on a column configuration,
    /// which is useful when you want to reuse a configuration with different separators
    /// or when building configurations using a builder pattern.
    ///
    /// # Arguments
    /// * `separator` - The new separator text
    ///
    /// # Example
    /// ```
    /// use inquire::tabular::{ColumnConfig, ColumnAlignment};
    ///
    /// // Set separator on a config with default separator
    /// let col = ColumnConfig::new(ColumnAlignment::Left)
    ///     .separator(": ");  // Replace " " with ": "
    ///
    /// // Or update an existing config
    /// let mut col = ColumnConfig::new_with_separator(", ", ColumnAlignment::Right);
    /// col = col.separator(" | ");  // Change from ", " to " | "
    /// ```
    #[must_use]
    pub fn separator(mut self, separator: impl Into<String>) -> Self {
        self.separator = separator.into();
        self
    }
}

/// Splits a string into columns based on the provided separators
///
/// # Arguments
/// * `text` - The text to split into columns
/// * `columns` - Column configurations including separators
///
/// # Returns
/// A vector of column contents (without the separators)
fn split_into_columns(text: &str, columns: &[ColumnConfig]) -> Vec<String> {
    let mut result = Vec::new();
    let mut remaining = text;

    for (idx, col_config) in columns.iter().enumerate() {
        if remaining.is_empty() {
            result.push(String::new());
            continue;
        }

        // For the last column, take everything that's left
        if idx == columns.len() - 1 {
            result.push(remaining.to_string());
            break;
        }

        // Find the separator
        if let Some(pos) = remaining.find(&col_config.separator) {
            result.push(remaining[..pos].to_string());
            remaining = &remaining[pos + col_config.separator.len()..];
        } else {
            // Separator not found, put the rest in this column
            result.push(remaining.to_string());
            remaining = "";
        }
    }

    // Fill remaining columns with empty strings if we ran out of text
    while result.len() < columns.len() {
        result.push(String::new());
    }

    result
}

/// Calculates the maximum width for each column across all rows
///
/// # Arguments
/// * `rows` - All rows split into columns
///
/// # Returns
/// A vector of maximum widths for each column
fn calculate_column_widths(rows: &[Vec<String>]) -> Vec<usize> {
    if rows.is_empty() {
        return Vec::new();
    }

    let num_columns = rows.first().map(Vec::len).unwrap_or(0);
    let mut max_widths = vec![0; num_columns];

    for row in rows {
        for (col_idx, cell) in row.iter().enumerate() {
            let width = cell.width();
            if let Some(current_max) = max_widths.get_mut(col_idx) {
                if width > *current_max {
                    *current_max = width;
                }
            }
        }
    }

    max_widths
}

/// Pads a string to the specified width according to the alignment
///
/// # Arguments
/// * `text` - The text to pad
/// * `width` - The target width
/// * `alignment` - How to align the text
///
/// # Returns
/// The padded string
fn pad_string(text: &str, width: usize, alignment: ColumnAlignment) -> String {
    let current_width = text.width();

    if current_width >= width {
        return text.to_string();
    }

    let padding = width - current_width;
    match alignment {
        ColumnAlignment::Left => format!("{}{}", text, " ".repeat(padding)),
        ColumnAlignment::Right => format!("{}{}", " ".repeat(padding), text),
    }
}

/// Formats multiple strings into aligned tabular columns
///
/// # Arguments
/// * `texts` - The texts to format
/// * `columns` - Column configurations
///
/// # Returns
/// A vector of formatted strings with aligned columns
///
/// # Example
/// ```
/// use inquire::tabular::{format_as_table, ColumnConfig, ColumnAlignment};
///
/// let options = vec![
///     "copy_current_location: 898.95 KB (2025-10-12 15:41), /path1".to_string(),
///     "summary-gen: 211.29 MB (2025-10-13 20:04), /path2".to_string(),
///     "rona: 1.26 GB (2025-10-14 18:29), /path3".to_string(),
/// ];
///
/// let columns = vec![
///     ColumnConfig::new_with_separator(": ", ColumnAlignment::Left),
///     ColumnConfig::new(ColumnAlignment::Right),  // Uses default " " separator
///     ColumnConfig::new_with_separator(", ", ColumnAlignment::Left),
/// ];
///
/// let formatted = format_as_table(&options, &columns);
/// ```
#[must_use]
pub fn format_as_table(texts: &[String], columns: &[ColumnConfig]) -> Vec<String> {
    if texts.is_empty() || columns.is_empty() {
        return texts.to_vec();
    }

    // Split all texts into columns
    let rows: Vec<Vec<String>> = texts
        .iter()
        .map(|text| split_into_columns(text, columns))
        .collect();

    // Calculate maximum width for each column
    let max_widths = calculate_column_widths(&rows);

    // Format each row
    rows.iter()
        .map(|row| {
            let mut formatted = String::new();
            for (col_idx, cell) in row.iter().enumerate() {
                if let Some(col_config) = columns.get(col_idx) {
                    // Pad the cell content
                    let padded = if col_idx == columns.len() - 1 {
                        // Don't pad the last column
                        cell.clone()
                    } else if let Some(&width) = max_widths.get(col_idx) {
                        pad_string(cell, width, col_config.alignment)
                    } else {
                        cell.clone()
                    };

                    formatted.push_str(&padded);

                    // Add separator if not the last column
                    if col_idx < columns.len() - 1 {
                        formatted.push_str(&col_config.separator);
                    }
                }
            }
            formatted
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_into_columns() {
        let columns = vec![
            ColumnConfig::new_with_separator(": ", ColumnAlignment::Left),
            ColumnConfig::new(ColumnAlignment::Right), // default " " separator
            ColumnConfig::new_with_separator(", ", ColumnAlignment::Left),
            ColumnConfig::new(ColumnAlignment::Left), // Last column
        ];

        let text = "name: 100KB (2024-01-01), /path";
        let result = split_into_columns(text, &columns);

        assert_eq!(result, vec!["name", "100KB", "(2024-01-01)", "/path"]);
    }

    #[test]
    fn test_calculate_column_widths() {
        let rows = vec![
            vec!["short".to_string(), "medium".to_string()],
            vec!["longer text".to_string(), "x".to_string()],
        ];

        let widths = calculate_column_widths(&rows);
        assert_eq!(widths, vec![11, 6]);
    }

    #[test]
    fn test_pad_string() {
        assert_eq!(pad_string("test", 8, ColumnAlignment::Left), "test    ");
        assert_eq!(pad_string("test", 8, ColumnAlignment::Right), "    test");
        assert_eq!(pad_string("test", 2, ColumnAlignment::Left), "test");
    }

    #[test]
    fn test_format_as_table() {
        let texts = vec![
            "a: 100KB, /path1".to_string(),
            "longer: 1MB, /path2".to_string(),
        ];

        let columns = vec![
            ColumnConfig::new_with_separator(": ", ColumnAlignment::Left),
            ColumnConfig::new_with_separator(", ", ColumnAlignment::Right),
            ColumnConfig::new(ColumnAlignment::Left), // Last column (no padding needed)
        ];

        let result = format_as_table(&texts, &columns);

        // After alignment:
        // "a     " + ": " + "100KB" + ", " + "/path1"
        // "longer" + ": " + "  1MB" + ", " + "/path2"
        assert_eq!(result[0], "a     : 100KB, /path1");
        assert_eq!(result[1], "longer:   1MB, /path2");
    }

    #[test]
    fn test_separator() {
        // Test replacing default separator
        let col1 = ColumnConfig::new(ColumnAlignment::Left);
        assert_eq!(col1.separator, " ");

        let col1 = col1.separator(": ");
        assert_eq!(col1.separator, ": ");

        // Test replacing custom separator
        let col2 = ColumnConfig::new_with_separator(", ", ColumnAlignment::Right);
        assert_eq!(col2.separator, ", ");

        let col2 = col2.separator(" | ");
        assert_eq!(col2.separator, " | ");

        // Test chaining
        let col3 = ColumnConfig::new(ColumnAlignment::Right).separator(" -> ");
        assert_eq!(col3.separator, " -> ");
        assert_eq!(col3.alignment, ColumnAlignment::Right);
    }

    #[test]
    fn test_empty_inputs() {
        // Empty text list
        let empty_texts: Vec<String> = vec![];
        let columns = vec![ColumnConfig::new(ColumnAlignment::Left)];
        let result = format_as_table(&empty_texts, &columns);
        assert!(result.is_empty());

        // Empty columns list
        let texts = vec!["test".to_string()];
        let empty_columns: Vec<ColumnConfig> = vec![];
        let result = format_as_table(&texts, &empty_columns);
        assert_eq!(result, texts);

        // Both empty
        let result = format_as_table(&empty_texts, &empty_columns);
        assert!(result.is_empty());
    }

    #[test]
    fn test_single_column() {
        let texts = vec![
            "short".to_string(),
            "medium text".to_string(),
            "very long text here".to_string(),
        ];

        let columns = vec![ColumnConfig::new(ColumnAlignment::Left)];
        let result = format_as_table(&texts, &columns);

        // Single column should not be padded (it's the last column)
        assert_eq!(result[0], "short");
        assert_eq!(result[1], "medium text");
        assert_eq!(result[2], "very long text here");
    }

    #[test]
    fn test_unicode_characters() {
        let texts = vec![
            "café: 100€, Paris".to_string(),
            "résumé: 200€, München".to_string(),
            "日本: 300¥, 東京".to_string(),
        ];

        let columns = vec![
            ColumnConfig::new_with_separator(": ", ColumnAlignment::Left),
            ColumnConfig::new_with_separator(", ", ColumnAlignment::Right),
            ColumnConfig::new(ColumnAlignment::Left),
        ];

        let result = format_as_table(&texts, &columns);

        // Check that unicode is handled correctly
        assert!(result[0].contains("café"));
        assert!(result[1].contains("résumé"));
        assert!(result[2].contains("日本"));

        // Verify all lines are properly formatted (no panic on unicode)
        assert_eq!(result.len(), 3);

        // Check that columns are aligned by verifying similar structure
        // The exact byte position may vary due to unicode, but width should be consistent
        assert!(result[0].contains(": "));
        assert!(result[1].contains(": "));
        assert!(result[2].contains(": "));
        assert!(result[0].contains(", "));
        assert!(result[1].contains(", "));
        assert!(result[2].contains(", "));
    }

    #[test]
    fn test_mixed_alignment() {
        let texts = vec!["a: 1, x".to_string(), "bb: 22, yy".to_string()];

        // Test various alignment combinations
        let columns = vec![
            ColumnConfig::new_with_separator(": ", ColumnAlignment::Left),
            ColumnConfig::new_with_separator(", ", ColumnAlignment::Right),
            ColumnConfig::new(ColumnAlignment::Left),
        ];

        let result = format_as_table(&texts, &columns);

        // First column (left-aligned)
        assert!(result[0].starts_with("a "));
        assert!(result[1].starts_with("bb"));

        // Second column (right-aligned) - numbers should align
        let first_comma = result[0].find(", ").unwrap();
        let second_comma = result[1].find(", ").unwrap();
        assert_eq!(first_comma, second_comma);
    }

    #[test]
    fn test_empty_cells() {
        let texts = vec!["a: : c".to_string(), "b: x: y".to_string()];

        let columns = vec![
            ColumnConfig::new_with_separator(": ", ColumnAlignment::Left),
            ColumnConfig::new_with_separator(": ", ColumnAlignment::Left),
            ColumnConfig::new(ColumnAlignment::Left),
        ];

        let result = format_as_table(&texts, &columns);

        // Empty cells should still be handled
        assert_eq!(result.len(), 2);
        assert!(result[0].contains("a"));
        assert!(result[1].contains("b"));
    }

    #[test]
    fn test_varying_column_counts() {
        // Some rows have fewer columns than others
        let texts = vec![
            "a: b, c, d".to_string(),
            "e: f".to_string(),    // Missing last two columns
            "g: h, i".to_string(), // Missing last column
        ];

        let columns = vec![
            ColumnConfig::new_with_separator(": ", ColumnAlignment::Left),
            ColumnConfig::new_with_separator(", ", ColumnAlignment::Left),
            ColumnConfig::new_with_separator(", ", ColumnAlignment::Left),
            ColumnConfig::new(ColumnAlignment::Left),
        ];

        let result = format_as_table(&texts, &columns);

        // All results should be formatted without panic
        assert_eq!(result.len(), 3);
        assert!(result[0].contains("a"));
        assert!(result[1].contains("e"));
        assert!(result[2].contains("g"));
    }

    #[test]
    fn test_special_separators() {
        let texts = vec![
            "a -> b => c".to_string(),
            "longer -> text => more".to_string(),
        ];

        let columns = vec![
            ColumnConfig::new_with_separator(" -> ", ColumnAlignment::Left),
            ColumnConfig::new_with_separator(" => ", ColumnAlignment::Left),
            ColumnConfig::new(ColumnAlignment::Left),
        ];

        let result = format_as_table(&texts, &columns);

        assert!(result[0].contains(" -> "));
        assert!(result[1].contains(" -> "));
        assert!(result[0].contains(" => "));
        assert!(result[1].contains(" => "));
    }

    #[test]
    fn test_very_long_content() {
        let long_text = "x".repeat(1000);
        let texts = vec![format!("short: {}", long_text), "a: b".to_string()];

        let columns = vec![
            ColumnConfig::new_with_separator(": ", ColumnAlignment::Left),
            ColumnConfig::new(ColumnAlignment::Left),
        ];

        let result = format_as_table(&texts, &columns);

        // Should handle very long content without panic
        assert_eq!(result.len(), 2);
        assert!(result[0].len() > 1000);
    }

    #[test]
    fn test_whitespace_handling() {
        let texts = vec!["  a  :  b  ,  c  ".to_string(), "d:e,f".to_string()];

        let columns = vec![
            ColumnConfig::new_with_separator(":", ColumnAlignment::Left),
            ColumnConfig::new_with_separator(",", ColumnAlignment::Left),
            ColumnConfig::new(ColumnAlignment::Left),
        ];

        let result = format_as_table(&texts, &columns);

        // Whitespace should be preserved in the data
        assert!(result[0].contains("  a  "));
        assert!(result[1].contains("d"));
    }

    #[test]
    fn test_all_right_aligned() {
        let texts = vec![
            "1: 10, 100".to_string(),
            "22: 20, 200".to_string(),
            "333: 30, 300".to_string(),
        ];

        let columns = vec![
            ColumnConfig::new_with_separator(": ", ColumnAlignment::Right),
            ColumnConfig::new_with_separator(", ", ColumnAlignment::Right),
            ColumnConfig::new(ColumnAlignment::Right),
        ];

        let result = format_as_table(&texts, &columns);

        // All columns right-aligned
        assert!(result[0].starts_with("  1")); // Padded on left
        assert!(result[1].starts_with(" 22"));
        assert!(result[2].starts_with("333"));
    }

    #[test]
    fn test_all_left_aligned() {
        let texts = vec![
            "a: b, c".to_string(),
            "aa: bb, cc".to_string(),
            "aaa: bbb, ccc".to_string(),
        ];

        let columns = vec![
            ColumnConfig::new_with_separator(": ", ColumnAlignment::Left),
            ColumnConfig::new_with_separator(", ", ColumnAlignment::Left),
            ColumnConfig::new(ColumnAlignment::Left),
        ];

        let result = format_as_table(&texts, &columns);

        // All columns left-aligned
        assert!(result[0].starts_with("a  ")); // Padded on right
        assert!(result[1].starts_with("aa "));
        assert!(result[2].starts_with("aaa"));
    }

    #[test]
    fn test_separator_not_found() {
        // Text doesn't contain the separator
        let texts = vec!["no separator here".to_string(), "also none".to_string()];

        let columns = vec![
            ColumnConfig::new_with_separator(": ", ColumnAlignment::Left),
            ColumnConfig::new(ColumnAlignment::Left),
        ];

        let result = format_as_table(&texts, &columns);

        // Should still work, treating entire string as first column
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_column_config_default_separator_constant() {
        assert_eq!(ColumnConfig::DEFAULT_SEPARATOR, " ");

        let col = ColumnConfig::new(ColumnAlignment::Left);
        assert_eq!(col.separator, ColumnConfig::DEFAULT_SEPARATOR);
    }

    #[test]
    fn test_column_alignment_equality() {
        assert_eq!(ColumnAlignment::Left, ColumnAlignment::Left);
        assert_eq!(ColumnAlignment::Right, ColumnAlignment::Right);
        assert_ne!(ColumnAlignment::Left, ColumnAlignment::Right);
    }

    #[test]
    fn test_multiple_separator_method_calls() {
        let col = ColumnConfig::new(ColumnAlignment::Left)
            .separator(": ")
            .separator(", ")
            .separator(" | ");

        // Last call should win
        assert_eq!(col.separator, " | ");
    }

    #[test]
    fn test_real_world_file_listing() {
        let texts = vec![
            "main.rs: -rw-r--r--, 1.2 KB, 2025-10-15 14:30, /src".to_string(),
            "lib.rs: -rw-r--r--, 856 B, 2025-10-14 10:20, /src".to_string(),
            "Cargo.toml: -rw-r--r--, 512 B, 2025-10-13 09:15, /".to_string(),
        ];

        let columns = vec![
            ColumnConfig::new_with_separator(": ", ColumnAlignment::Left), // filename
            ColumnConfig::new_with_separator(", ", ColumnAlignment::Left), // permissions
            ColumnConfig::new_with_separator(", ", ColumnAlignment::Right), // size
            ColumnConfig::new_with_separator(", ", ColumnAlignment::Left), // date
            ColumnConfig::new(ColumnAlignment::Left),                      // path
        ];

        let result = format_as_table(&texts, &columns);

        // Verify formatting
        assert_eq!(result.len(), 3);

        // Filenames should align
        let colon_pos: Vec<usize> = result.iter().map(|s| s.find(": ").unwrap()).collect();
        assert_eq!(colon_pos[0], colon_pos[1]);
        assert_eq!(colon_pos[1], colon_pos[2]);
    }

    #[test]
    fn test_real_world_server_listing() {
        let texts = vec![
            "web-01: 192.168.1.10, 8080, Active, 99.9%".to_string(),
            "api-gateway: 192.168.1.20, 3000, Active, 99.5%".to_string(),
            "db: 192.168.1.30, 5432, Stopped, 0%".to_string(),
        ];

        let columns = vec![
            ColumnConfig::new_with_separator(": ", ColumnAlignment::Left),
            ColumnConfig::new_with_separator(", ", ColumnAlignment::Right),
            ColumnConfig::new_with_separator(", ", ColumnAlignment::Right),
            ColumnConfig::new_with_separator(", ", ColumnAlignment::Left),
            ColumnConfig::new(ColumnAlignment::Right),
        ];

        let result = format_as_table(&texts, &columns);

        assert_eq!(result.len(), 3);
        assert!(result[0].contains("web-01"));
        assert!(result[1].contains("api-gateway"));
        assert!(result[2].contains("db"));
    }

    #[test]
    fn test_numbers_alignment() {
        let texts = vec![
            "Item1: 1, 10, 100".to_string(),
            "Item2: 2, 20, 200".to_string(),
            "Item3: 3, 30, 300".to_string(),
        ];

        let columns = vec![
            ColumnConfig::new_with_separator(": ", ColumnAlignment::Left),
            ColumnConfig::new_with_separator(", ", ColumnAlignment::Right),
            ColumnConfig::new_with_separator(", ", ColumnAlignment::Right),
            ColumnConfig::new(ColumnAlignment::Right),
        ];

        let result = format_as_table(&texts, &columns);

        // Numbers should be right-aligned for easy comparison
        assert_eq!(result.len(), 3);

        // Find positions of first comma (after first number column)
        let comma_positions: Vec<usize> = result.iter().map(|s| s.find(", ").unwrap()).collect();

        // All should be at same position
        assert_eq!(comma_positions[0], comma_positions[1]);
        assert_eq!(comma_positions[1], comma_positions[2]);
    }
}
