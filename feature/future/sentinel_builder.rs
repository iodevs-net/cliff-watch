//! Sentinel Report Builder
//!
//! Deterministic assembler for SentinelReport.
//!
//! RESPONSIBILITIES:
//! - Collect precomputed metrics.
//! - Validate structural invariants.
//! - Assemble a SentinelReport.
//!
//! NON-RESPONSIBILITIES:
//! - No metric computation.
//! - No IO, hashing, signing or PoW.
//! - No clock ownership.
//! - No policy or classification.
//!
//! This module is orchestration glue, not intelligence.

use crate::sentinel_report::{
    IntegrityEnvelope,
    MetricsBundle,
    ObservationWindow,
    SentinelReport,
};

/// Builder error surface is intentionally small and explicit.
#[derive(Debug, thiserror::Error)]
pub enum BuilderError {
    #[error("missing report_id")]
    MissingReportId,

    #[error("missing workspace_id")]
    MissingWorkspaceId,

    #[error("missing observation window")]
    MissingObservation,

    #[error("observation window is structurally invalid")]
    InvalidObservation,

    #[error("missing integrity envelope")]
    MissingIntegrity,

    #[error("sentinel report failed structural invariants")]
    UnsoundReport,
}

/// Deterministic builder for SentinelReport.
#[derive(Debug, Default)]
pub struct SentinelBuilder {
    report_id: Option<String>,
    workspace_id: Option<String>,
    observation: Option<ObservationWindow>,
    metrics: MetricsBundle,
    integrity: Option<IntegrityEnvelope>,
}

impl SentinelBuilder {
    /// Create an empty builder.
    pub fn new() -> Self {
        Self::default()
    }

    pub fn report_id(mut self, id: impl Into<String>) -> Self {
        self.report_id = Some(id.into());
        self
    }

    pub fn workspace_id(mut self, id: impl Into<String>) -> Self {
        self.workspace_id = Some(id.into());
        self
    }

    pub fn observation(mut self, obs: ObservationWindow) -> Self {
        self.observation = Some(obs);
        self
    }

    pub fn metrics(mut self, metrics: MetricsBundle) -> Self {
        self.metrics = metrics;
        self
    }

    pub fn integrity(mut self, integrity: IntegrityEnvelope) -> Self {
        self.integrity = Some(integrity);
        self
    }

    /// Consume builder and emit a structurally valid SentinelReport.
    pub fn build(self) -> Result<SentinelReport, BuilderError> {
        let report_id = self.report_id.ok_or(BuilderError::MissingReportId)?;
        let workspace_id = self.workspace_id.ok_or(BuilderError::MissingWorkspaceId)?;
        let observation = self.observation.ok_or(BuilderError::MissingObservation)?;
        let integrity = self.integrity.ok_or(BuilderError::MissingIntegrity)?;

        if observation.start_us > observation.end_us
            || observation.raw_event_count < observation.emitted_event_count
        {
            return Err(BuilderError::InvalidObservation);
        }

        let report = SentinelReport::new(
            report_id,
            workspace_id,
            observation,
            self.metrics,
            integrity,
        );

        if !report.is_structurally_sound() {
            return Err(BuilderError::UnsoundReport);
        }

        Ok(report)
    }
}

// -----------------------------------------------------------------------------
// Tests
// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn builds_valid_sentinel_report() {
        let report = SentinelBuilder::new()
            .report_id("r-001")
            .workspace_id("ws-123")
            .observation(ObservationWindow {
                start_us: 1,
                end_us: 10,
                raw_event_count: 10,
                emitted_event_count: 5,
            })
            .metrics(MetricsBundle::default())
            .integrity(IntegrityEnvelope {
                content_hash: "hash".into(),
                signatures: BTreeMap::new(),
                proof_of_work: None,
            })
            .build()
            .unwrap();

        assert_eq!(report.report_id, "r-001");
        assert!(report.metrics_finalized_at_us > 0);
        assert!(report.is_structurally_sound());
    }
}
