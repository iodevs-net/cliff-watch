# Changelog

All notable changes to Cliff-Watch are documented in this file.

The format is based on [Conventional Commits](https://www.conventionalcommits.org/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [6.0.0] - 2026-02-17

### 🎉 Major Release: Non-Intrusive Architecture

This release completes the transformation to a non-intrusive VSCode extension architecture, prioritizing user privacy while maintaining robust Proof of Human Work validation.

#### Added

- **VSCode Extension (Cliff-Witness)**
  - New VSCode extension: `clients/cliff-watch-witness/`
  - Focus tracking via VSCode APIs
  - Edit tracking with burst detection
  - Navigation tracking (scroll, hover, goto definition)
  - Heartbeat mechanism (15-second intervals)
  - Privacy-filtered telemetry transmission

- **Privacy Filtering Module**
  - SHA-256 file path hashing (`crates/cliff-watch-core/src/privacy.rs`)
  - Timestamp bucketing (configurable, default 5 seconds)
  - Content sanitization (metadata only, no raw content)
  - Privacy-preserving metrics aggregation

- **Configuration UI**
  - VSCode settings integration
  - Visual configuration panel
  - Real-time settings synchronization

- **Metrics Engine**
  - Local metrics computation in TypeScript
  - Burstiness calculation
  - Entropy estimation
  - Focus time tracking

#### Changed

- **Architecture**
  - Daemon now optional (extension can work standalone for metrics)
  - IPC communication via Unix sockets
  - Local-first processing model

- **Installation**
  - VSIX package distribution
  - No root privileges required
  - No hardware capture (removed evdev)

#### Removed

- **evdev Hardware Capture**
  - Removed `backend/linux.rs` evdev code
  - Removed root privilege requirement
  - Removed intrusive hardware monitoring

#### Migration Notes

For users upgrading from v5.x:

1. **Extension Installation**: Download the new VSIX and install via VSCode
2. **No More Root**: The extension no longer requires root privileges
3. **Daemon Optional**: The daemon is now optional for basic metrics
4. **Privacy Improved**: File paths are now hashed, timestamps bucketed

---

## [5.2.0] - 2026-01-15

### Release: Elite Version

#### Added

- Enhanced NCD calculation using Zstandard compression
- Temporal memory EMA engine (80/20 split)
- Thermodynamic battery visualization
- Sovereign privacy mode (zero-knowledge proofs)

#### Changed

- Improved entropy density calculation
- Optimized battery charging algorithm

#### Fixed

- Memory leak in entropy calculation
- Race condition in focus tracking

---

## [5.1.0] - 2025-12-01

### Release: Enhanced Stability

#### Added

- Configuration file support (`cliff-watch.toml`)
- System check command
- Detailed metrics output

#### Changed

- Refactored daemon IPC
- Improved error handling

#### Fixed

- Git2 linkage issues
- Static build compatibility

---

## [5.0.0] - 2025-11-01

### Release: Major Refactor

#### Added

- Core library (`cliff-watch-core`)
- CLI commands modularization
- Focus protocol implementation

#### Changed

- Complete codebase restructure
- Migration to workspace-based Cargo layout

---

## [4.0.0] - 2025-09-01

### Release: Focus Protocol

#### Added

- Focus session tracking
- Mouse sentinel for input detection
- Kinematic analyzer

#### Changed

- Improved human score calculation
- Better false positive handling

---

## [3.0.0] - 2025-07-01

### Release: Privacy First

#### Added

- Privacy impact assessment
- Local-first architecture
- Zero-knowledge design principles

---

## [2.0.0] - 2025-05-01

### Release: Metrics Engine

#### Added

- Comprehensive metrics collection
- Statistical analysis module
- Complexity scoring

---

## [1.0.0] - 2025-03-01

### Release: Initial Version

#### Added

- Initial proof-of-concept implementation
- Daemon-based monitoring
- Git hook integration
- Basic entropy calculation

---

## Migration Guides

### Upgrading to v6.0.0

If you're upgrading from an earlier version:

1. **Remove Old Installation**
   ```bash
   # Stop and remove old daemon service (if running)
   sudo systemctl stop cliff-watch-daemon
   sudo systemctl disable cliff-watch-daemon
   ```

2. **Install New Extension**
   - Download `cliff-watch-witness-v*.vsix`
   - Install via VSCode: `Extensions > ... > Install from VSIX`

3. **Update Configuration**
   ```toml
   [governance]
   difficulty = "Normal"
   min_entropy = 2.5
   audit_mode = true
   ```

4. **Optional: Install CLI**
   ```bash
   cargo build --release
   sudo cp target/release/cliff-watch /usr/local/bin/
   ```

### From v5.x to v6.0

Key differences:

| Aspect | v5.x | v6.0 |
|--------|------|------|
| Installation | Daemon service | VSCode Extension |
| Root Required | Yes (evdev) | No |
| Privacy | Medium | High |
| Setup Time | 30 min | 2 min |

---

## Deprecation Notices

### evdev Hardware Capture (Deprecated in v4.0, Removed in v6.0)

The evdev-based hardware capture has been removed. Users relying on this feature should:

1. Install the VSCode extension for equivalent functionality
2. Remove `legacy-evdev` feature from dependencies

### Daemon-Only Mode (Deprecated in v6.0)

The daemon-only mode is deprecated. Please use the VSCode extension for IDE integration.

---

## Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md) for contribution guidelines.

---

## License

This project is licensed under the MIT License - see the [LICENSE](./LICENSE) file for details.

---

*This changelog was generated for Cliff-Watch v6.0.0*
