# Cliff-Watch Troubleshooting Guide

> **Version**: 6.0 (Non-Intrusive)

This guide covers common issues and solutions for Cliff-Watch users and developers.

---

## Table of Contents

1. [Installation Issues](#installation-issues)
2. [Build Errors](#build-errors)
3. [Runtime Issues](#runtime-issues)
4. [VSCode Extension Issues](#vscode-extension-issues)
5. [Daemon Issues](#daemon-issues)
6. [Commit Validation Issues](#commit-validation-issues)
7. [Performance Issues](#performance-issues)
8. [Debugging Tools](#debugging-tools)

---

## Installation Issues

### Rust Not Found

**Symptom:** `cargo: command not found`

**Solution:**
```bash
# Install Rust via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Verify installation
rustc --version
cargo --version
```

### Missing System Dependencies

**Symptom:** `error: failed to run custom build command for 'git2'`

**Solution (Ubuntu/Debian):**
```bash
sudo apt-get install libssl-dev pkg-config libgit2-dev cmake
```

**Solution (macOS):**
```bash
brew install openssl pkg-config git
```

### Binary Size Too Large

**Symptom:** Binary exceeds expected size

**Solution:**
```bash
# Ensure release profile is optimized for size
cargo build --release -Z minimal-versions

# Check binary size
ls -lh target/release/cliff-watch
```

---

## Build Errors

### Dependency Resolution Failures

**Symptom:** `error: failed to select a version for...`

**Solution:**
```bash
# Update dependencies
cargo update

# Clean and rebuild
cargo clean
cargo build --release
```

### Feature Flags Not Found

**Symptom:** `error[E0432]: unresolved import `crate::feature``

**Solution:**
Check your `Cargo.toml` has the correct features:
```toml
[dependencies]
cliff-watch-core = { path = "crates/cliff-watch-core", features = ["ast-analysis"] }
```

### Static Linking Errors

**Symptom:** `error: library 'git2' required but not found`

**Solution:**
Ensure vendored-libgit2 feature is enabled:
```toml
git2 = { version = "0.20", default-features = false, features = ["vendored-libgit2"] }
```

---

## Runtime Issues

### Daemon Won't Start

**Symptom:** `Error: Failed to start daemon`

**Diagnostics:**
```bash
# Check if another instance is running
ps aux | grep cliff-watch

# Try starting with verbose output
RUST_LOG=debug cliff-watch daemon
```

**Common Causes:**
1. Another daemon already running
2. Missing write permissions to `/tmp`
3. Port already in use

**Solutions:**
```bash
# Kill existing daemon
pkill cliff-watch-daemon

# Clear stale sockets
rm -f /tmp/cliff-watch.sock

# Try different port
cliff-watch daemon --port 9876
```

### IPC Connection Refused

**Symptom:** `Error: Connection refused: /tmp/cliff-watch.sock`

**Solution:**
```bash
# Verify daemon is running
cliff-watch status

# Restart daemon
cliff-watch off
cliff-watch on
```

---

## VSCode Extension Issues

### Extension Not Loading

**Symptom:** Extension doesn't activate on VSCode startup

**Diagnostics:**
1. Open VSCode Developer Tools (`Help > Toggle Developer Tools`)
2. Check Console for errors

**Common Causes:**
- VSCode version too old (requires ^1.108.1)
- Extension disabled in settings

**Solutions:**
```json
{
  "extensions.autoUpdate": true,
  "cliff-watch.enabled": true
}
```

### Daemon Connection Failed

**Symptom:** "Failed to connect to Cliff-Watch daemon"

**Solution:**
1. The extension works standalone without daemon
2. If using daemon mode, ensure it's running: `cliff-watch daemon`
3. Check firewall settings
4. Verify socket permissions:
```bash
ls -la /tmp/cliff-watch.sock
```

### Focus Events Not Detected

**Symptom:** Focus tracking not working in VSCode

**Solution:**
1. Reload VSCode window
2. Check extension host logs
3. Verify VSCode API accessibility

### Extension v3.0: Privacy Filtering Issues

**Symptom:** Metrics showing but commits still blocked

**Cause:** Local metrics computation may have different thresholds

**Solution:**
1. Check your Humanity Score: `cliff-watch metrics`
2. Verify difficulty setting in VSCode settings
3. Ensure `audit_mode` is enabled for testing:
```json
{
  "cliff-watch.auditMode": true
}
```

### Extension v3.0: Configuration Not Saving

**Symptom:** Settings changes don't persist

**Solution:**
1. Check VSCode settings.json syntax
2. Restart VSCode after changing settings
3. Verify write permissions on `.vscode/settings.json`

---

## Commit Validation Issues

### Commit Blocked: Thermodynamic Failure

**Symptom:** `Error: Thermodynamic failure - Insufficient battery`

**Cause:** The system detected low human probability score

**Solutions:**

1. **Spend more time in focused sessions:**
   - Take breaks between major changes
   - Navigate between files naturally
   - Make smaller, more deliberate edits

2. **Check your metrics:**
```bash
cliff-watch metrics
```

3. **Lower the difficulty (temporary):**
```toml
[governance]
difficulty = "Easy"
```

4. **Enable audit mode (for teams):**
```toml
[governance]
audit_mode = true  # Warn only, don't block
```

### False Positives

**Symptom:** Legitimate commits being blocked

**Solution:**
```bash
# Check current score
cliff-watch metrics

# View detailed logs
RUST_LOG=debug git commit -m "Fix bug"
```

**Common triggers:**
- Large paste operations
- Auto-format on save
- Automated refactoring tools

### NCD Threshold Too High

**Symptom:** Original code being flagged as copy

**Solution:**
```toml
[governance]
min_entropy = 1.5  # Lower for more tolerance
```

---

## Performance Issues

### High CPU Usage

**Symptom:** Daemon consuming excessive CPU

**Diagnostics:**
```bash
top -p $(pgrep cliff-watch-daemon)
```

**Solutions:**
1. Increase debounce window:
```toml
[monitoring]
debounce_window_ms = 2000
```

2. Ignore more file types:
```toml
[monitoring]
ignore_extensions = ["log", "lock", "tmp", "json", "map"]
```

### Memory Leaks

**Symptom:** Increasing memory usage over time

**Solution:**
```bash
# Restart daemon periodically
crontab -e
# Add: 0 */4 * * * cliff-watch off && cliff-watch on
```

### Slow Commits

**Symptom:** Commit takes longer than expected

**Cause:** Large repository or complex validation

**Solution:**
```bash
# Limit files checked
echo "node_modules/" >> .gitignore
echo "target/" >> .gitignore
```

---

## Debugging Tools

### System Check

Verify all dependencies:
```bash
cliff-watch system-check
```

Expected output:
```
Sentinel System Integrity Check:
✔ Git2 (libgit2): Static Linked
✔ Crypto (Ed25519): Key Generation Subsystem Active
✔ Hashing (SHA256): Engine Online
✔ Entropy Engine (Zstd): Compression Active
✔ Statistics (Statrs): Compute Modules Loaded
```

### Verbose Logging

Enable debug logging:
```bash
RUST_LOG=debug cliff-watch daemon
RUST_LOG=debug git commit -m "Test"
```

### Profile Memory

```bash
# Install memory profiler
cargo install heaptrack

# Run with profiling
heaptrack ./target/release/cliff-watch daemon
```

### Test Suite

Run comprehensive tests:
```bash
# All tests
cargo test --workspace

# Specific crate
cargo test -p cliff-watch-core

# With coverage
cargo install cargo-llvm-cov
cargo llvm-cov --workspace
```

---

## Upgrade Issues (v5.x to v6.0)

### Old Daemon Still Running

**Symptom:** New extension conflicts with old daemon

**Solution:**
```bash
# Stop old daemon
pkill cliff-watch-daemon

# Remove old service (if installed)
sudo systemctl stop cliff-watch-daemon
sudo systemctl disable cliff-watch-daemon
```

### evdev Feature Deprecated

**Symptom:** Build errors related to evdev

**Solution:**
```toml
# Remove legacy-evdev feature from Cargo.toml
# The extension now uses VSCode APIs instead
```

### Migration from Daemon-Based to Extension

**Symptom:** Not receiving metrics after upgrade

**Solution:**
1. Uninstall old daemon-based setup
2. Install new VSCode extension (VSIX)
3. Restart VSCode
4. Configure settings if needed

---

## Common Error Messages (v6.0)

| Error | Cause | Solution |
|-------|-------|----------|
| `Git2 linkage failed` | libgit2 not linked | Rebuild with vendored feature |
| `Entropy Engine error` | Zstd compression failed | Check file permissions |
| `Crypto subsystem inactive` | Ed25519 failure | Run system check |
| `IPC timeout` | Daemon not responding (optional in v6.0) | Use extension standalone mode |
| `ZKP verification failed` | Proof invalid | Regenerate proof |
| `Extension not activated` | VSCode API issue | Reload VSCode window |
| `Privacy filter error` | Hash computation failed | Check Node.js version |
| `Metrics computation failed` | Local processing error | Check VSCode console logs |

---

## Getting Help

### Check Logs

```bash
# Daemon logs
journalctl -u cliff-watch-daemon -f

# Extension logs (VSCode)
# Help > Toggle Developer Tools > Console
```

### Report Issues

Include in your report:
- Output of `cliff-watch system-check`
- Rust version: `rustc --version`
- OS version: `uname -a`
- Relevant logs with `RUST_LOG=debug`

### Community Support

- GitHub Issues: https://github.com/iodevs-net/cliff-watch/issues
- Discussions: https://github.com/iodevs-net/cliff-watch/discussions

---

## Related Documentation

- [Configuration Guide](./configuration.md)
- [VSCode Extension Guide](./cliff-watch-witness.md)
- [Quick Start Guide](./quick-start.md)
- [Migration Guide](./migration-guide-nonintrusive.md)
- [API Documentation](./api/README.md)
- [Scientific Validation](./validation_report.md)
