//! Entropy primitives — statistical properties of byte sequences
//!
//! This module provides *pure*, descriptive entropy calculations.
//!
//! RESPONSIBILITIES:
//! - Compute Shannon entropy over byte distributions.
//! - Provide normalized variants for downstream composition.
//!
//! NON-RESPONSIBILITIES:
//! - No compression-based metrics (see structural.rs).
//! - No classification or thresholds.
//! - No policy or governance decisions.
//!
//! These functions are mathematical utilities, not signals.

/// Compute Shannon entropy (in bits) of a byte slice.
///
/// Range:
/// - 0.0 → perfectly uniform (all bytes identical)
/// - 8.0 → maximum entropy (uniform distribution)
pub fn shannon_entropy(data: &[u8]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }

    let mut frequency = [0u64; 256];

    for &byte in data {
        frequency[byte as usize] += 1;
    }

    let len = data.len() as f64;
    let mut entropy = 0.0;

    for &count in &frequency {
        if count > 0 {
            let p = count as f64 / len;
            entropy -= p * p.log2();
        }
    }

    entropy
}

/// Compute normalized Shannon entropy in range [0.0, 1.0].
///
/// Normalization is relative to maximum possible entropy (8 bits per byte).
///
/// This is a *mathematical normalization*, not a judgment.
pub fn normalized_entropy(data: &[u8]) -> f64 {
    shannon_entropy(data) / 8.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_entropy_for_uniform_data() {
        let data = vec![42u8; 1024];
        assert_eq!(shannon_entropy(&data), 0.0);
    }

    #[test]
    fn high_entropy_for_random_like_data() {
        let data: Vec<u8> = (0..=255).collect();
        let e = shannon_entropy(&data);
        assert!(e > 7.0);
    }

    #[test]
    fn normalized_entropy_in_unit_range() {
        let data = b"some test data";
        let e = normalized_entropy(data);
        assert!((0.0..=1.0).contains(&e));
    }
}
