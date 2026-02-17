# Cliff-Watch Documentation Index

> **Version**: 5.2 (Elite)

Welcome to the Cliff-Watch documentation. This index provides a comprehensive overview of all available documentation.

---

## Table of Contents

1. [Getting Started](#getting-started)
2. [Core Documentation](#core-documentation)
3. [User Guides](#user-guides)
4. [Developer Guides](#developer-guides)
5. [Architecture](#architecture)
6. [Reference](#reference)
7. [Research](#research)

---

## Getting Started

| Document | Description | Language |
|----------|-------------|----------|
| [README](../README.md) | Project overview, quick start, installation | English |
| [CONTRIBUTING.md](../CONTRIBUTING.md) | Contribution guidelines | English |

### Quick Start

```bash
# Clone and build
git clone https://github.com/iodevs-net/cliff-watch.git
cd cliff-watch
cargo build --release

# Initialize in repository
cd /your/repo
./target/release/cliff-watch init
./target/release/cliff-watch daemon
```

---

## Core Documentation

| Document | Description |
|----------|-------------|
| [estructura_proyecto.md](estructura_proyecto.md) | Project structure and organization (Spanish) |
| [dry_lean_solid.md](principios/dry_lean_solid.md) | Design principles |
| [validation_report.md](validation_report.md) | Scientific validation report |

---

## User Guides

### Installation & Setup

| Document | Description |
|----------|-------------|
| [cliff-watch-witness.md](cliff-watch-witness.md) | VSCode extension installation and usage |
| [deployment.md](deployment.md) | Production deployment guide |

### Configuration

| Document | Description |
|----------|-------------|
| [Configuration Reference](configuration.md) | Complete configuration options |
| [troubleshooting.md](troubleshooting.md) | Common issues and solutions |

---

## Developer Guides

### API Documentation

| Document | Description |
|----------|-------------|
| [API Reference](api/README.md) | Rust API documentation |
| [Module Documentation](api/modules.md) | Detailed module descriptions |

### Contributing

| Document | Description |
|----------|-------------|
| [CONTRIBUTING.md](../CONTRIBUTING.md) | Contribution guidelines |
| [Roadmap](roadmaps/start/roadmap.md) | Implementation roadmap |

---

## Architecture

### Design Principles

| Document | Description |
|----------|-------------|
| [Architecture Audit](architecture_audit_dry_solid_lean_kiss.md) | Architecture review |
| [Phase 2 Architecture](phase2/arquitectura_sistema.md) | System architecture (Spanish) |
| [Module Definition](phase2/definicion_modulos.md) | Module definitions (Spanish) |

### Technical Specs

| Document | Description |
|----------|-------------|
| [Mouse Sentinel Design](phase2/diseno_mouse_sentinel.md) | Sentinel design document (Spanish) |
| [Phase 1 Requirements](phase1/requisitos_resumidos.md) | Requirements summary (Spanish) |
| [Dependencies](phase1/dependencias_validadas.md) | Validated dependencies (Spanish) |

---

## Reference

### CLI Reference

```bash
# System check
cliff-watch system-check

# Initialize repository
cliff-watch init

# Start daemon
cliff-watch daemon

# View metrics
cliff-watch metrics

# Verify commit
cliff-watch verify <commit-hash>
```

### Configuration Reference

```toml
[governance]
difficulty = "Normal"    # Easy, Normal, Hardcore
min_entropy = 2.5
audit_mode = true

[monitoring]
debounce_window_ms = 500
ignore_extensions = ["log", "lock", "tmp"]
```

---

## Research

| Document | Description |
|----------|-------------|
| [Mouse Sentinel Research](research/v1/Mouse-Sentinel.md) | Initial research (Spanish) |
| [Key Requirements](research/v1/requisitos_clave.md) | Key requirements (Spanish) |
| [Rust Principles](research/v1/rust-lean-dry-solid.md) | Rust implementation (Spanish) |
| [Sentinel Protocol](research/v1/sentinel-protocol.md) | Protocol specification (Spanish) |
| [Research v3 - Gemini](research/v3/gemini.md) | AI research |
| [Research v3 - Perplexity](research/v3/perplexity.md) | AI research |

---

## Roadmap

| Document | Description |
|----------|-------------|
| [Main Roadmap](roadmaps/start/roadmap.md) | Implementation roadmap (Spanish) |
| [Architecture Correction](roadmaps/architecture_correction_roadmap.md) | Architecture fixes |
| [Pareto Roadmap](roadmaps/cliff-craft-pareto-roadmap.md) | Pareto optimization |

---

## Additional Resources

### Edge Cases

| Document | Description |
|----------|-------------|
| [Edge Cases](EDGE_CASES.md) | Edge case handling |

### Testing

| Document | Description |
|----------|-------------|
| [Test Results](test_results_summary.md) | Test results summary |

---

## Language Guide

The project documentation uses mixed languages:

- **English**: Primary documentation (README, API, Contributing)
- **Spanish**: Phase documentation, research, older guides

### Translation Status

| Document | Status | Language |
|----------|--------|----------|
| README.md | ✅ Complete | English |
| API Documentation | ✅ Complete | English |
| Contributing.md | ✅ Complete | English |
| Troubleshooting | ✅ Complete | English |
| Deployment | ✅ Complete | English |
| VSCode Extension Guide | ✅ Complete | English |
| Migration Guide | ✅ Complete | English |
| estructura_proyecto.md | ⚠️ Needs Review | Spanish |
| validation_report.md | ⚠️ Needs Review | Spanish |
| Phase Documentation | ⚠️ Needs Review | Spanish |

---

## Navigation Tips

### Finding Information

1. **For users**: Start with [README](../README.md) → [cliff-watch-witness.md](cliff-watch-witness.md)
2. **For developers**: Start with [API Reference](api/README.md) → [CONTRIBUTING.md](../CONTRIBUTING.md)
3. **For deployment**: See [deployment.md](deployment.md)
4. **For troubleshooting**: See [troubleshooting.md](troubleshooting.md)

### Document Versioning

- Documents are versioned to match the software release
- Check the document header for version information
- Latest documentation: v5.2 (Elite)

---

## Contributing to Documentation

1. Fork the repository
2. Create a branch: `git checkout -b docs/improve-something`
3. Make improvements
4. Submit a pull request

See [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines.

---

*Last Updated: 2026-02-16*
