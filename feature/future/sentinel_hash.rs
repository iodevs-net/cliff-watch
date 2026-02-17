//! Sentinel Hasher — canonical content hashing
//!
//! RESPONSIBILITIES:
//! - Produce deterministic byte representation.
//! - Hash SentinelReport excluding signatures and PoW.
//!
//! NON-RESPONSIBILITIES:
//! - No IO.
//! - No signing.
//! - No policy.

use crate::sentinel_report::{IntegrityEnvelope, SentinelReport};
use serde::Serialize;
use sha2::{Digest, Sha256};

#[derive(Debug, thiserror::Error)]
pub enum HashError {
    #[error("serialization failed")]
    SerializationFailed,
}

/// Compute canonical hash of a SentinelReport.
///
/// Rules:
/// - Deterministic JSON (serde).
/// - Integrity.signatures and integrity.proof_of_work are excluded.
pub fn hash_report(report: &SentinelReport) -> Result<String, HashError> {
    let stripped = CanonicalReportView::from(report);

    let bytes =
        serde_json::to_vec(&stripped).map_err(|_| HashError::SerializationFailed)?;

    let digest = Sha256::digest(&bytes);
    Ok(hex::encode(digest))
}

/// Canonical, hashable view of SentinelReport.
#[derive(Serialize)]
struct CanonicalReportView<'a> {
    version: u32,
    report_id: &'a str,
    metrics_finalized_at_us: u128,
    workspace_id: &'a str,
    observation: &'a crate::sentinel_report::ObservationWindow,
    metrics: &'a crate::sentinel_report::MetricsBundle,
    integrity: CanonicalIntegrity<'a>,
}

#[derive(Serialize)]
struct CanonicalIntegrity<'a> {
    content_hash: &'a str,
}

impl<'a> From<&'a SentinelReport> for CanonicalReportView<'a> {
    fn from(r: &'a SentinelReport) -> Self {
        Self {
            version: r.version,
            report_id: &r.report_id,
            metrics_finalized_at_us: r.metrics_finalized_at_us,
            workspace_id: &r.workspace_id,
            observation: &r.observation,
            metrics: &r.metrics,
            integrity: CanonicalIntegrity {
                content_hash: &r.integrity.content_hash,
            },
        }
    }
}
