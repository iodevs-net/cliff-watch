/**
 * Configuration Management for Cliff-Watch Witness
 * Handles VSCode configuration settings and provides type-safe access
 */

import * as vscode from 'vscode';

/**
 * Privacy levels for metrics collection
 */
export enum PrivacyLevel {
    Minimal = 'minimal',
    Standard = 'standard',
    Maximum = 'maximum'
}

/**
 * Configuration interface for Cliff-Watch Witness
 */
export interface CliffWatchConfig {
    /** Enable/disable metrics collection */
    enabled: boolean;
    /** Human score threshold (0-100) - affects color coding */
    humanScoreThreshold: number;
    /** Enable/disable focus time tracking */
    focusTimeTracking: boolean;
    /** Privacy level for data collection */
    privacyLevel: PrivacyLevel;
    /** Status bar update interval in milliseconds */
    statusBarUpdateInterval: number;
}

/**
 * Default configuration values
 */
const DEFAULT_CONFIG: CliffWatchConfig = {
    enabled: true,
    humanScoreThreshold: 70,
    focusTimeTracking: true,
    privacyLevel: PrivacyLevel.Standard,
    statusBarUpdateInterval: 5000
};

/**
 * Configuration manager class
 */
export class ConfigurationManager {
    private config: CliffWatchConfig;
    private configChangeDisposable: vscode.Disposable | null = null;

    constructor() {
        this.config = this.loadConfig();
    }

    /**
     * Load configuration from VSCode settings
     */
    private loadConfig(): CliffWatchConfig {
        const config = vscode.workspace.getConfiguration('cliffWatchWitness');
        return {
            enabled: config.get<boolean>('enabled', DEFAULT_CONFIG.enabled),
            humanScoreThreshold: config.get<number>('humanScoreThreshold', DEFAULT_CONFIG.humanScoreThreshold),
            focusTimeTracking: config.get<boolean>('focusTimeTracking', DEFAULT_CONFIG.focusTimeTracking),
            privacyLevel: config.get<PrivacyLevel>('privacyLevel', DEFAULT_CONFIG.privacyLevel),
            statusBarUpdateInterval: config.get<number>('statusBarUpdateInterval', DEFAULT_CONFIG.statusBarUpdateInterval)
        };
    }

    /**
     * Get current configuration
     */
    public getConfig(): CliffWatchConfig {
        return { ...this.config };
    }

    /**
     * Update configuration from VSCode settings
     */
    public refreshConfig(): void {
        this.config = this.loadConfig();
    }

    /**
     * Watch for configuration changes
     * @param callback Function to call when configuration changes
     */
    public watchConfigChanges(callback: (newConfig: CliffWatchConfig) => void): void {
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
    public dispose(): void {
        if (this.configChangeDisposable) {
            this.configChangeDisposable.dispose();
            this.configChangeDisposable = null;
        }
    }
}

/**
 * Configuration schema for VSCode settings
 * This should be added to package.json's contributes.configuration
 */
export const CONFIGURATION_SCHEMA = {
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
