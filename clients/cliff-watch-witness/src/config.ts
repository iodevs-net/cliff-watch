/**
 * Configuration constants for Cliff-Watch Witness
 * Centralized configuration to avoid magic numbers throughout the codebase
 */

// Socket communication
export const SOCKET_PATH = '/tmp/cliff-watch-sensor.sock';

// Scroll throttle in milliseconds (1 second)
export const SCROLL_THROTTLE_MS = 1000;

// Paste threshold in characters (single change > this is likely a paste)
export const PASTE_THRESHOLD_CHARS = 30;

// Edit burst timeout in milliseconds (500ms)
export const EDIT_BURST_TIMEOUT_MS = 500;

// Edit burst paste threshold (accumulated chars > this is likely a paste)
export const EDIT_BURST_PASTE_THRESHOLD = 100;

// Heartbeat interval in milliseconds (15 seconds)
export const HEARTBEAT_INTERVAL_MS = 15000;

// Maximum buffer size for events when disconnected
export const MAX_BUFFER_SIZE = 100;

// Retry initial delay in milliseconds (1 second)
export const RETRY_INITIAL_DELAY_MS = 1000;

// Retry maximum delay in milliseconds (30 seconds)
export const RETRY_MAX_DELAY_MS = 30000;

// Daemon startup delay in milliseconds (500ms)
export const DAEMON_STARTUP_DELAY_MS = 500;
