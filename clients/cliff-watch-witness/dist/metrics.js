"use strict";
/**
 * Metrics Engine for Cliff-Watch Witness
 * Calculates metrics in-memory without daemon dependency
 * Optimized with memoization and debouncing
 */
Object.defineProperty(exports, "__esModule", { value: true });
exports.MetricsEngine = void 0;
/**
 * Metrics Engine Class
 * Tracks and calculates metrics in-memory using VSCode APIs
 * Optimized with memoization and debouncing
 */
class MetricsEngine {
    constructor() {
        this.lastBurstTime = null;
        // Memoization cache with TTL (time-to-live) of 1000ms
        this.memoCache = {
            burstiness: { value: 0, timestamp: 0 },
            ncd: { value: 0, timestamp: 0 },
            focus: { value: 0, timestamp: 0 }
        };
        // Debounce state
        this.debounce = {
            timer: null,
            pendingEvents: []
        };
        // Debounce delay in milliseconds
        this.DEBOUNCE_DELAY_MS = 100;
        // Memoization TTL in milliseconds
        this.MEMO_TTL_MS = 1000;
        this.state = {
            editBursts: [],
            burstIntervals: [],
            editHistory: [],
            focusStartTime: null,
            totalFocusTime: 0,
            focusSessions: [],
            burstinessScore: 0,
            ncdScore: 0,
            focusScore: 0,
            keystrokeTimestamps: [],
            navigationEvents: 0
        };
    }
    /**
     * Process a sensor event and update metrics state
     * Uses debouncing for rapid events
     */
    processEvent(event) {
        // Add event to debounce queue
        this.debounce.pendingEvents.push(event);
        // Clear existing timer
        if (this.debounce.timer !== null) {
            clearTimeout(this.debounce.timer);
        }
        // Set new timer to process batched events
        this.debounce.timer = setTimeout(() => {
            this.processBatchedEvents();
        }, this.DEBOUNCE_DELAY_MS);
    }
    /**
     * Process batched events from debounce queue
     */
    processBatchedEvents() {
        const events = this.debounce.pendingEvents;
        this.debounce.pendingEvents = [];
        this.debounce.timer = null;
        // Process each event
        for (const event of events) {
            this.processEventInternal(event);
        }
    }
    /**
     * Internal event processing without debouncing
     */
    processEventInternal(event) {
        switch (event.type) {
            case 'focus_gained':
                this.handleFocusGained(event);
                break;
            case 'focus_lost':
                this.handleFocusLost(event);
                break;
            case 'edit_burst':
                this.handleEditBurst(event);
                break;
            case 'keystroke':
                this.handleKeystroke(event);
                break;
            case 'navigation':
                this.handleNavigation(event);
                break;
            case 'heartbeat':
                // Heartbeats are used for liveness detection, no metric calculation needed
                break;
            case 'disconnect':
                // Handle disconnect - flush any pending state
                this.flush();
                break;
        }
    }
    /**
     * Handle focus gained event
     */
    handleFocusGained(event) {
        this.state.focusStartTime = event.timestamp_ms;
    }
    /**
     * Handle focus lost event
     */
    handleFocusLost(event) {
        if (this.state.focusStartTime !== null) {
            const sessionDuration = event.timestamp_ms - this.state.focusStartTime;
            this.state.totalFocusTime += sessionDuration;
            this.state.focusSessions.push({
                start: this.state.focusStartTime,
                end: event.timestamp_ms
            });
            this.state.focusStartTime = null;
            // Invalidate memo cache for focus score
            this.invalidateMemoCache('focus');
        }
    }
    /**
     * Handle edit burst event
     */
    handleEditBurst(event) {
        const burstData = {
            timestamp: event.timestamp_ms,
            chars_delta: event.chars_delta,
            is_likely_paste: event.metadata?.is_likely_paste || false
        };
        this.state.editBursts.push(burstData);
        // Calculate burst interval if we have a previous burst
        if (this.lastBurstTime !== null) {
            const interval = event.timestamp_ms - this.lastBurstTime;
            this.state.burstIntervals.push(interval);
        }
        this.lastBurstTime = event.timestamp_ms;
        // Simplified NCD: track edit pattern
        // In a real implementation, this would use compression algorithms
        // For IDE context, we use a simplified approach based on edit patterns
        this.state.editHistory.push(`edit:${event.chars_delta}:${event.timestamp_ms}`);
        if (this.state.editHistory.length > 100) {
            this.state.editHistory.shift(); // Keep last 100 edits
        }
        // Invalidate memo caches
        this.invalidateMemoCache('burstiness');
        this.invalidateMemoCache('ncd');
    }
    /**
     * Handle keystroke event
     */
    handleKeystroke(event) {
        this.state.keystrokeTimestamps.push(event.timestamp_ms);
        // Keep only last 100 keystrokes for analysis
        if (this.state.keystrokeTimestamps.length > 100) {
            this.state.keystrokeTimestamps.shift();
        }
    }
    /**
     * Handle navigation event
     */
    handleNavigation(event) {
        this.state.navigationEvents++;
    }
    /**
     * Invalidate memoization cache for a specific metric
     */
    invalidateMemoCache(metric) {
        this.memoCache[metric].timestamp = 0;
    }
    /**
     * Check if memo cache is valid
     */
    isCacheValid(cacheEntry) {
        return Date.now() - cacheEntry.timestamp < this.MEMO_TTL_MS;
    }
    /**
     * Calculate burstiness score (0-1) with memoization
     * Measures variability in edit burst patterns
     */
    calculateBurstiness() {
        // Check cache first
        if (this.isCacheValid(this.memoCache.burstiness)) {
            return this.memoCache.burstiness.value;
        }
        if (this.state.editBursts.length < 2) {
            this.memoCache.burstiness.value = 0;
            this.memoCache.burstiness.timestamp = Date.now();
            return 0; // Not enough data
        }
        // Calculate mean and standard deviation of burst intervals
        const intervals = this.state.burstIntervals;
        const mean = intervals.reduce((sum, val) => sum + val, 0) / intervals.length;
        const variance = intervals.reduce((sum, val) => sum + Math.pow(val - mean, 2), 0) / intervals.length;
        const stdDev = Math.sqrt(variance);
        // Burstiness = coefficient of variation (stdDev / mean)
        // Normalize to 0-1 range (typical values are 0.1-2.0)
        const cv = mean > 0 ? stdDev / mean : 0;
        // Apply sigmoid-like normalization to get 0-1 score
        // Lower burstiness (more consistent) = higher human-like score
        // Higher burstiness (more irregular) = lower human-like score
        const normalizedBurstiness = 1 / (1 + Math.exp(-2 * (cv - 0.5)));
        // Invert: we want low burstiness to be high score
        const result = 1 - normalizedBurstiness;
        // Cache the result
        this.memoCache.burstiness.value = result;
        this.memoCache.burstiness.timestamp = Date.now();
        return result;
    }
    /**
     * Calculate Normalized Compression Distance (NCD) score (0-1) with memoization
     * Simplified for IDE context using edit pattern analysis
     */
    calculateNCD() {
        // Check cache first
        if (this.isCacheValid(this.memoCache.ncd)) {
            return this.memoCache.ncd.value;
        }
        if (this.state.editHistory.length < 2) {
            this.memoCache.ncd.value = 0;
            this.memoCache.ncd.timestamp = Date.now();
            return 0; // Not enough data
        }
        // Simplified NCD calculation based on edit pattern similarity
        // In a full implementation, this would use actual compression algorithms
        // For IDE context, we analyze edit patterns for consistency
        const edits = this.state.editHistory;
        let patternSimilarity = 0;
        // Analyze consecutive edit pairs for pattern similarity
        for (let i = 1; i < edits.length; i++) {
            const prev = edits[i - 1];
            const curr = edits[i];
            // Simple similarity check: similar edit sizes indicate consistent patterns
            const prevSize = parseInt(prev.split(':')[1]);
            const currSize = parseInt(curr.split(':')[1]);
            if (prevSize > 0 && currSize > 0) {
                const ratio = Math.min(prevSize, currSize) / Math.max(prevSize, currSize);
                patternSimilarity += ratio;
            }
        }
        // Normalize to 0-1
        const avgSimilarity = patternSimilarity / (edits.length - 1);
        // Higher similarity = higher human-like score
        this.memoCache.ncd.value = avgSimilarity;
        this.memoCache.ncd.timestamp = Date.now();
        return avgSimilarity;
    }
    /**
     * Calculate focus score (0-1) with memoization
     * Based on total focus time and session patterns
     */
    calculateFocusScore() {
        // Check cache first
        if (this.isCacheValid(this.memoCache.focus)) {
            return this.memoCache.focus.value;
        }
        // Normalize focus time to 0-1 based on reasonable working session
        // Assuming 8 hours (28800000 ms) is a full day of work
        const maxFocusTime = 28800000;
        const normalizedFocus = Math.min(this.state.totalFocusTime / maxFocusTime, 1);
        // Consider session count - more sessions might indicate fragmentation
        const sessionCount = this.state.focusSessions.length;
        const sessionPenalty = sessionCount > 0 ? Math.min(sessionCount / 20, 0.3) : 0;
        const result = Math.max(0, normalizedFocus - sessionPenalty);
        // Cache the result
        this.memoCache.focus.value = result;
        this.memoCache.focus.timestamp = Date.now();
        return result;
    }
    /**
     * Calculate human score (0-1)
     * Formula: 0.4 * Burstiness + 0.4 * NCD + 0.2 * Focus
     */
    calculateHumanScore() {
        const B = this.calculateBurstiness();
        const N = this.calculateNCD();
        const F = this.calculateFocusScore();
        // Weighted formula: 0.4*B + 0.4*N + 0.2*F
        const humanScore = 0.4 * B + 0.4 * N + 0.2 * F;
        return Math.max(0, Math.min(1, humanScore));
    }
    /**
     * Update focus time (called periodically or on focus events)
     */
    updateFocusTime() {
        // If currently focused, update the total focus time
        if (this.state.focusStartTime !== null) {
            const currentFocusDuration = Date.now() - this.state.focusStartTime;
            // We don't add this to total yet - it's added on focus_lost
            // This method is for periodic updates if needed
        }
        // Invalidate memo cache for focus score
        this.invalidateMemoCache('focus');
    }
    /**
     * Get current metrics state
     */
    getMetrics() {
        return {
            burstinessScore: this.calculateBurstiness(),
            ncdScore: this.calculateNCD(),
            focusScore: this.calculateFocusScore(),
            humanScore: this.calculateHumanScore(),
            totalFocusTime: this.state.totalFocusTime,
            editBurstsCount: this.state.editBursts.length,
            navigationEventsCount: this.state.navigationEvents
        };
    }
    /**
     * Flush and reset metrics state
     */
    flush() {
        // Process any pending debounced events
        if (this.debounce.timer !== null) {
            clearTimeout(this.debounce.timer);
            this.processBatchedEvents();
        }
        if (this.state.focusStartTime !== null) {
            const sessionDuration = Date.now() - this.state.focusStartTime;
            this.state.totalFocusTime += sessionDuration;
            this.state.focusSessions.push({
                start: this.state.focusStartTime,
                end: Date.now()
            });
            this.state.focusStartTime = null;
            this.invalidateMemoCache('focus');
        }
    }
    /**
     * Reset all metrics state
     */
    reset() {
        // Clear debounce timer
        if (this.debounce.timer !== null) {
            clearTimeout(this.debounce.timer);
            this.debounce.timer = null;
        }
        this.state = {
            editBursts: [],
            burstIntervals: [],
            editHistory: [],
            focusStartTime: null,
            totalFocusTime: 0,
            focusSessions: [],
            burstinessScore: 0,
            ncdScore: 0,
            focusScore: 0,
            keystrokeTimestamps: [],
            navigationEvents: 0
        };
        this.lastBurstTime = null;
        // Invalidate all memo caches
        this.memoCache = {
            burstiness: { value: 0, timestamp: 0 },
            ncd: { value: 0, timestamp: 0 },
            focus: { value: 0, timestamp: 0 }
        };
        this.debounce.pendingEvents = [];
    }
    /**
     * Dispose of the metrics engine and clean up resources
     */
    dispose() {
        this.flush();
        // Clear debounce timer
        if (this.debounce.timer !== null) {
            clearTimeout(this.debounce.timer);
            this.debounce.timer = null;
        }
    }
}
exports.MetricsEngine = MetricsEngine;
//# sourceMappingURL=metrics.js.map