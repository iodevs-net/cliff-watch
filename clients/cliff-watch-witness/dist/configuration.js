"use strict";
/**
 * Configuration Management for Cliff-Watch Witness
 * Handles VSCode configuration settings and provides type-safe access
 */
Object.defineProperty(exports, "__esModule", { value: true });
exports.CONFIGURATION_SCHEMA = exports.ConfigurationManager = exports.PrivacyLevel = void 0;
const vscode = require("vscode");
/**
 * Privacy levels for metrics collection
 */
var PrivacyLevel;
(function (PrivacyLevel) {
    PrivacyLevel["Minimal"] = "minimal";
    PrivacyLevel["Standard"] = "standard";
    PrivacyLevel["Maximum"] = "maximum";
})(PrivacyLevel || (exports.PrivacyLevel = PrivacyLevel = {}));
/**
 * Default configuration values
 */
const DEFAULT_CONFIG = {
    enabled: true,
    humanScoreThreshold: 70,
    focusTimeTracking: true,
    privacyLevel: PrivacyLevel.Standard,
    statusBarUpdateInterval: 5000
};
/**
 * Configuration manager class
 */
class ConfigurationManager {
    constructor() {
        this.configChangeDisposable = null;
        this.config = this.loadConfig();
    }
    /**
     * Load configuration from VSCode settings
     */
    loadConfig() {
        const config = vscode.workspace.getConfiguration('cliffWatchWitness');
        return {
            enabled: config.get('enabled', DEFAULT_CONFIG.enabled),
            humanScoreThreshold: config.get('humanScoreThreshold', DEFAULT_CONFIG.humanScoreThreshold),
            focusTimeTracking: config.get('focusTimeTracking', DEFAULT_CONFIG.focusTimeTracking),
            privacyLevel: config.get('privacyLevel', DEFAULT_CONFIG.privacyLevel),
            statusBarUpdateInterval: config.get('statusBarUpdateInterval', DEFAULT_CONFIG.statusBarUpdateInterval)
        };
    }
    /**
     * Get current configuration
     */
    getConfig() {
        return { ...this.config };
    }
    /**
     * Update configuration from VSCode settings
     */
    refreshConfig() {
        this.config = this.loadConfig();
    }
    /**
     * Watch for configuration changes
     * @param callback Function to call when configuration changes
     */
    watchConfigChanges(callback) {
        this.configChangeDisposable = vscode.workspace.onDidChangeConfiguration((e) => {
            if (e.affectsConfiguration('cliffWatchWitness')) {
                this.refreshConfig();
                callback(this.getConfig());
            }
        });
    }
    /**
     * Dispose of configuration change watcher
     */
    dispose() {
        if (this.configChangeDisposable) {
            this.configChangeDisposable.dispose();
            this.configChangeDisposable = null;
        }
    }
}
exports.ConfigurationManager = ConfigurationManager;
/**
 * Configuration schema for VSCode settings
 * This should be added to package.json's contributes.configuration
 */
exports.CONFIGURATION_SCHEMA = {
    properties: {
        'cliffWatchWitness.enabled': {
            type: 'boolean',
            default: true,
            description: 'Enable or disable Cliff-Watch metrics collection'
        },
        'cliffWatchWitness.humanScoreThreshold': {
            type: 'number',
            default: 70,
            minimum: 0,
            maximum: 100,
            description: 'Human score threshold (0-100) for color coding in status bar'
        },
        'cliffWatchWitness.focusTimeTracking': {
            type: 'boolean',
            default: true,
            description: 'Enable or disable focus time tracking'
        },
        'cliffWatchWitness.privacyLevel': {
            type: 'string',
            enum: ['minimal', 'standard', 'maximum'],
            default: 'standard',
            description: 'Privacy level for metrics collection',
            enumDescriptions: [
                'Minimal: Collect only essential metrics',
                'Standard: Collect standard metrics for analysis',
                'Maximum: Collect detailed metrics for comprehensive analysis'
            ]
        },
        'cliffWatchWitness.statusBarUpdateInterval': {
            type: 'number',
            default: 5000,
            minimum: 1000,
            maximum: 60000,
            description: 'Status bar update interval in milliseconds'
        }
    }
};
//# sourceMappingURL=configuration.js.map