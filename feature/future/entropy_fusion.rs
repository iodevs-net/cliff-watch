//! Entropy Fusion — cross-domain anomaly aggregation
//!
//! This module fuses independent descriptive metrics into
//! normalized provenance signals.
//!
//! RESPONSIBILITIES:
//! - Combine temporal, structural, kinematic and entropy metrics.
//! - Normalize signals into a common [0,1] space.
//! - Produce explainable anomaly indicators.
//!
//! NON-RESPONSIBILITIES:
//! - No human / AI classification.
//! - No thresholds or enforcement.
//! - No IO, async, logging or persistence.
//!
//! This module is the *bridge* between measurement and governance.

use crate::temporal::TemporalMetrics;
use crate::structural::StructuralMetrics;
use crate::kinematic::KinematicMetrics;

/// Output of entropy fusion.
///
/// All fields are continuous and descriptive.
#[derive(Debug, Clone)]
pub struct FusionSignals {
    /// Temporal irregularity signal.
    pub temporal_anomaly: f64,

    /// Structural redundancy / algorithmicity signal.
    pub structural_anomaly: f64,

    /// Kinematic unnaturalness signal.
    pub kinematic_anomaly: f64,

    /// Aggregate entropy anomaly score [0,1].
    pub combined_entropy_score: f64,
}

/// Errors are structural, not heuristic.
#[derive(Debug, thiserror::Error)]
pub enum FusionError {
    #[error("No metrics provided")]
    EmptyInput,
}

/// Fuse available metrics into anomaly signals.
///
/// Missing metric groups are handled gracefully.
/// No assumptions are made about intent.
///
/// # Contract
/// - All inputs are already validated.
/// - Outputs are normalized to [0,1].
pub fn fuse_entropy_signals(
    temporal: Option<&TemporalMetrics>,
    structural: Option<&StructuralMetrics>,
    kinematic: Option<&KinematicMetrics>,
) -> Result<FusionSignals, FusionError> {
    if temporal.is_none() && structural.is_none() && kinematic.is_none() {
        return Err(FusionError::EmptyInput);
    }

    let temporal_anomaly = temporal
        .map(normalize_temporal)
        .unwrap_or(0.0);

    let structural_anomaly = structural
        .map(normalize_structural)
        .unwrap_or(0.0);

    let kinematic_anomaly = kinematic
        .map(normalize_kinematic)
        .unwrap_or(0.0);

    // Simple convex combination.
    // Weights are intentionally flat and explainable.
    let mut components = Vec::new();
    if temporal.is_some() {
        components.push(temporal_anomaly);
    }
    if structural.is_some() {
        components.push(structural_anomaly);
    }
    if kinematic.is_some() {
        components.push(kinematic_anomaly);
    }

    let combined_entropy_score =
        components.iter().sum::<f64>() / components.len() as f64;

    Ok(FusionSignals {
        temporal_anomaly,
        structural_anomaly,
        kinematic_anomaly,
        combined_entropy_score,
    })
}

// -----------------------------------------------------------------------------
// Normalization helpers (domain-specific, deterministic)
// -----------------------------------------------------------------------------

fn normalize_temporal(m: &TemporalMetrics) -> f64 {
    // Burstiness near -1 → machine-like
    // Burstiness near +1 → human-like
    // We invert to represent anomaly.
    ((1.0 - m.burstiness) / 2.0).clamp(0.0, 1.0)
}

fn normalize_structural(m: &StructuralMetrics) -> f64 {
    // NCD already approximates [0,1]
    m.ncd.clamp(0.0, 1.0)
}

fn normalize_kinematic(m: &KinematicMetrics) -> f64 {
    // High jerk + high velocity variance → anomaly
    let jerk = m.mean_jerk.clamp(0.0, 1.0);
    let vel_std = m.velocity_std.clamp(0.0, 1.0);

    (0.6 * jerk + 0.4 * vel_std).clamp(0.0, 1.0)
}

// -----------------------------------------------------------------------------
// Tests
// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fusion_with_single_metric_works() {
        let temporal = TemporalMetrics {
            burstiness: -0.8,
            mean_interval_us: 100.0,
            stddev_interval_us: 1.0,
            sample_count: 10,
        };

        let f = fuse_entropy_signals(
            Some(&temporal),
            None,
            None,
        )
        .unwrap();

        assert!(f.temporal_anomaly > 0.8);
        assert_eq!(f.structural_anomaly, 0.0);
    }

    #[test]
    fn fusion_combines_multiple_metrics() {
        let temporal = TemporalMetrics {
            burstiness: -0.5,
            mean_interval_us: 100.0,
            stddev_interval_us: 2.0,
            sample_count: 10,
        };

        let structural = StructuralMetrics {
            ncd: 0.7,
            c_x: 100,
            c_y: 100,
            c_xy: 150,
        };

        let f = fuse_entropy_signals(
            Some(&temporal),
            Some(&structural),
            None,
        )
        .unwrap();

        assert!(f.combined_entropy_score > 0.5);
    }
}
