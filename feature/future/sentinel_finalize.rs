//! Sentinel Finalize
//!
//! Deterministic finalization pipeline for SentinelReport integrity.
//!
//! RESPONSIBILITIES:
//! - Canonicalize a SentinelReport (without integrity).
//! - Compute content hash.
//! - Attach cryptographic signatures.
//! - Optionally attach Proof-of-Work.
//!
//! NON-RESPONSIBILITIES:
//! - No metric computation.
//! - No IO.
//! - No key storage.
//! - No policy or classification.
//!
//! This module seals truth. It does not judge it.

use crate::sentinel_report::{
    IntegrityEnvelope,
    ProofOfWork,
    SentinelReport,
};
use crate::sentinel_sign::{sign_content_hash, SignatureScheme};
use crate::sentinel_pow::{PowChallenge, PowSolution};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

#[derive(Debug, thiserror::Error)]
pub enum FinalizeError {
    #[error("serialization failed")]
    Serialization,

    #[error("signing failed")]
    Signing,

    #[error("proof of work verification failed")]
    InvalidPoW,
}

/// Finalize a SentinelReport by sealing its integrity.
///
/// Flow:
/// 1. Canonical serialize report *without* integrity.
/// 2. Hash serialized bytes.
/// 3. Sign hash.
/// 4. Optionally attach PoW.
pub fn finalize_report<F>(
    mut report: SentinelReport,
    scheme: SignatureScheme,
    sign_fn: F,
    pow: Option<(PowChallenge, PowSolution)>,
) -> Result<SentinelReport, FinalizeError>
where
    F: Fn(&[u8]) -> Vec<u8>,
{
    // ---------------------------------------------------------------------
    // Step 1: canonical serialization (integrity excluded)
    // ---------------------------------------------------------------------

    let canonical_bytes = canonicalize_without_integrity(&report)
        .map_err(|_| FinalizeError::Serialization)?;

    // ---------------------------------------------------------------------
    // Step 2: hash
    // ---------------------------------------------------------------------

    let content_hash = hash_bytes(&canonical_bytes);

    // ---------------------------------------------------------------------
    // Step 3: sign
    // ---------------------------------------------------------------------

    let (scheme_id, signature) =
        sign_content_hash(scheme, &content_hash, sign_fn)
            .map_err(|_| FinalizeError::Signing)?;

    let mut signatures = BTreeMap::new();
    signatures.insert(scheme_id, signature);

    // ---------------------------------------------------------------------
    // Step 4: optional PoW (already verified upstream)
    // ---------------------------------------------------------------------

    let proof_of_work = pow.map(|(_, sol)| ProofOfWork {
        difficulty: 0, // informational only; verifier owns semantics
        nonce: sol.nonce,
        hash: hex::encode(sol.hash),
    });

    // ---------------------------------------------------------------------
    // Step 5: seal integrity envelope
    // ---------------------------------------------------------------------

    report.integrity = IntegrityEnvelope {
        content_hash: hex::encode(content_hash),
        signatures,
        proof_of_work,
    };

    Ok(report)
}

// -----------------------------------------------------------------------------
// Internal helpers
// -----------------------------------------------------------------------------

fn canonicalize_without_integrity(
    report: &SentinelReport,
) -> Result<Vec<u8>, ()> {
    #[derive(Serialize)]
    struct Canonical<'a> {
        version: u32,
        report_id: &'a str,
        metrics_finalized_at_us: u128,
        workspace_id: &'a str,
        observation: &'a crate::sentinel_report::ObservationWindow,
        metrics: &'a crate::sentinel_report::MetricsBundle,
    }

    let canonical = Canonical {
        version: report.version,
        report_id: &report.report_id,
        metrics_finalized_at_us: report.metrics_finalized_at_us,
        workspace_id: &report.workspace_id,
        observation: &report.observation,
        metrics: &report.metrics,
    };

    serde_json::to_vec(&canonical).map_err(|_| ())
}

fn hash_bytes(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

// -----------------------------------------------------------------------------
// Tests
// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sentinel_report::{MetricsBundle, ObservationWindow};
    use std::collections::BTreeMap;

    #[test]
    fn finalizes_report_with_signature() {
        let report = SentinelReport {
            version: 1,
            report_id: "r-001".into(),
            metrics_finalized_at_us: 42,
            workspace_id: "ws".into(),
            observation: ObservationWindow {
                start_us: 1,
                end_us: 2,
                raw_event_count: 10,
                emitted_event_count: 5,
            },
            metrics: MetricsBundle::default(),
            integrity: IntegrityEnvelope {
                content_hash: String::new(),
                signatures: BTreeMap::new(),
                proof_of_work: None,
            },
        };

        let finalized = finalize_report(
            report,
            SignatureScheme::Ed25519,
            |data| data.to_vec(), // fake signer
            None,
        )
        .unwrap();

        assert!(!finalized.integrity.content_hash.is_empty());
        assert!(!finalized.integrity.signatures.is_empty());
    }
}
