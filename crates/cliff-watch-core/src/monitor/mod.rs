//! Observation layer: file edit telemetry and mouse kinematics.
//!
//! This module coordinates multiple monitoring components:
//! - **FileWatcher**: Watches the filesystem for code edits
//! - **KinematicAnalyzer**: Mouse kinematic analysis via MouseSentinel
//! - **BatteryManager**: Attention battery for cognitive effort tracking
//! - **FocusTracker**: Focus tracking integration

mod file_watcher;
mod kinematic_analyzer;
mod battery_manager;
mod focus_tracker;

// Re-export public API for backward compatibility
pub use file_watcher::{
    EditEvent, EditKind, FileMonitor, MonitorConfig, MonitorError, MonitorStatsSnapshot,
    OverflowPolicy, Shutdown,
};
pub use kinematic_analyzer::{GitMonitor, GitMonitorConfig, GitMonitorError};
pub use battery_manager::AttentionBattery;
pub use focus_tracker::FocusTrackerIntegration;
