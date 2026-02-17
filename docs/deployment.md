# Cliff-Watch Deployment Guide

> **Version**: 5.2 (Elite)

This guide covers deployment strategies for Cliff-Watch in various environments.

---

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Development Deployment](#development-deployment)
3. [Production Deployment](#production-deployment)
4. [Systemd Service](#systemd-service)
5. [Docker Deployment](#docker-deployment)
6. [CI/CD Integration](#cicd-integration)
7. [Monitoring](#monitoring)

---

## Architecture Overview

```
┌─────────────────┐     ┌──────────────────┐     ┌─────────────────┐
│  VSCode         │────▶│  Cliff-Watch     │────▶│  Git Repository │
│  Extension      │     │  Daemon          │     │  (Validation)   │
└─────────────────┘     └──────────────────┘     └─────────────────┘
```

### Components

| Component | Description | Binary |
|-----------|-------------|--------|
| CLI | User interface | `cliff-watch` |
| Daemon | Background monitoring | `cliff-watch-daemon` |
| Extension | VSCode integration | `cliff-watch-witness` |
| Hooks | Git integration | (Shell scripts) |

---

## Development Deployment

### Local Development

```bash
# Clone repository
git clone https://github.com/iodevs-net/cliff-watch.git
cd cliff-watch

# Build release binary
cargo build --release

# Add to PATH
export PATH=$PATH:$(pwd)/target/release

# Verify installation
cliff-watch system-check
```

### Quick Setup for Development

```bash
# Initialize in test repository
cd /path/to/test/repo
cliff-watch init

# Start daemon
cliff-watch daemon

# View metrics
cliff-watch metrics
```

---

## Production Deployment

### Binary Installation

#### Linux (Manual)

```bash
# Build release binary
cargo build --release

# Install to system
sudo cp target/release/cliff-watch /usr/local/bin/
sudo cp target/release/cliff-watch-daemon /usr/local/bin/

# Create configuration directory
sudo mkdir -p /etc/cliff-watch
sudo cp cliff-watch.toml /etc/cliff-watch/
```

#### Linux (Package)

```bash
# Build .deb package
cargo install cargo-deb
cargo deb

# Install
sudo dpkg -i target/debian/cliff-watch_*.deb
```

### Directory Structure

```
/etc/cliff-watch/
├── cliff-watch.toml      # Configuration
/var/lib/cliff-watch/
├── state.json           # Session state
├── metrics.json         # Historical metrics
~/.config/cliff-watch/
├── config.toml          # User overrides
└── keys/               # Cryptographic keys
```

---

## Systemd Service

### Installation

Create `/etc/systemd/system/cliff-watch-daemon.service`:

```ini
[Unit]
Description=Cliff-Watch Daemon
After=network.target

[Service]
Type=simple
User=cliff-watch
Group=cliff-watch
ExecStart=/usr/local/bin/cliff-watch daemon
Restart=on-failure
RestartSec=10
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
```

### Management

```bash
# Reload systemd
sudo systemctl daemon-reload

# Enable on boot
sudo systemctl enable cliff-watch-daemon

# Start service
sudo systemctl start cliff-watch-daemon

# Check status
sudo systemctl status cliff-watch-daemon

# View logs
sudo journalctl -u cliff-watch-daemon -f
```

---

## Docker Deployment

### Dockerfile

```dockerfile
FROM rust:1.75-slim AS builder

WORKDIR /build
RUN apt-get update && apt-get install -y pkg-config libssl-dev cmake
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /build/target/release/cliff-watch /usr/local/bin/
COPY --from=builder /build/target/release/cliff-watch-daemon /usr/local/bin/
COPY cliff-watch.toml /etc/cliff-watch/

ENTRYPOINT ["cliff-watch-daemon"]
```

### Docker Compose

```yaml
version: '3.8'
services:
  cliff-watch:
    build: .
    container_name: cliff-watch-daemon
    volumes:
      - ./data:/var/lib/cliff-watch
      - ./config/cliff-watch.toml:/etc/cliff-watch/cliff-watch.toml:ro
    restart: unless-stopped
    environment:
      - RUST_LOG=info
```

### Running

```bash
# Build image
docker build -t cliff-watch:latest .

# Run container
docker run -d \
  --name cliff-watch \
  -v $(pwd)/data:/var/lib/cliff-watch \
  -v $(pwd)/config:/etc/cliff-watch:ro \
  cliff-watch:latest
```

---

## CI/CD Integration

### GitHub Actions

```yaml
name: Cliff-Watch Validation

on: [push, pull_request]

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Cliff-Watch
        run: |
          cargo build --release
          echo "$GITHUB_WORKSPACE/target/release" >> $GITHUB_PATH
      
      - name: Initialize
        run: cliff-watch init
      
      - name: Run Tests
        run: cargo test
      
      - name: Commit (if allowed)
        if: github.event_name == 'push'
        run: git commit -m "Tests passed" || echo "Skipping commit"
```

### Git Hooks (Server-Side)

Add to `.git/hooks/pre-receive`:

```bash
#!/bin/bash

while read oldrev newrev refname; do
    # Get list of commits
    commits=$(git rev-list $oldrev..$newrev)
    
    for commit in $commits; do
        # Verify each commit
        cliff-watch verify $commit || exit 1
    done
done
```

---

## Monitoring

### Health Check

```bash
# Check daemon health
cliff-watch system-check

# Check metrics
cliff-watch metrics
```

### Prometheus Metrics

Configure metrics export in `cliff-watch.toml`:

```toml
[monitoring]
enable_prometheus = true
prometheus_port = 9090
```

### Log Aggregation

```bash
# JSON logging
[monitoring]
log_format = "json"
log_file = "/var/log/cliff-watch/daemon.log"
```

---

## Security Considerations

### Key Management

1. **Generate keys on first run:**
```bash
cliff-watch init --generate-keys
```

2. **Backup keys:**
```bash
tar -czf cliff-watch-keys.tar.gz ~/.config/cliff-watch/keys/
```

3. **Rotate keys annually:**
```bash
cliff-watch rotate-keys
```

### Firewall Rules

```bash
# Allow local IPC
sudo ufw allow from 127.0.0.1 to 127.0.0.1 port 9090

# If using network metrics
sudo ufw allow from 10.0.0.0/8 port 9090
```

---

## Backup and Recovery

### Backup

```bash
# Backup configuration and state
tar -czf cliff-watch-backup-$(date +%Y%m%d).tar.gz \
  /etc/cliff-watch \
  /var/lib/cliff-watch \
  ~/.config/cliff-watch
```

### Recovery

```bash
# Restore from backup
tar -xzf cliff-watch-backup-20240101.tar.gz -C /

# Restart daemon
sudo systemctl restart cliff-watch-daemon

# Verify integrity
cliff-watch system-check
```

---

## Performance Optimizations (Phase 4)

### Optional Dependencies

Cliff-Watch now uses optional dependencies to reduce binary size and improve compilation times for the non-intrusive VSCode extension mode:

| Dependency | Feature Flag | Description |
|------------|---------------|-------------|
| `bulletproofs` | `zkp` | Zero-Knowledge Proof generation for daemon mode |
| `tss-esapi` | `tpm` | TPM (Trusted Platform Module) support |
| `syn` | `ast-analysis` | AST analysis for advanced code complexity metrics |

### Build Modes

#### Non-Intrusive Mode (Default)
```bash
# Minimal build for VSCode extension
cargo build --release
```

#### Full Feature Mode
```bash
# Build with all optional features
cargo build --release --features zkp,tpm,ast-analysis
```

#### Daemon Mode
```bash
# Build daemon with ZKP support
cargo build --release -p cliff-watch-daemon --features zkp
```

### Performance Improvements

The TypeScript metrics engine now includes:

1. **Memoization**: Expensive calculations (burstiness, NCD, focus score) are cached for 1000ms TTL
2. **Event Debouncing**: Rapid events are batched and processed together with 100ms delay
3. **Optimized Event Processing**: Reduced redundant calculations and improved state management

### Binary Size Impact

| Build Mode | Approximate Size | Features |
|------------|------------------|----------|
| Default (minimal) | ~3.5 MB | Core metrics only |
| Daemon mode | ~4.2 MB | + ZKP support |
| Full features | ~5.0 MB | + TPM + AST analysis |

---

## Related Documentation

- [Configuration Guide](./configuration.md)
- [API Documentation](./api/README.md)
- [Troubleshooting](./troubleshooting.md)
