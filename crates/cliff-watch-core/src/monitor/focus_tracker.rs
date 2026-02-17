//! Focus tracker integration for tracking developer focus state.

use crate::focus_protocol::{SensorEvent, NavigationType};
use crate::focus_session::{FocusTracker, FocusMetrics};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use tracing::info;

/// Focus tracker integration wrapper.
pub struct FocusTrackerIntegration {
    tracker: Arc<RwLock<FocusTracker>>,
}

impl FocusTrackerIntegration {
    pub fn new() -> Self {
        Self {
            tracker: Arc::new(RwLock::new(FocusTracker::new())),
        }
    }

    pub fn get_tracker(&self) -> Arc<RwLock<FocusTracker>> {
        self.tracker.clone()
    }

    pub fn get_metrics(&self) -> FocusMetrics {
        self.tracker
            .read()
            .expect("Focus tracker RwLock poisoned")
            .get_metrics()
    }

    pub fn handle_sensor_event(&self, event: SensorEvent) {
        if let Ok(mut tracker) = self.tracker.write() {
            match event {
                SensorEvent::FocusGained { file_path, .. } => {
                    tracker.focus_gained(file_path.clone().map(PathBuf::from));
                    info!("Focus Gained: {:?}", file_path);
                }
                SensorEvent::FocusLost { .. } => {
                    tracker.focus_lost();
                    info!("Focus Lost");
                }
                SensorEvent::EditBurst { file_path, chars_delta, .. } => {
                    tracker.edit_burst(&file_path, chars_delta);
                }
                SensorEvent::Navigation {
                    file_path,
                    nav_type,
                    timestamp_ms,
                    ..
                } => {
                    tracker.navigation(&file_path, timestamp_ms);
                    if nav_type == NavigationType::FileSwitch {
                        info!("Switched to file: {}", file_path);
                    }
                }
                SensorEvent::Heartbeat { .. } => {
                    tracker.heartbeat();
                }
                SensorEvent::Disconnect { .. } => {
                    tracker.reset();
                    tracing::warn!("IDE Sensor disconnected");
                }
                SensorEvent::Keystroke { .. } => {
                    // Por ahora solo notificamos presencia cinemática, 
                    // en el futuro esto alimentará el motor de entropía NCD.
                    tracker.heartbeat();
                }
            }
        }
    }

    pub fn mark_as_productive(&self, file_path: std::path::PathBuf) {
        if let Ok(mut tracker) = self.tracker.write() {
            tracker.mark_as_productive(file_path);
        }
    }
}
