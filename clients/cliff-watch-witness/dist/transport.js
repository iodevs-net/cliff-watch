"use strict";
/**
 * Transport implementation for Cliff-Watch Witness
 *
 * NOTE: This is a stub implementation that maintains the ITransport interface
 * for future compatibility. The actual metrics processing is now handled
 * by MetricsEngine in-memory, eliminating the daemon dependency.
 */
Object.defineProperty(exports, "__esModule", { value: true });
exports.Transport = void 0;
/**
 * Stub Transport implementation
 * Maintains the ITransport interface for potential future use
 * but does not communicate with any daemon or socket
 */
class Transport {
    constructor() {
        this.errorCallback = null;
        this.disposed = false;
        // No daemon startup - metrics are now calculated in-memory by MetricsEngine
        console.log('[Transport] Using in-memory MetricsEngine (daemon dependency removed)');
    }
    send(event) {
        if (this.disposed) {
            console.warn('[Transport] Cannot send event - transport is disposed');
            return;
        }
        // Events are now processed by MetricsEngine directly
        // This stub exists only to maintain interface compatibility
    }
    onError(callback) {
        this.errorCallback = callback;
    }
    dispose() {
        this.disposed = true;
        console.log('[Transport] Disposed (no daemon cleanup needed)');
    }
}
exports.Transport = Transport;
//# sourceMappingURL=transport.js.map