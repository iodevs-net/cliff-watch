# Cliff-Watch Quick Start Guide

> **Version**: 6.0 (Non-Intrusive)  
> **Estimated Setup Time**: 2 minutes

This guide will help you get started with Cliff-Watch in minutes. Follow these steps to protect your Git repository with Proof of Human Work.

---

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Installation](#installation)
3. [Configuration](#configuration)
4. [Basic Usage](#basic-usage)
5. [Verifying Installation](#verifying-installation)
6. [Next Steps](#next-steps)

---

## Prerequisites

Before installing Cliff-Watch, ensure you have:

| Requirement | Minimum Version | Notes |
|-------------|----------------|-------|
| VSCode | 1.108.1+ | IDE for extension |
| Git | Any recent version | For commit hooks |
| Linux/macOS | - | Windows WSL also works |

---

## Installation

### Step 1: Download the Extension

1. Go to [Cliff-Watch Releases](https://github.com/iodevs-net/cliff-watch/releases)
2. Download the latest `cliff-watch-witness-v*.vsix` file

### Step 2: Install in VSCode

1. Open VSCode
2. Press `Ctrl+Shift+X` (or `Cmd+Shift+X` on macOS) to open Extensions
3. Click the `...` menu in the top-right corner
4. Select **"Install from VSIX..."**
5. Navigate to the downloaded `.vsix` file
6. Click **Install**

### Step 3: Enable the Extension

The extension should activate automatically. To verify:

1. Open VSCode Settings (`Ctrl+,`)
2. Search for "Cliff-Watch"
3. Ensure **Cliff-Watch: Enabled** is checked

---

## Configuration

### Quick Configuration (Recommended)

For most users, the default settings work well. Simply add this to your project's `cliff-watch.toml` file:

```toml
[governance]
difficulty = "Normal"
min_entropy = 2.5
audit_mode = true
```

### Advanced Configuration

To customize behavior, open VSCode Settings and configure:

| Setting | Default | Description |
|---------|---------|-------------|
| `cliff-watch.enabled` | `true` | Enable/disable monitoring |
| `cliff-watch.difficulty` | `"Normal"` | Easy, Normal, Hardcore |
| `cliff-watch.auditMode` | `true` | Warn instead of blocking |
| `cliff-watch.minEntropy` | `2.5` | Minimum entropy threshold |
| `cliff-watch.debounceMs` | `500` | Debounce window (ms) |
| `cliff-watch.heartbeatSec` | `15` | Heartbeat interval (sec) |

#### Example JSON Settings

```json
{
  "cliff-watch.enabled": true,
  "cliff-watch.difficulty": "Normal",
  "cliff-watch.auditMode": true,
  "cliff-watch.minEntropy": 2.5
}
```

---

## Basic Usage

### Step 1: Start Coding Normally

1. Open your project in VSCode
2. Start coding as you normally would
3. The extension automatically tracks your activity:
   - Focus events (when you're in VSCode)
   - Edit patterns (typing vs. pasting)
   - Navigation (file changes)

### Step 2: Make a Commit

When you're ready to commit:

```bash
git add .
git commit -m "Your commit message"
```

The commit hook will validate your work:

- **✅ Success**: Commit proceeds normally
- **⚠️ Warning** (audit mode): Commit proceeds with warning
- **❌ Blocked**: Commit rejected - insufficient human work

### Step 3: View Your Metrics

Check your current Humanity Score:

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

## Verifying Installation

### Check Extension Status

1. Open VSCode Developer Tools (`Help > Toggle Developer Tools`)
2. Check the Console for: `"Cliff-Watch Witness activated"`

### Test with a Sample Commit

1. Make a small change to a file
2. Commit the change
3. Verify it succeeds or shows expected warning

### Check Logs

If issues occur:

```bash
# VSCode: Help > Toggle Developer Tools > Console
# Look for: cliff-watch, Cliff-Watch, or witness
```

---

## Next Steps

### For Individual Developers

- Explore [Configuration Options](#configuration)
- Try adjusting difficulty levels
- Enable/disable audit mode based on preference

### For Teams

- Set up shared `cliff-watch.toml` configuration
- Use audit mode initially, then enforce later
- Review [Troubleshooting Guide](./troubleshooting.md) for common issues

### For Contributors

- Review [Architecture Documentation](./architecture_audit_dry_solid_lean_kiss.md)
- Check [API Documentation](./api/README.md)
- See [Contributing Guide](../CONTRIBUTING.md)

---

## Quick Reference

### Key Commands

```bash
# View metrics
cliff-watch metrics

# Check system
cliff-watch system-check

# Initialize repository (optional)
cliff-watch init

# Start daemon (optional)
cliff-watch daemon
```

### Difficulty Levels

| Level | Description | Use Case |
|-------|-------------|----------|
| Easy | Low threshold | Learning/prototyping |
| Normal | Balanced | General development |
| Hardcore | High threshold | Strict quality control |

### Privacy

Cliff-Watch v6.0 is privacy-first:

- ✅ No root required
- ✅ No hardware capture
- ✅ File paths are hashed (SHA-256)
- ✅ Timestamps are bucketed
- ✅ All processing is local

---

## Getting Help

- **Documentation**: [Full Docs](./cliff-watch-witness.md)
- **Troubleshooting**: [Troubleshooting Guide](./troubleshooting.md)
- **Migration**: [Migration Guide](./migration-guide-nonintrusive.md)
- **Issues**: [GitHub Issues](https://github.com/iodevs-net/cliff-watch/issues)

---

*Quick Start Complete! Start coding with confidence.*
