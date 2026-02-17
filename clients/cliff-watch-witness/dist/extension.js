"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.activate = activate;
exports.deactivate = deactivate;
const vscode = require("vscode");
const metrics_1 = require("./metrics");
const utils_1 = require("./utils");
const config_1 = require("./config");
const configuration_1 = require("./configuration");
let metricsEngine;
let configManager;
let metricsDisplayInterval = null;
let statusBarItem = null;
/**
 * Format milliseconds to human-readable time string
 */
function formatTime(milliseconds) {
    const seconds = Math.floor(milliseconds / 1000);
    const minutes = Math.floor(seconds / 60);
    const hours = Math.floor(minutes / 60);
    if (hours > 0) {
        return `${hours}h ${minutes % 60}m`;
    }
    else if (minutes > 0) {
        return `${minutes}m ${seconds % 60}s`;
    }
    else {
        return `${seconds}s`;
    }
}
/**
 * Get color based on human score and threshold
 */
function getScoreColor(score, threshold) {
    const scorePercentage = score * 100;
    if (scorePercentage >= threshold) {
        return 'green';
    }
    else if (scorePercentage >= threshold * 0.7) {
        return 'orange';
    }
    else {
        return 'red';
    }
}
/**
 * Get icon based on human score
 */
function getScoreIcon(score) {
    const scorePercentage = score * 100;
    if (scorePercentage >= 80) {
        return '$(check-circle)';
    }
    else if (scorePercentage >= 60) {
        return '$(info)';
    }
    else if (scorePercentage >= 40) {
        return '$(warning)';
    }
    else {
        return '$(error)';
    }
}
/**
 * Create or update status bar item with metrics
 */
function updateStatusBar(metrics) {
    const config = configManager.getConfig();
    if (!config.enabled) {
        if (statusBarItem) {
            statusBarItem.text = '$(circle-slash) Cliff-Watch Disabled';
            statusBarItem.tooltip = 'Cliff-Watch metrics collection is disabled. Enable it in settings.';
            statusBarItem.show();
        }
        return;
    }
    if (!statusBarItem) {
        statusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 100);
    }
    const scorePercentage = metrics.humanScore * 100;
    const color = getScoreColor(metrics.humanScore, config.humanScoreThreshold);
    const icon = getScoreIcon(metrics.humanScore);
    statusBarItem.text = `${icon} Human Score: ${scorePercentage.toFixed(1)}%`;
    statusBarItem.backgroundColor = new vscode.ThemeColor(`statusBarItem.${color}Background`);
    statusBarItem.color = new vscode.ThemeColor(`statusBarItem.${color}Foreground`);
    // Create detailed tooltip
    const tooltip = new vscode.MarkdownString();
    tooltip.isTrusted = true;
    tooltip.supportHtml = true;
    tooltip.appendMarkdown(`## Cliff-Watch Metrics\n\n`);
    tooltip.appendMarkdown(`**Human Score:** ${scorePercentage.toFixed(1)}%\n\n`);
    tooltip.appendMarkdown(`**Component Scores:**\n`);
    tooltip.appendMarkdown(`- Burstiness: ${(metrics.burstinessScore * 100).toFixed(1)}%\n`);
    tooltip.appendMarkdown(`- NCD: ${(metrics.ncdScore * 100).toFixed(1)}%\n`);
    tooltip.appendMarkdown(`- Focus: ${(metrics.focusScore * 100).toFixed(1)}%\n\n`);
    tooltip.appendMarkdown(`**Activity Today:**\n`);
    tooltip.appendMarkdown(`- Focus Time: ${formatTime(metrics.totalFocusTime)}\n`);
    tooltip.appendMarkdown(`- Edit Bursts: ${metrics.editBurstsCount}\n`);
    tooltip.appendMarkdown(`- Navigation Events: ${metrics.navigationEventsCount}\n\n`);
    tooltip.appendMarkdown(`---\n`);
    tooltip.appendMarkdown(`*Privacy Level: ${config.privacyLevel}*\n`);
    tooltip.appendMarkdown(`*Threshold: ${config.humanScoreThreshold}%*`);
    statusBarItem.tooltip = tooltip;
    statusBarItem.command = 'cliffWatchWitness.showMetrics';
    statusBarItem.show();
}
/**
 * Display metrics in console and update status bar
 */
function displayMetrics() {
    if (!metricsEngine)
        return;
    const metrics = metricsEngine.getMetrics();
    const config = configManager.getConfig();
    console.log('[MetricsEngine] Current Metrics:', {
        burstinessScore: metrics.burstinessScore.toFixed(3),
        ncdScore: metrics.ncdScore.toFixed(3),
        focusScore: metrics.focusScore.toFixed(3),
        humanScore: metrics.humanScore.toFixed(3),
        totalFocusTime: `${(metrics.totalFocusTime / 1000).toFixed(1)}s`,
        editBurstsCount: metrics.editBurstsCount,
        navigationEventsCount: metrics.navigationEventsCount,
        enabled: config.enabled,
        privacyLevel: config.privacyLevel
    });
    updateStatusBar(metrics);
}
/**
 * Process sensor event if enabled
 */
function processEvent(event) {
    const config = configManager.getConfig();
    if (!config.enabled) {
        return;
    }
    // Check privacy level for focus tracking
    if (event.type === 'focus_gained' || event.type === 'focus_lost') {
        if (!config.focusTimeTracking) {
            return;
        }
    }
    if (metricsEngine) {
        metricsEngine.processEvent(event);
    }
}
/**
 * Handle configuration changes
 */
function handleConfigChange(newConfig) {
    console.log('[Cliff-Watch] Configuration changed:', newConfig);
    // Restart metrics display interval with new interval
    if (metricsDisplayInterval) {
        clearInterval(metricsDisplayInterval);
    }
    if (newConfig.enabled) {
        metricsDisplayInterval = setInterval(() => {
            displayMetrics();
        }, newConfig.statusBarUpdateInterval);
    }
    else {
        // Update status bar to show disabled state
        updateStatusBar({
            burstinessScore: 0,
            ncdScore: 0,
            focusScore: 0,
            humanScore: 0,
            totalFocusTime: 0,
            editBurstsCount: 0,
            navigationEventsCount: 0
        });
    }
}
/**
 * Show detailed metrics in output channel
 */
function showDetailedMetrics() {
    if (!metricsEngine) {
        vscode.window.showInformationMessage('Cliff-Watch is not active.');
        return;
    }
    const metrics = metricsEngine.getMetrics();
    const config = configManager.getConfig();
    const outputChannel = vscode.window.createOutputChannel('Cliff-Watch Metrics');
    outputChannel.appendLine('Cliff-Watch Metrics Report');
    outputChannel.appendLine('========================');
    outputChannel.appendLine('');
    outputChannel.appendLine(`Enabled: ${config.enabled}`);
    outputChannel.appendLine(`Privacy Level: ${config.privacyLevel}`);
    outputChannel.appendLine(`Focus Time Tracking: ${config.focusTimeTracking}`);
    outputChannel.appendLine('');
    outputChannel.appendLine('--- Component Scores ---');
    outputChannel.appendLine(`Human Score: ${(metrics.humanScore * 100).toFixed(2)}%`);
    outputChannel.appendLine(`  - Burstiness: ${(metrics.burstinessScore * 100).toFixed(2)}%`);
    outputChannel.appendLine(`  - NCD: ${(metrics.ncdScore * 100).toFixed(2)}%`);
    outputChannel.appendLine(`  - Focus: ${(metrics.focusScore * 100).toFixed(2)}%`);
    outputChannel.appendLine('');
    outputChannel.appendLine('--- Activity ---');
    outputChannel.appendLine(`Focus Time: ${formatTime(metrics.totalFocusTime)}`);
    outputChannel.appendLine(`Edit Bursts: ${metrics.editBurstsCount}`);
    outputChannel.appendLine(`Navigation Events: ${metrics.navigationEventsCount}`);
    outputChannel.appendLine('');
    outputChannel.appendLine('--- Configuration ---');
    outputChannel.appendLine(`Human Score Threshold: ${config.humanScoreThreshold}%`);
    outputChannel.appendLine(`Status Bar Update Interval: ${config.statusBarUpdateInterval}ms`);
    outputChannel.show();
}
function activate(context) {
    console.log('Cliff-Watch Witness is active');
    // Initialize configuration manager
    configManager = new configuration_1.ConfigurationManager();
    // Initialize MetricsEngine
    metricsEngine = new metrics_1.MetricsEngine();
    context.subscriptions.push({ dispose: () => metricsEngine.dispose() });
    context.subscriptions.push({ dispose: () => configManager.dispose() });
    // Register command to show detailed metrics
    context.subscriptions.push(vscode.commands.registerCommand('cliffWatchWitness.showMetrics', showDetailedMetrics));
    // Watch for configuration changes
    configManager.watchConfigChanges(handleConfigChange);
    // 2.1 Focus Tracking
    context.subscriptions.push(vscode.window.onDidChangeWindowState((e) => {
        const timestamp_ms = (0, utils_1.now)();
        if (e.focused) {
            const editor = vscode.window.activeTextEditor;
            const file_path = editor ? editor.document.fileName : null;
            // Validate file_path before creating focus event
            if (file_path) {
                const event = { type: 'focus_gained', file_path, timestamp_ms };
                processEvent(event);
            }
            else {
                console.warn('[Extension] focus_gained event skipped: no active text editor or file_path is null');
            }
        }
        else {
            const event = { type: 'focus_lost', timestamp_ms };
            processEvent(event);
        }
    }), vscode.window.onDidChangeActiveTextEditor((editor) => {
        const timestamp_ms = (0, utils_1.now)();
        if (editor) {
            const file_path = editor.document.fileName;
            // Validate file_path before creating focus event
            if (file_path) {
                const event = { type: 'focus_gained', file_path, timestamp_ms };
                processEvent(event);
            }
            else {
                console.warn('[Extension] focus_gained event skipped: file_path is null');
            }
        }
    }));
    // 2.2 Edit Tracking (Bursting)
    let editBurstAccumulator = 0;
    let editBurstTimeout = null;
    let currentEditFile = null;
    context.subscriptions.push(vscode.workspace.onDidChangeTextDocument((e) => {
        if (e.document.uri.scheme !== 'file')
            return;
        // CNS v3.0 Pareto Filtering
        // Ignoramos Undo/Redo para no contaminar la métrica de originalidad humana
        if (e.reason === vscode.TextDocumentChangeReason.Undo || e.reason === vscode.TextDocumentChangeReason.Redo) {
            return;
        }
        const timestamp_ms = (0, utils_1.now)();
        const delta = e.contentChanges.reduce((acc, change) => {
            return acc + (change.text.length - change.rangeLength);
        }, 0);
        // CNS v3.0: Detección de Pegado Probable
        // Si un solo cambio inserta más de PASTE_THRESHOLD_CHARS caracteres, o si la ráfaga es sospechosamente rápida.
        const is_likely_paste = e.contentChanges.some(c => c.text.length > config_1.PASTE_THRESHOLD_CHARS);
        // Enviamos evento de tecleo atómico solo si es un cambio pequeño (tecleo real)
        // para habilitar análisis de latencia física en el MetricsEngine.
        if (e.contentChanges.length === 1 && e.contentChanges[0].text.length === 1 && !is_likely_paste) {
            processEvent({
                type: 'keystroke',
                file_path: e.document.fileName,
                timestamp_ms,
                metadata: { char: e.contentChanges[0].text }
            });
        }
        if (editBurstTimeout) {
            clearTimeout(editBurstTimeout);
        }
        editBurstTimeout = setTimeout(() => {
            flushEditBurst();
        }, config_1.EDIT_BURST_TIMEOUT_MS);
    }));
    function flushEditBurst() {
        if (editBurstAccumulator === 0 || !currentEditFile)
            return;
        const event = {
            type: 'edit_burst',
            file_path: currentEditFile,
            chars_delta: editBurstAccumulator,
            timestamp_ms: (0, utils_1.now)(),
            metadata: {
                is_likely_paste: editBurstAccumulator > config_1.EDIT_BURST_PASTE_THRESHOLD // Si la ráfaga acumulada es muy grande
            }
        };
        processEvent(event);
        editBurstAccumulator = 0;
    }
    // 2.3 Navigation Tracking
    let lastScrollTime = 0;
    context.subscriptions.push(vscode.window.onDidChangeTextEditorVisibleRanges((e) => {
        const currentTime = (0, utils_1.now)();
        if (currentTime - lastScrollTime < config_1.SCROLL_THROTTLE_MS)
            return; // Throttle 1s
        processEvent({
            type: 'navigation',
            file_path: e.textEditor.document.fileName,
            nav_type: 'scroll',
            timestamp_ms: currentTime
        });
        lastScrollTime = currentTime;
    }), 
    // Detección de Foco de Lectura por Hover
    vscode.languages.registerHoverProvider('*', {
        provideHover(document, position, token) {
            processEvent({
                type: 'navigation',
                file_path: document.fileName,
                nav_type: 'hover',
                timestamp_ms: (0, utils_1.now)()
            });
            return null; // No interferimos con otros hovers
        }
    }));
    // Detección de Navegación (Go to Definition / Navegación interna)
    context.subscriptions.push(vscode.window.onDidChangeTextEditorSelection((e) => {
        if (e.kind === vscode.TextEditorSelectionChangeKind.Command) {
            // Probablemente un comando de navegación (Go to definition, etc)
            processEvent({
                type: 'navigation',
                file_path: e.textEditor.document.fileName,
                nav_type: 'go_to_definition',
                timestamp_ms: (0, utils_1.now)()
            });
        }
    }));
    // Heartbeat v3.0: Identificamos si el sensor está "vivo" y en qué versión.
    const heartbeatInterval = setInterval(() => {
        processEvent({
            type: 'heartbeat',
            timestamp_ms: (0, utils_1.now)()
        });
    }, config_1.HEARTBEAT_INTERVAL_MS); // 15s para mayor resolución en el MetricsEngine
    context.subscriptions.push({ dispose: () => clearInterval(heartbeatInterval) });
    // Display metrics periodically
    const config = configManager.getConfig();
    metricsDisplayInterval = setInterval(() => {
        displayMetrics();
    }, config.statusBarUpdateInterval);
    context.subscriptions.push({ dispose: () => {
            if (metricsDisplayInterval)
                clearInterval(metricsDisplayInterval);
        } });
    // Initial status bar update
    displayMetrics();
}
function deactivate() {
    processEvent({ type: 'disconnect', timestamp_ms: (0, utils_1.now)() });
    if (metricsEngine) {
        metricsEngine.flush();
    }
    if (statusBarItem) {
        statusBarItem.dispose();
    }
}
//# sourceMappingURL=extension.js.map