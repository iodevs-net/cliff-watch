"use strict";
/**
 * Configuration constants for Cliff-Watch Witness
 * Centralized configuration to avoid magic numbers throughout the codebase
 */
Object.defineProperty(exports, "__esModule", { value: true });
exports.DAEMON_STARTUP_DELAY_MS = exports.RETRY_MAX_DELAY_MS = exports.RETRY_INITIAL_DELAY_MS = exports.MAX_BUFFER_SIZE = exports.HEARTBEAT_INTERVAL_MS = exports.EDIT_BURST_PASTE_THRESHOLD = exports.EDIT_BURST_TIMEOUT_MS = exports.PASTE_THRESHOLD_CHARS = exports.SCROLL_THROTTLE_MS = exports.SOCKET_PATH = void 0;
// Socket communication
exports.SOCKET_PATH = '/tmp/cliff-watch-sensor.sock';
// Scroll throttle in milliseconds (1 second)
exports.SCROLL_THROTTLE_MS = 1000;
// Paste threshold in characters (single change > this is likely a paste)
exports.PASTE_THRESHOLD_CHARS = 30;
// Edit burst timeout in milliseconds (500ms)
exports.EDIT_BURST_TIMEOUT_MS = 500;
// Edit burst paste threshold (accumulated chars > this is likely a paste)
exports.EDIT_BURST_PASTE_THRESHOLD = 100;
// Heartbeat interval in milliseconds (15 seconds)
exports.HEARTBEAT_INTERVAL_MS = 15000;
// Maximum buffer size for events when disconnected
exports.MAX_BUFFER_SIZE = 100;
// Retry initial delay in milliseconds (1 second)
exports.RETRY_INITIAL_DELAY_MS = 1000;
// Retry maximum delay in milliseconds (30 seconds)
exports.RETRY_MAX_DELAY_MS = 30000;
// Daemon startup delay in milliseconds (500ms)
exports.DAEMON_STARTUP_DELAY_MS = 500;
//# sourceMappingURL=config.js.map