/**
 * Transport interface for Cliff-Watch Witness
 * Defines the contract for event transport implementations
 */

import { SensorEvent } from './types';

/**
 * Interface for transporting sensor events to the daemon
 */
export interface ITransport {
    /**
     * Send a sensor event to the daemon
     * @param event - The sensor event to send
     */
    send(event: SensorEvent): void;

    /**
     * Register a callback for error handling
     * @param callback - Function to call when an error occurs
     */
    onError(callback: (error: Error) => void): void;

    /**
     * Dispose of the transport and clean up resources
     */
    dispose(): void;
}
