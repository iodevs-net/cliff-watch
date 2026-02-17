//! Sentinel Report — Auditable provenance contract
//!
//! Immutable, serializable, signable truth container.
//! Contains NO heuristics, NO IO, NO async, NO policy.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

// -----------------------------------------------------------------------------
// Versioning
// -----------------------------------------------------------------------------

/// Increment only on breaking semantic changes.
pub const SENTINEL_REPORT_VERSION: u32 = 1;

// -----------------------------------------------------------------------------
// Core report
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentinelReport {
    /// Schema version of this report.
    pub version: u32,

    /// Opaque, collision-resistant identifier.
    pub report_id: String,

    /// Timestamp when all metrics were frozen (UNIX epoch, µs).
    pub metrics_finalized_at_us: u128,

    /// Observed workspace / repository identifier.
    pub workspace_id: String,

    /// High-level observation window.
    pub observation: ObservationWindow,

    /// Collected metric groups.
    pub metrics: MetricsBundle,

    /// System-level integrity metadata.
    pub integrity: IntegrityEnvelope,
}

impl SentinelReport {
    /// Construct a new immutable SentinelReport.
    ///
    /// Time is frozen internally to prevent upstream manipulation.
    pub fn new(
        report_id: impl Into<String>,
        workspace_id: impl Into<String>,
        observation: ObservationWindow,
        metrics: MetricsBundle,
        integrity: IntegrityEnvelope,
    ) -> Self {
        Self {
            version: SENTINEL_REPORT_VERSION,
            report_id: report_id.into(),
            metrics_finalized_at_us: now_us(),
            workspace_id: workspace_id.into(),
            observation,
            metrics,
            integrity,
        }
    }

    /// Structural invariants that MUST hold for any valid report.
    ///
    /// This is not policy. This is schema sanity.
    pub fn is_structurally_sound(&self) -> bool {
        self.version == SENTINEL_REPORT_VERSION
            && self.observation.start_us <= self.observation.end_us
            && self.observation.raw_event_count >= self.observation.emitted_event_count
    }
}

// -----------------------------------------------------------------------------
// Observation window
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservationWindow {
    /// First observed event timestamp (µs).
    pub start_us: u128,

    /// Last observed event timestamp (µs).
    pub end_us: u128,

    /// Total number of low-level events observed (pre-filter).
    pub raw_event_count: u64,

    /// Total number of emitted semantic events.
    pub emitted_event_count: u64,
}

// -----------------------------------------------------------------------------
// Metrics bundle (canonical imports only)
// -----------------------------------------------------------------------------

use crate::temporal::TemporalMetrics;
use crate::structural::StructuralMetrics;
use crate::kinematic::KinematicMetrics;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MetricsBundle {
    pub temporal: Option<TemporalMetrics>,
    pub structural: Option<StructuralMetrics>,
    pub kinematic: Option<KinematicMetrics>,

    /// Extension point for future metrics (namespaced, deterministic order).
    pub extensions: BTreeMap<String, serde_json::Value>,
}

// -----------------------------------------------------------------------------
// Integrity envelope
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityEnvelope {
    /// Hash of canonical serialized report (excluding signatures & PoW).
    pub content_hash: String,

    /// Cryptographic signatures (keyed by scheme / key id).
    pub signatures: BTreeMap<String, String>,

    /// Optional proof-of-work / cost signal.
    pub proof_of_work: Option<ProofOfWork>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofOfWork {
    /// Difficulty parameter (leading zero bits).
    pub difficulty: u8,

    /// Nonce that satisfies the PoW.
    pub nonce: u64,

    /// Hash output that met the difficulty.
    pub hash: String,
}

// -----------------------------------------------------------------------------
// Utilities
// -----------------------------------------------------------------------------

#[inline]
fn now_us() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_micros()
}

// -----------------------------------------------------------------------------
// Tests
// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sentinel_report_is_serializable_and_sound() {
        let report = SentinelReport::new(
            "report-001",
            "workspace-xyz",
            ObservationWindow {
                start_us: 1,
                end_us: 2,
                raw_event_count: 10,
                emitted_event_count: 5,
            },
            MetricsBundle::default(),
            IntegrityEnvelope {
                content_hash: "hash".into(),
                signatures: BTreeMap::new(),
                proof_of_work: None,
            },
        );

        let json = serde_json::to_string(&report).unwrap();
        let back: SentinelReport = serde_json::from_str(&json).unwrap();

        assert_eq!(back.version, SENTINEL_REPORT_VERSION);
        assert!(back.metrics_finalized_at_us > 0);
        assert!(back.is_structurally_sound());
    }
}
