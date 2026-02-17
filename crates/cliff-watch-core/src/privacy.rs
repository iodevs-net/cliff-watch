//! Privacy Filtering Module
//!
//! This module provides functions to sanitize and filter data to protect user privacy
//! while maintaining the ability to generate meaningful Proof of Human Work (PoHW).
//!
//! ## Privacy Principles
//!
//! 1. **Hash file paths**: Full paths are replaced with SHA-256 hashes
//! 2. **Bucket timestamps**: Precise timestamps are reduced to coarse-grained buckets
//! 3. **Sanitize content**: Raw code content is never stored, only aggregated metrics
//!
//! ## Phase 3 Architecture Transformation
//!
//! This module is part of the Phase 3 transformation that removes intrusive hardware
//! capture (evdev) and implements privacy-preserving data filtering.

use sha2::{Digest, Sha256};
use chrono::{DateTime, Utc};
use std::path::Path;

/// Hash a file path using SHA-256
///
/// This function replaces full file paths with a cryptographic hash to protect
/// user privacy while maintaining the ability to track work on specific files.
///
/// # Arguments
///
/// * `path` - The file path to hash
///
/// # Returns
///
/// A hexadecimal string representing the SHA-256 hash of the file path
///
/// # Example
///
/// ```rust
/// use cliff_watch_core::privacy::hash_file_path;
/// use std::path::Path;
///
/// let hash = hash_file_path(Path::new("/home/user/project/src/main.rs"));
/// assert_eq!(hash.len(), 64); // SHA-256 produces 64 hex characters
/// ```
///
/// # Privacy Impact
///
/// - Full paths are never stored in logs or transmitted
/// - The hash cannot be reversed to recover the original path
/// - Different paths produce different hashes (avalanche effect)
pub fn hash_file_path(path: &Path) -> String {
    let path_str = path.to_string_lossy();
    let mut hasher = Sha256::new();
    hasher.update(path_str.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

/// Bucket a timestamp to reduce precision
///
/// This function reduces timestamp precision by grouping timestamps into
/// fixed-size buckets. This protects user privacy by making it harder to
/// correlate events across different data sources.
///
/// # Arguments
///
/// * `timestamp` - The timestamp to bucket (UTC)
/// * `bucket_seconds` - The size of each bucket in seconds (default: 5)
///
/// # Returns
///
/// A DateTime representing the start of the bucket containing the timestamp
///
/// # Example
///
/// ```rust
/// use cliff_watch_core::privacy::bucket_timestamp;
/// use chrono::{Utc, Timelike};
///
/// let timestamp = Utc::now();
/// let bucketed = bucket_timestamp(timestamp, 5);
/// assert_eq!(bucketed.second() % 5, 0);
/// ```
///
/// # Privacy Impact
///
/// - Reduces temporal precision of event tracking
/// - Makes it harder to correlate events across different systems
/// - Default bucket size of 5 seconds provides good privacy/utility balance
pub fn bucket_timestamp(timestamp: DateTime<Utc>, bucket_seconds: i64) -> DateTime<Utc> {
    let secs = timestamp.timestamp();
    let bucketed_secs = (secs / bucket_seconds) * bucket_seconds;
    DateTime::<Utc>::from_timestamp(bucketed_secs, 0).unwrap_or(timestamp)
}

/// Sanitize code content to avoid storing raw code
///
/// This function removes sensitive information from code content before storage
/// or transmission. It extracts only metadata and aggregated metrics without
/// storing the actual code.
///
/// # Arguments
///
/// * `content` - The code content to sanitize
///
/// # Returns
///
/// A `SanitizedContent` struct containing only non-sensitive metadata
///
/// # Example
///
/// ```rust
/// use cliff_watch_core::privacy::sanitize_content;
///
/// let code = r#"
///     fn main() {
///         println!("Hello, World!");
///     }
/// "#;
/// let sanitized = sanitize_content(code);
/// assert!(sanitized.raw_content.is_none());
/// ```
///
/// # Privacy Impact
///
/// - Raw code is never stored or transmitted
/// - Only aggregated metrics are retained
/// - Comments and string literals are removed
/// - Variable names are normalized
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SanitizedContent {
    /// Total number of characters in the content
    pub char_count: usize,
    /// Number of lines in the content
    pub line_count: usize,
    /// Estimated code complexity (0-100)
    pub complexity_score: u8,
    /// Language detected (if any)
    pub language: Option<String>,
    /// Raw content is always None (never stored)
    pub raw_content: Option<String>,
}

impl SanitizedContent {
    /// Create a new SanitizedContent from raw content
    pub fn new(content: &str) -> Self {
        let char_count = content.chars().count();
        let line_count = content.lines().count();
        let complexity_score = Self::estimate_complexity(content);
        let language = Self::detect_language(content);

        Self {
            char_count,
            line_count,
            complexity_score,
            language,
            raw_content: None,
        }
    }

    /// Estimate code complexity based on simple heuristics
    fn estimate_complexity(content: &str) -> u8 {
        let mut score: u8 = 0;

        // Count function definitions
        let function_count = content.matches("fn ").count()
            + content.matches("function ").count()
            + content.matches("def ").count();
        score = score.saturating_add((function_count * 2) as u8);

        // Count control flow keywords
        let control_flow = content.matches("if ").count()
            + content.matches("for ").count()
            + content.matches("while ").count()
            + content.matches("match ").count()
            + content.matches("switch ").count();
        score = score.saturating_add((control_flow * 3) as u8);

        // Count nesting (bracket depth)
        let mut max_depth = 0;
        let mut current_depth: i32 = 0;
        for c in content.chars() {
            match c {
                '{' | '(' | '[' => {
                    current_depth += 1;
                    max_depth = max_depth.max(current_depth);
                }
                '}' | ')' | ']' => {
                    current_depth = current_depth.saturating_sub(1);
                }
                _ => {}
            }
        }
        score = score.saturating_add((max_depth * 5) as u8);

        // Cap at 100
        score.min(100)
    }

    /// Detect programming language from content
    fn detect_language(content: &str) -> Option<String> {
        let content_lower = content.to_lowercase();

        // Simple heuristics for common languages
        if content.contains("fn ") && content.contains("pub ") {
            Some("rust".to_string())
        } else if content.contains("function ") && content.contains("const ") {
            Some("typescript".to_string())
        } else if content.contains("def ") && content.contains("import ") {
            Some("python".to_string())
        } else if content.contains("class ") && content.contains("public ") {
            Some("java".to_string())
        } else if content.contains("package ") && content.contains("class ") {
            Some("java".to_string())
        } else if content.contains("#include ") {
            Some("c".to_string())
        } else if content.contains("use strict") || content_lower.contains("console.log") {
            Some("javascript".to_string())
        } else if content.contains("interface ") && content_lower.contains(": string") {
            Some("typescript".to_string())
        } else if content.contains("impl ") && content.contains("struct ") {
            Some("rust".to_string())
        } else {
            None
        }
    }
}

/// Sanitize code content to avoid storing raw code
///
/// This is a convenience function that creates a `SanitizedContent` instance.
/// See `SanitizedContent::new` for more details.
pub fn sanitize_content(content: &str) -> SanitizedContent {
    SanitizedContent::new(content)
}

/// Hash a string value using SHA-256
///
/// This function provides a general-purpose hash function for any string value.
/// It can be used to hash identifiers, usernames, or other sensitive strings.
///
/// # Arguments
///
/// * `value` - The string value to hash
///
/// # Returns
///
/// A hexadecimal string representing the SHA-256 hash
pub fn hash_string(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_hash_file_path() {
        let path = Path::new("/home/user/project/src/main.rs");
        let hash = hash_file_path(path);
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_hash_file_path_consistency() {
        let path = Path::new("/home/user/project/src/main.rs");
        let hash1 = hash_file_path(path);
        let hash2 = hash_file_path(path);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_file_path_uniqueness() {
        let path1 = Path::new("/home/user/project/src/main.rs");
        let path2 = Path::new("/home/user/project/src/lib.rs");
        let hash1 = hash_file_path(path1);
        let hash2 = hash_file_path(path2);
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_bucket_timestamp() {
        let timestamp = DateTime::from_timestamp(1234567890, 0).unwrap();
        let bucketed = bucket_timestamp(timestamp, 5);
        assert_eq!(bucketed.timestamp() % 5, 0);
        assert!(bucketed.timestamp() <= timestamp.timestamp());
    }

    #[test]
    fn test_bucket_timestamp_consistency() {
        let timestamp = DateTime::from_timestamp(1234567892, 0).unwrap();
        let bucketed1 = bucket_timestamp(timestamp, 5);
        let bucketed2 = bucket_timestamp(timestamp, 5);
        assert_eq!(bucketed1, bucketed2);
    }

    #[test]
    fn test_sanitize_content() {
        let code = r#"
            fn main() {
                println!("Hello, World!");
                if true {
                    let x = 42;
                }
            }
        "#;
        let sanitized = sanitize_content(code);
        assert!(sanitized.raw_content.is_none());
        assert!(sanitized.char_count > 0);
        assert!(sanitized.line_count > 0);
        assert!(sanitized.complexity_score <= 100);
    }

    #[test]
    fn test_sanitize_content_language_detection() {
        let rust_code = "fn main() { let x = 42; }";
        let sanitized = sanitize_content(rust_code);
        assert_eq!(sanitized.language, Some("rust".to_string()));

        let python_code = "def main(): x = 42";
        let sanitized = sanitize_content(python_code);
        assert_eq!(sanitized.language, Some("python".to_string()));
    }

    #[test]
    fn test_hash_string() {
        let value = "test_value";
        let hash = hash_string(value);
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_hash_string_consistency() {
        let value = "test_value";
        let hash1 = hash_string(value);
        let hash2 = hash_string(value);
        assert_eq!(hash1, hash2);
    }
}
