/**
 * Utility functions for Cliff-Watch Witness
 */

/**
 * Get the current timestamp in milliseconds
 * Centralized timestamp function to enable easier testing and potential future adjustments
 * @returns Current timestamp in milliseconds since Unix epoch
 */
export function now(): number {
    return Date.now();
}
