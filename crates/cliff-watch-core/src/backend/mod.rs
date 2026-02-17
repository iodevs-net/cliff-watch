//! Capa de abstracción para backends de captura de eventos
//!
//! ## Arquitectura v2.0 ("El Testigo Silencioso")
//!
//! Este módulo soporta dos modos de operación:
//!
//! 1. **v2.0 (Default)**: `IdeSensorBackend` recibe eventos de extensiones de IDE
//!    via Unix socket. No requiere root, respeta privacidad.
//!
//! 2. **Legacy (REMOVED)**: `LinuxBackend` ha sido eliminado en Phase 3.
//!
//! # DEPRECATED: Legacy evdev Backend - REMOVED IN PHASE 3
//!
//! The legacy evdev backend has been removed as part of Phase 3 Architecture Transformation.
//! It was privacy-invasive and required root privileges.
//!
//! Migration path: Use the v2.0 `IdeSensorBackend` instead, which:
//! - Does not require root privileges
//! - Respects user privacy
//! - Works across different platforms via IDE extensions
//!
//! ## Privacy Filtering Module
//!
//! A new privacy filtering module (`privacy.rs`) has been introduced to:
//! - Hash file paths using SHA-256
//! - Bucket timestamps to reduce precision
//! - Sanitize code content to avoid storing raw data
//!
//! See the `privacy` module for more details.

#[cfg(all(target_os = "linux", feature = "legacy-evdev"))]
pub mod linux;

pub mod ide_sensor;

use crate::mouse_sentinel::InputEvent;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use anyhow::Result;

/// Trait que define un backend de captura de eventos (legacy)
///
/// # DEPRECATED
///
/// This trait is retained for API compatibility but hardware capture
/// has been removed in Phase 3. Use `IdeSensorBackend` instead.
#[deprecated(since = "0.3.0", note = "Hardware capture removed. Use IdeSensorBackend instead.")]
pub trait Backend: Send + Sync {
    /// Inicia la captura de eventos y los envía a través del canal tx.
    /// La captura debe detenerse cuando el shutdown es cancelado.
    #[deprecated(since = "0.3.0", note = "Hardware capture removed. Use IdeSensorBackend instead.")]
    fn start(&self, tx: mpsc::Sender<InputEvent>, shutdown: CancellationToken) -> Result<()>;
}

/// Backend de prueba que simula eventos de mouse
///
/// # DEPRECATED
///
/// MockBackend is retained for testing purposes only.
/// Hardware capture has been removed in Phase 3.
#[deprecated(since = "0.3.0", note = "Use IdeSensorBackend for production.")]
pub struct MockBackend;

impl Backend for MockBackend {
    fn start(&self, tx: mpsc::Sender<InputEvent>, shutdown: CancellationToken) -> Result<()> {
        tokio::spawn(async move {
            let mut x = 0.0;
            let mut y = 0.0;
            let mut t = 0.0;

            loop {
                tokio::select! {
                    _ = shutdown.cancelled() => break,
                    _ = tokio::time::sleep(std::time::Duration::from_millis(100)) => {
                        x += rand::random::<f64>() * 10.0 - 5.0;
                        y += rand::random::<f64>() * 10.0 - 5.0;
                        t += 0.1;
                        let _ = tx.send(InputEvent::Mouse { x, y, t }).await;
                    }
                }
            }
        });
        Ok(())
    }
}

/// Devuelve el backend predeterminado para la plataforma actual
///
/// ## v2.0 Behavior
/// Retorna `None` por defecto porque v2.0 usa `IdeSensorBackend` en lugar
/// de backends de hardware. El daemon debe iniciar `IdeSensorBackend` explícitamente.
///
/// # DEPRECATED: Legacy Backend Support - REMOVED IN PHASE 3
///
/// This function is deprecated. Hardware capture has been completely removed.
/// The `legacy-evdev` feature is no longer functional.
///
/// Migration: Use `IdeSensorBackend` directly instead of relying on this function.
#[deprecated(
    since = "0.3.0",
    note = "Hardware capture removed in Phase 3. Use IdeSensorBackend directly."
)]
pub fn get_default_backend() -> Option<Box<dyn Backend>> {
    // v2.0: No legacy backend by default
    // Hardware capture has been removed in Phase 3
    #[cfg(not(feature = "legacy-evdev"))]
    {
        None
    }

    // Legacy mode - no longer functional
    #[cfg(feature = "legacy-evdev")]
    {
        // Hardware capture has been removed in Phase 3
        // Return None to force migration to IdeSensorBackend
        None
    }
}
