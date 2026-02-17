//! Sentinel Collector — metric aggregation layer
//!
//! RESPONSIBILITIES:
//! - Coordinate metric analyzers.
//! - Populate MetricsBundle.
//!
//! NON-RESPONSIBILITIES:
//! - No policy.
//! - No thresholds.
//! - No report building.

use crate::{
    kinematic::{analyze_kinematics, KinematicEvent},
    structural::analyze_ncd,
    temporal::analyze_timestamps,
    sentinel_report::MetricsBundle,
};

#[derive(Debug, thiserror::Error)]
pub enum CollectorError {
    #[error("temporal analysis failed")]
    Temporal,

    #[error("structural analysis failed")]
    Structural,

    #[error("kinematic analysis failed")]
    Kinematic,
}

/// Input set for collection (explicit, boring, honest).
pub struct CollectionInput<'a> {
    pub timestamps_us: Option<&'a [u128]>,
    pub structural_pair: Option<(&'a [u8], &'a [u8])>,
    pub kinematic_events: Option<&'a [KinematicEvent]>,
}

/// Collect all available metrics into a bundle.
pub fn collect_metrics(input: CollectionInput) -> Result<MetricsBundle, CollectorError> {
    let mut bundle = MetricsBundle::default();

    if let Some(ts) = input.timestamps_us {
        bundle.temporal = Some(
            analyze_timestamps(ts).map_err(|_| CollectorError::Temporal)?,
        );
    }

    if let Some((x, y)) = input.structural_pair {
        bundle.structural = Some(
            analyze_ncd(x, y).map_err(|_| CollectorError::Structural)?,
        );
    }

    if let Some(events) = input.kinematic_events {
        bundle.kinematic = Some(
            analyze_kinematics(events).map_err(|_| CollectorError::Kinematic)?,
        );
    }

    Ok(bundle)
}
