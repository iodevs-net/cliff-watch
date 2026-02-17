// Command modules for cliff-watch CLI
// Each module handles a specific command following Single Responsibility Principle

pub mod config;
pub mod daemon;
pub mod init;
pub mod metrics;
pub mod report;
pub mod verify;

// Re-export command handlers for convenience
pub use config::handle_config;
pub use daemon::{handle_daemon, handle_off, handle_on, handle_status};
pub use init::handle_init;
pub use metrics::handle_metrics;
pub use report::handle_report;
pub use verify::handle_verify;
