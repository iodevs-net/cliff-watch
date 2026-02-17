/**
 * Transport implementation for Cliff-Watch Witness
 * 
 * NOTE: This is a stub implementation that maintains the ITransport interface
 * for future compatibility. The actual metrics processing is now handled
 * by MetricsEngine in-memory, eliminating the daemon dependency.
 */

import { SensorEvent } from './types';
import { ITransport } from './transport-interface';

/**
 * Stub Transport implementation
 * Maintains the ITransport interface for potential future use
 * but does not communicate with any daemon or socket
 */
export class Transport implements ITransport {
    private errorCallback: ((error: Error) => void) | null = null;
    private disposed: boolean = false;

    constructor() {
        // No daemon startup - metrics are now calculated in-memory by MetricsEngine
        console.log('[Transport] Using in-memory MetricsEngine (daemon dependency removed)');
    }

    public send(event: SensorEvent): void {
        if (this.disposed) {
            console.warn('[Transport] Cannot send event - transport is disposed');
            return;
        }
        // Events are now processed by MetricsEngine directly
        // This stub exists only to maintain interface compatibility
    }

    public onError(callback: (error: Error) => void): void {
        this.errorCallback = callback;
    }

    public dispose(): void {
        this.disposed = true;
        console.log('[Transport] Disposed (no daemon cleanup needed)');
    }
}
