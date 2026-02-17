//! Backend de captura para Linux usando evdev
//!
//! # DEPRECATED - REMOVED IN PHASE 3
//!
//! This module has been removed as part of the Phase 3 Architecture Transformation.
//! The evdev-based hardware capture was privacy-invasive and required root privileges.
//!
//! ## Migration Path
//!
//! Use the `IdeSensorBackend` from `ide_sensor.rs` instead:
//! - Does not require root privileges
//! - Respects user privacy
//! - Works across different platforms via IDE extensions
//!
//! The legacy evdev backend read directly from `/dev/input/event*` devices,
//! which raised significant privacy concerns. This module is retained only for
//! documentation purposes and will be completely removed in a future version.

use crate::mouse_sentinel::InputEvent;
use crate::backend::Backend;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use anyhow::Result;

/// Legacy LinuxBackend - REMOVED
///
/// This struct is kept for API compatibility but is no longer functional.
/// The evdev-based hardware capture has been removed in Phase 3.
#[deprecated(since = "0.3.0", note = "Hardware capture removed. Use IdeSensorBackend instead.")]
pub struct LinuxBackend {
    _device_path: Option<std::path::PathBuf>,
}

impl LinuxBackend {
    /// Create a new LinuxBackend (no longer functional)
    #[deprecated(since = "0.3.0", note = "Hardware capture removed. Use IdeSensorBackend instead.")]
    pub fn new(_device_path: Option<std::path::PathBuf>) -> Self {
        Self { _device_path: None }
    }

    /// Discover input devices - REMOVED
    ///
    /// This function no longer performs any discovery.
    /// Hardware capture has been removed in Phase 3.
    #[deprecated(since = "0.3.0", note = "Hardware capture removed. Use IdeSensorBackend instead.")]
    pub fn discover_input_devices() -> Vec<std::path::PathBuf> {
        Vec::new()
    }
}

impl Backend for LinuxBackend {
    /// Start capture - REMOVED
    ///
    /// This function no longer performs any capture.
    /// Hardware capture has been removed in Phase 3.
    #[deprecated(since = "0.3.0", note = "Hardware capture removed. Use IdeSensorBackend instead.")]
    fn start(&self, _tx: mpsc::Sender<InputEvent>, _shutdown: CancellationToken) -> Result<()> {
        Err(anyhow::anyhow!(
            "Hardware capture removed in Phase 3. Use IdeSensorBackend instead."
        ))
    }
}
