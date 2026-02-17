# Future Architecture Modules (v3.0)

This directory contains modules planned for v3.0 of the cliff-watch architecture. These modules are **not integrated** into the main codebase and are kept here for future reference and implementation.

## Overview

These modules represent the next generation of the SentinelReport system, providing a unified, auditable provenance contract with cryptographic integrity guarantees.

## Module Descriptions

### SentinelReport Core

| Module | Purpose | Status |
|---------|---------|--------|
| `sentinel_report.rs` | Immutable, serializable, signable truth container | Planned v3.0 |
| `sentinel_builder.rs` | Deterministic builder pattern for SentinelReport | Planned v3.0 |
| `sentinel_collector.rs` | Metric aggregation layer (temporal, structural, kinematic) | Planned v3.0 |
| `sentinel_finalize.rs` | Finalization pipeline (canonicalize, hash, sign, PoW) | Planned v3.0 |

### Cryptographic Integrity

| Module | Purpose | Status |
|---------|---------|--------|
| `sentinel_hash.rs` | Canonical content hashing of SentinelReport | Planned v3.0 |
| `sentinel_sign.rs` | Cryptographic signing and verification abstraction | Planned v3.0 |
| `sentinel_signer.rs` | Ed25519 signer implementation | Planned v3.0 |
| `sentinel_pow.rs` | Adaptive Proof-of-Work for governance friction | Planned v3.0 |

### Analysis & Fusion

| Module | Purpose | Status |
|---------|---------|--------|
| `entropy_fusion.rs` | Cross-domain anomaly aggregation | Planned v3.0 |

## Integration Notes

### Current Implementation Status

The main codebase (`crates/cliff-watch-core/`) already implements equivalent functionality for some concepts:

| Future Module | Current Equivalent | Notes |
|---------------|-------------------|-------|
| `kinematic.rs` | `mouse_sentinel.rs` | More comprehensive implementation exists |
| `temporal.rs` | `stats.rs` | `calculate_burstiness()` provides temporal analysis |
| `structural.rs` | `stats.rs` | `calculate_ncd()` provides structural analysis |
| `stats.rs` | `stats.rs` | Full implementation with additional features |

### Migration Path

When implementing v3.0:

1. **Integrate SentinelReport system** as the primary report structure
2. **Replace ThermodynamicReport** with SentinelReport
3. **Add entropy fusion layer** for cross-domain anomaly aggregation
4. **Implement PoW** for governance friction (optional feature)
5. **Update crypto module** to support the new signature scheme abstraction

## Design Principles

These modules follow the v3.0 architecture principles:

- **Separation of Concerns**: Each module has clear, single responsibilities
- **No Policy**: Modules compute metrics, they don't make decisions
- **No IO**: Pure functions, no async, no logging
- **Deterministic**: Same inputs always produce same outputs
- **Auditable**: All operations are transparent and verifiable

## Dependencies

These modules require the following crates (not currently in main dependencies):

- `thiserror` - Error handling
- `serde` - Serialization
- `sha2` - SHA-256 hashing
- `hex` - Hex encoding/decoding
- `ed25519_dalek` - Ed25519 signatures
- `statrs` - Statistical functions

## Testing

Each module includes comprehensive unit tests. Run with:

```bash
cargo test --package cliff-watch-core --lib
```

(Once integrated into the main codebase)
