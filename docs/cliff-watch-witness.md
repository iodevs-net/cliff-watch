# Cliff-Watch Witness VSCode Extension Guide

> **Version**: 3.0.0 (Non-Intrusive)  
> **Engine Support**: VSCode ^1.108.1  
> **Publisher**: iodevs

The Cliff-Watch Witness is a VSCode extension that serves as the "Sovereign Witness" for the Proof of Human Work (PoHW) protocol. It monitors user activity in the IDE to generate cryptographic proof of human authorship while maintaining privacy through local processing and filtering.

---

## Table of Contents

1. [Installation](#installation)
2. [Configuration](#configuration)
3. [Features](#features)
4. [Architecture](#architecture)
5. [Usage](#usage)
6. [Troubleshooting](#troubleshooting)
7. [Development](#development)

---

## Installation

### From VSIX (Recommended)

The extension is distributed as a `.vsix` package.

1. Download the latest `cliff-watch-witness-v*.vsix` from the [releases](https://github.com/iodevs-net/cliff-watch/releases)
2. In VSCode:
   - Open the Extensions view: `Ctrl+Shift+X` (or `Cmd+Shift+X` on macOS)
   - Click the `...` menu in the top-right corner
   - Select "Install from VSIX..."
   - Navigate to the downloaded `.vsix` file

### From Source

```bash
# Navigate to the extension directory
cd clients/cliff-watch-witness

# Install dependencies
npm install

# Compile TypeScript
npm run compile

# Package as VSIX (requires vsce)
npx vsce package
```

---

## Configuration

### Extension Settings (VSCode)

The extension can be configured directly in VSCode Settings:

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `cliff-watch.enabled` | boolean | `true` | Enable/disable monitoring |
| `cliff-watch.difficulty` | string | `"Normal"` | Easy, Normal, Hardcore |
| `cliff-watch.auditMode` | boolean | `true` | Warn instead of blocking |
| `cliff-watch.minEntropy` | number | `2.5` | Minimum entropy threshold |
| `cliff-watch.debounceMs` | number | `500` | Debounce window in ms |
| `cliff-watch.heartbeatSec` | number | `15` | Heartbeat interval in seconds |

### Configuration File

You can also configure via `cliff-watch.toml` in your project or home directory:

```toml
[governance]
difficulty = "Normal"        # Easy, Normal, Hardcore
min_entropy = 2.5           # Minimum entropy (bits/byte)
audit_mode = true            # Warn only, don't block

[monitoring]
debounce_window_ms = 500
ignore_extensions = ["log", "lock", "tmp", "json"]
```

### Daemon Connection (Optional)

The extension can work standalone or communicate with the CLI daemon via IPC. To use with daemon:

```bash
# Start the daemon (optional)
cliff-watch daemon

# Or use the CLI
cliff-watch on
```

---

## Features

### 1. Focus Tracking ✅

Monitors window focus events to detect when the user is actively working in VSCode:

- **Focus Gained**: User opened a file or returned to VSCode
- **Focus Lost**: User switched to another application

### 2. Edit Tracking (Burst Detection) ✅

Analyzes edit patterns to detect human typing characteristics:

- **Atomic Keystrokes**: Captures individual keystrokes (< 2 chars) for latency analysis
- **Edit Bursts**: Groups edits within 500ms windows
- **Paste Detection**: Flags suspiciously large edits (> 30 chars) as likely paste operations
- **Undo/Redo Filtering**: Ignores undo/redo operations to prevent metric pollution

### 3. Navigation Tracking ✅

Tracks file navigation patterns:

- **Scroll Events**: Monitors scrolling through files
- **Hover Events**: Detects code inspection activities
- **Go to Definition**: Tracks code navigation commands

### 4. Heartbeat ✅

Sends periodic heartbeat events (every 15 seconds) to verify the sensor is active and functional.

### 5. Privacy Filtering (v3.0) ✅

New in v3.0 - Privacy-preserving telemetry:

- **File Path Hashing**: SHA-256 hashed paths (no full paths transmitted)
- **Timestamp Bucketing**: 5-second precision (configurable)
- **Content Sanitization**: Only metadata, no raw content
- **Local Processing**: All metrics computed client-side

### 6. Configuration UI (v3.0) ✅

- Visual settings panel in VSCode
- Real-time settings synchronization
- Difficulty presets

---

## Architecture

### Non-Intrusive Design (v3.0)

```
┌─────────────────────────────────────────────────────────────┐
│                     VSCode Extension                          │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │ Focus Tracker│  │ Edit Tracker │  │ Navigation Track.│  │
│  └──────┬───────┘  └──────┬───────┘  └────────┬─────────┘  │
│         │                 │                    │             │
│         └────────────────┴────────────────────┘             │
│                          │                                   │
│                    ┌─────┴─────┐                            │
│                    │Privacy    │                            │
│                    │Filter     │                            │
│                    └─────┬─────┘                            │
│                          │                                   │
│                    ┌─────┴─────┐                            │
│                    │Transport  │                            │
│                    └─────┬─────┘                            │
└──────────────────────────┼──────────────────────────────────┘
                           │
                           │ IPC (Unix Socket / Named Pipe)
                           │ OR Standalone (Local Only)
                           │
┌──────────────────────────┼──────────────────────────────────┐
│                    ┌─────┴─────┐                            │
│                    │   Daemon  │   (Optional)              │
│                    │ (Optional)│                            │
│                    └───────────┘                            │
│                   Cliff-Watch Daemon                         │
└──────────────────────────────────────────────────────────────┘
```

### Module: `extension.ts`

The main extension entry point. Handles:

- Activation/deactivation lifecycle
- Event subscription management
- Focus, edit, and navigation tracking
- Heartbeat generation
- Privacy filtering integration

### Module: `transport.ts`

Handles communication with the Cliff-Watch daemon (optional):

- Connection management
- Event serialization
- Error handling and reconnection
- Fallback to standalone mode

### Module: `privacy.ts` (v3.0)

New privacy filtering module:

- SHA-256 file path hashing
- Timestamp bucketing
- Content sanitization
- Privacy metrics aggregation

### Module: `metrics.ts` (v3.0)

Local metrics computation:

- Burstiness calculation
- Entropy estimation
- Focus time tracking
- Human score estimation

### Module: `types.ts`

TypeScript type definitions for sensor events:

```typescript
type SensorEvent = 
  | { type: 'focus_gained'; file_path: string | null; timestamp_ms: number }
  | { type: 'focus_lost'; timestamp_ms: number }
  | { type: 'keystroke'; file_path: string; timestamp_ms: number; metadata: { char: string } }
  | { type: 'edit_burst'; file_path: string; chars_delta: number; timestamp_ms: number; metadata: { is_likely_paste: boolean } }
  | { type: 'navigation'; file_path: string; nav_type: NavigationType; timestamp_ms: number }
  | { type: 'heartbeat'; timestamp_ms: number }
  | { type: 'disconnect'; timestamp_ms: number };

// Privacy-filtered event
type SanitizedEvent = {
  type: SensorEvent['type'];
  file_path_hash?: string;  // SHA-256, not full path
  timestamp_bucket: number;  // Bucketed to 5s
  // ... other filtered fields
};

type NavigationType = 'scroll' | 'hover' | 'go_to_definition';
```

---

## Usage

### Initial Setup

1. Install the VSCode extension (see [Installation](#installation))
2. (Optional) Build and install the Cliff-Watch CLI:

```bash
# Build the project
cargo build --release

# Add to PATH
export PATH=$PATH:$(pwd)/target/release

# Initialize in your repository (optional)
cliff-watch init
```

### Normal Workflow

1. Open VSCode and start coding normally
2. The extension automatically tracks your activity
3. When you're ready to commit:

```bash
git commit -m "Your commit message"
```

4. The commit hook will verify your "Humanity Score" based on the telemetry collected by the extension

### Viewing Metrics

Check your current metrics:

```bash
cliff-watch metrics
```

Example output:
```
GovMonitor - Estado Termodinámico v2.1:
  🔋 Energía (Kinética+Foco): 85.0%
  🧠 Acoplamiento Cognitivo:  0.92
  🛡️  Human Probability:     88.0%
```

---

## Troubleshooting

### Extension Not Activating

1. Check VSCode version (must be >= 1.108.1)
2. Open Developer Tools (`Help > Toggle Developer Tools`)
3. Check the Console for errors

### Daemon Connection Issues

If using daemon mode:

1. Ensure the daemon is running: `cliff-watch daemon`
2. Check the daemon logs for connection errors
3. Verify the IPC socket exists (Linux: `/tmp/cliff-watch.sock`)

**Note**: The extension works standalone without the daemon. Metrics are computed locally.

### False Positives (Blocked Commits)

If your commits are being blocked unexpectedly:

1. Check your Humanity Score: `cliff-watch metrics`
2. Try increasing the battery by spending more time in focused coding sessions
3. Enable audit mode for warnings instead of blocks:

```toml
[governance]
audit_mode = true
```

### Privacy Concerns

If you have privacy concerns:

1. The extension only sends hashed file paths (SHA-256)
2. Timestamps are bucketed to 5-second intervals
3. No raw content is ever transmitted
4. All processing happens locally

### Performance Issues

If VSCode feels slow:

1. Check extension host CPU usage in Task Manager
2. Disable other extensions temporarily
3. Increase debounce window in settings

---

## Development

### Building from Source

```bash
# Clone the repository
git clone https://github.com/iodevs-net/cliff-watch.git
cd cliff-watch/clients/cliff-watch-witness

# Install dependencies
npm install

# Watch mode for development
npm run watch

# Run tests
npm test
```

### Project Structure

```
cliff-watch-witness/
├── package.json           # Extension manifest
├── src/
│   ├── extension.ts      # Main entry point
│   ├── transport.ts      # Daemon communication
│   ├── privacy.ts        # Privacy filtering (v3.0)
│   ├── metrics.ts        # Local metrics (v3.0)
│   ├── configuration.ts  # Configuration UI (v3.0)
│   └── types.ts          # TypeScript definitions
├── tests/
│   ├── certification.test.ts
│   └── integration.test.ts
└── bin/
    └── cliff-watch-daemon-linux-x64  # Bundled daemon (optional)
```

### Testing

```bash
# Run all tests
npm test

# Run with coverage
npm test -- --coverage
```

---

## Security & Privacy

### Data Collection

The extension collects:

- **Timestamps**: When you focus, type, or navigate (bucketed to 5s)
- **File path hashes**: SHA-256 of file paths (not full paths)
- **Edit sizes**: Character count changes only
- **Navigation types**: Scroll, hover, goto

The extension does **NOT** collect:

- **Keystroke content**: Only metadata (timing, size)
- **File contents**: Only file path hashes
- **Sensitive data**: No passwords, tokens, or personal information

### Privacy Guarantees

- **Local Processing**: All data is processed locally
- **Zero-Knowledge**: No raw data is sent to external servers
- **User Control**: Users can disable monitoring at any time
- **No Root Required**: Works without special privileges
- **Privacy Filtering**: SHA-256 hashing, timestamp bucketing

---

## License

ISC License - See [LICENSE](../../LICENSE) for details.

---

## Related Documentation

- [Main README](../../README.md)
- [CHANGELOG](../../CHANGELOG.md)
- [Migration Guide](./migration-guide-nonintrusive.md)
- [Quick Start Guide](./quick-start.md)
- [Troubleshooting](./troubleshooting.md)
