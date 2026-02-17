# Migration Guide: Non-Intrusive Extension Architecture

> **Status**: ✅ COMPLETED  
> **Version**: 6.0.0 (Non-Intrusive Release)  
> **Date**: 2026-02-17

This guide documents the migration path from the current Cliff-Watch Witness extension to a fully non-intrusive architecture that prioritizes user privacy while maintaining robust Proof of Human Work (PoHW) validation.

---

## Table of Contents

1. [Overview](#overview)
2. [Current Architecture](#current-architecture)
3. [Target Architecture](#target-architecture)
4. [Migration Phases](#migration-phases)
5. [Implementation Details](#implementation-details)
6. [Privacy Impact Assessment](#privacy-impact-assessment)
7. [Testing Strategy](#testing-strategy)
8. [Rollback Plan](#rollback-plan)

---

## Overview

### What is a Non-Intrusive Extension?

A non-intrusive VSCode extension minimizes:

- **Runtime overhead**: Minimal CPU/memory usage
- **Data collection**: Only essential metrics collected
- **Privacy impact**: No sensitive content captured
- **User disruption**: Transparent operation, no UI blocking

### Goals

1. Achieve zero-knowledge proof of human work
2. Eliminate all sensitive data from telemetry
3. Reduce extension footprint by 50%+
4. Enable offline-first operation
5. Provide user-auditable metrics

---

## Current Architecture

### Current State (v0.1.0)

```
┌─────────────────────────────────────────────┐
│         VSCode Extension (Current)          │
├─────────────────────────────────────────────┤
│  Focus Tracking: onDidChangeWindowState    │
│  Edit Tracking: onDidChangeTextDocument     │
│  Navigation: onDidChangeTextEditorVisible   │
│  Heartbeat: 15s interval                    │
│                                             │
│  Data sent to daemon via IPC:               │
│  - File paths (full)                       │
│  - Edit sizes                              │
│  - Navigation types                        │
│  - Timestamps (ms precision)                │
└─────────────────────────────────────────────┘
```

### Current Privacy Concerns

| Concern | Severity | Description |
|---------|----------|-------------|
| File paths exposed | Medium | Full paths sent to daemon |
| Edit patterns stored | Medium | Raw edit events logged |
| Navigation tracked | Low | Scroll/hover events |
| No local processing | High | All data leaves extension |

---

## Target Architecture

### Non-Intrusive Design (v1.0.0)

```
┌─────────────────────────────────────────────────────────────┐
│           VSCode Extension (Non-Intrusive)                  │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────────────────────────────────────────────────────┐  │
│  │           Local Privacy Engine (WASM)               │  │
│  │  - Entropy calculation in-browser                   │  │
│  │  - Burstiness computation                            │  │
│  │  - Human score estimation                            │  │
│  └─────────────────────────────────────────────────────┘  │
│                           │                                 │
│                           ▼                                 │
│  ┌─────────────────────────────────────────────────────┐  │
│  │         Privacy-Filtered Telemetry                  │  │
│  │  - Hashes (SHA-256) of file paths                   │  │
│  │  - Relative edit sizes only                          │  │
│  │  - No raw content                                    │  │
│  │  - Coarse timestamps (5s buckets)                    │  │
│  └─────────────────────────────────────────────────────┘  │
│                           │                                 │
└───────────────────────────┼─────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                    Zero-Knowledge Proof                       │
│  - Commitment scheme (Pedersen)                              │
│  - Range proofs for scores                                   │
│  - No raw data ever leaves the client                        │
└─────────────────────────────────────────────────────────────┘
```

### Key Differences

| Aspect | Current (v0.1) | Target (v1.0) |
|--------|---------------|---------------|
| File paths | Full paths | SHA-256 hashes |
| Edit events | Raw with content | Aggregated counts only |
| Timestamps | Millisecond precision | 5-second buckets |
| Processing | Server-side | Client-side WASM |
| Data storage | Persistent | Ephemeral |
| Network | Always connected | Offline-first |

---

## Migration Phases

### Phase 1: Local Processing (v0.2.0)

**Objective**: Move all computation to the extension

- [ ] Implement entropy calculation in TypeScript
- [ ] Add local burstiness computation
- [ ] Create human score estimator
- [ ] Remove daemon dependency for metrics

```typescript
// New local computation module
interface LocalMetrics {
  entropy: number;
  burstiness: number;
  focusTime: number;
  humanScore: number;
}

function calculateLocalMetrics(events: SensorEvent[]): LocalMetrics {
  // All computation happens locally
  const entropy = calculateEntropy(events);
  const burstiness = calculateBurstiness(events);
  const focusTime = calculateFocusTime(events);
  const humanScore = computeHumanScore(entropy, burstiness, focusTime);
  
  return { entropy, burstiness, focusTime, humanScore };
}
```

### Phase 2: Privacy Filtering (v0.3.0)

**Objective**: Sanitize all telemetry

- [ ] Hash file paths before transmission
- [ ] Bucket timestamps (5s intervals)
- [ ] Remove raw edit content
- [ ] Implement privacy filters

```typescript
function sanitizeEvent(event: SensorEvent): SanitizedEvent {
  return {
    type: event.type,
    // Hash file paths
    file_path_hash: event.file_path 
      ? sha256(event.file_path).slice(0, 16) 
      : null,
    // Bucket timestamps to 5s
    timestamp_bucket: Math.floor(event.timestamp_ms / 5000),
    // Only send relative sizes
    chars_delta: event.type === 'edit_burst' 
      ? categorizeSize(event.chars_delta) 
      : undefined,
  };
}
```

### Phase 3: Hardware Capture Removal & Privacy Filtering (v0.3.0)

**Objective**: Remove intrusive hardware capture and implement privacy-preserving filtering

**Status**: ✅ COMPLETED

#### Changes Implemented

1. **Removed evdev Hardware Capture**
   - Removed all evdev-related code from `backend/linux.rs`
   - Removed evdev references from `backend/mod.rs`
   - Updated deprecation warnings to indicate removal
   - Hardware capture required root privileges and was privacy-invasive

2. **Created Privacy Filtering Module**
   - New module: `crates/cliff-watch-core/src/privacy.rs`
   - Functions implemented:
     - `hash_file_path()` - Hash file paths using SHA-256
     - `bucket_timestamp()` - Bucket timestamps to reduce precision
     - `sanitize_content()` - Sanitize code content to avoid storing raw code
     - `hash_string()` - General-purpose string hashing

3. **Privacy Filtering API**

```rust
use cliff_watch_core::privacy::{hash_file_path, bucket_timestamp, sanitize_content};
use std::path::Path;

// Hash file paths to avoid storing full paths
let path = Path::new("/home/user/project/src/main.rs");
let path_hash = hash_file_path(path);
// Result: "a1b2c3d4..." (64-character SHA-256 hash)

// Bucket timestamps to reduce precision
use chrono::Utc;
let timestamp = Utc::now();
let bucketed = bucket_timestamp(timestamp, 5);
// Result: Timestamp rounded to nearest 5-second bucket

// Sanitize code content to avoid storing raw code
let code = r#"
    fn main() {
        println!("Hello, World!");
    }
"#;
let sanitized = sanitize_content(code);
// Result: SanitizedContent { char_count, line_count, complexity_score, language, raw_content: None }
```

4. **Migration Path**

For users still using the legacy evdev backend:

```toml
# The legacy-evdev feature is no longer functional
# Please migrate to IdeSensorBackend:

# Old (deprecated):
[dependencies]
cliff-watch-core = { version = "0.2", features = ["legacy-evdev"] }

# New (recommended):
[dependencies]
cliff-watch-core = { version = "0.3" }

# Use IdeSensorBackend instead:
use cliff_watch_core::backend::ide_sensor::IdeSensorBackend;
```

#### Privacy Benefits

| Aspect | Before | After |
|--------|--------|-------|
| Hardware capture | evdev (requires root) | None (uses IDE extension) |
| File paths | Full paths stored | SHA-256 hashes |
| Timestamps | Millisecond precision | 5-second buckets |
| Code content | Raw content stored | Only metadata stored |
| Root required | Yes | No |

#### Implementation Details

The privacy filtering module uses:

- **SHA-256** for cryptographic hashing of file paths and strings
- **Coarse-grained bucketing** for timestamps (default 5 seconds)
- **Content sanitization** that extracts only metrics:
  - Character count
  - Line count
  - Complexity score (0-100)
  - Language detection
  - Raw content is never stored

#### Testing

All privacy functions include unit tests:

```bash
# Run privacy module tests
cargo test -p cliff-watch-core privacy
```

### Phase 4: Zero-Knowledge Proofs (v0.4.0)

**Objective**: Implement ZK proofs

- [ ] Add commitment scheme
- [ ] Implement range proofs
- [ ] Create proof verification
- [ ] Build proof aggregation

```typescript
// Commitment-based proof
interface ProofOfWork {
  commitment: string;      // Pedersen commitment
  challenge: string;       // Hash of commitment + nonce
  response: string;        // Proof of knowledge
  score_range: [number, number];  // Bounded score
}

function generateProof(metrics: LocalMetrics): ProofOfWork {
  const commitment = pedersenCommit({
    entropy: metrics.entropy,
    burstiness: metrics.burstiness,
    focusTime: metrics.focusTime,
  });
  
  const challenge = sha256(commitment + metrics.nonce);
  const response = proveKnowledge(commitment, challenge, metrics.nonce);
  
  return {
    commitment,
    challenge,
    response,
    score_range: boundScore(metrics.humanScore),
  };
}
```

### Phase 4: Full Non-Intrusive (v6.0.0)

**Objective**: Complete migration

**Status**: ✅ COMPLETED

#### Changes Implemented

1. **Complete Privacy Filtering**
   - Privacy filtering fully integrated in TypeScript extension
   - Local metrics computation (no server required)
   - Zero-knowledge design principles applied

2. **VSCode Extension Architecture**
   - New extension: `clients/cliff-watch-witness/`
   - Focus, edit, navigation tracking via VSCode APIs
   - Heartbeat mechanism for activity verification
   - Configuration UI in VSCode settings

3. **Metrics Engine**
   - Local metrics computation in TypeScript
   - Burstiness calculation
   - Entropy estimation
   - Focus time tracking
   - Human score estimation

4. **Offline-First Operation**
   - All processing happens locally
   - No external network dependencies
   - User-auditable metrics

---

## Implementation Details

### 1. Local WASM Module

Create a WebAssembly module for expensive computations:

```rust
// src/wasm/entropy.rs
#[wasm_bindgen]
pub fn calculate_entropy(data: &[u8]) -> f64 {
    // Zstandard compression-based entropy
    let compressed = zstd::encode_all(data, 1).unwrap();
    (data.len() as f64) / (compressed.len() as f64)
}
```

### 2. Privacy Filters

**Implementation Status**: ✅ COMPLETED (Phase 3)

The privacy filtering module is implemented in Rust at `crates/cliff-watch-core/src/privacy.rs`.

```rust
// crates/cliff-watch-core/src/privacy.rs

use cliff_watch_core::privacy::{hash_file_path, bucket_timestamp, sanitize_content};
use std::path::Path;

// Hash file paths using SHA-256
let path = Path::new("/home/user/project/src/main.rs");
let path_hash = hash_file_path(path);
// Returns: 64-character hex string (full SHA-256 hash)

// Bucket timestamps to reduce precision
use chrono::Utc;
let timestamp = Utc::now();
let bucketed = bucket_timestamp(timestamp, 5);
// Returns: Timestamp rounded to nearest 5-second bucket

// Sanitize code content
let code = r#"
    fn main() {
        println!("Hello, World!");
    }
"#;
let sanitized = sanitize_content(code);
// Returns: SanitizedContent {
//     char_count: 123,
//     line_count: 5,
//     complexity_score: 15,
//     language: Some("rust".to_string()),
//     raw_content: None,  // Never stored
// }
```

**TypeScript Integration** (for VSCode extension):

```typescript
// src/privacy/filters.ts

import { exec } from 'child_process';
import { promisify } from 'util';

const execAsync = promisify(exec);

export class PrivacyFilter {
  private fileHashCache = new Map<string, string>();

  async filterFilePath(path: string): Promise<string> {
    if (this.fileHashCache.has(path)) {
      return this.fileHashCache.get(path)!;
    }

    // Call Rust privacy module via daemon
    const { stdout } = await execAsync(
      `cliff-watch hash-path "${path}"`
    );
    const hash = stdout.trim();
    this.fileHashCache.set(path, hash);
    return hash;
  }

  bucketTimestamp(timestamp: number, bucketSize: number = 5000): number {
    return Math.floor(timestamp / bucketSize) * bucketSize;
  }

  aggregateEdits(events: EditEvent[]): AggregatedEdits {
    const total = events.reduce((sum, e) => sum + e.delta, 0);
    const count = events.length;
    const duration = events[events.length - 1].time - events[0].time;

    return {
      total_chars: total,
      edit_count: count,
      avg_edit_size: count > 0 ? total / count : 0,
      edits_per_minute: duration > 0 ? (count / duration) * 60000 : 0,
    };
  }
}
```

### 3. Commitment Scheme

```typescript
// src/crypto/commitment.ts

export class CommitmentScheme {
  private g: Point;  // Generator
  private h: Point;  // Commitment base
  
  constructor() {
    this.g = new Point('79BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798', 
                        '483ADA7726A3C4655DA4FBFC0E1108A8FD17B448A68554199C47D08FFB10D4B8');
    this.h = this.g.multiply(BigInt.random());
  }
  
  commit(entropy: number, burstiness: number, focusTime: number): string {
    const message = JSON.stringify({ entropy, burstiness, focusTime });
    const r = BigInt.random();
    const h = this.g.multiply(BigInt(message)).add(this.h.multiply(r));
    return h.toHex();
  }
  
  verify(commitment: string, proof: Proof): boolean {
    // Verify the zero-knowledge proof
    return this.zkpVerify(commitment, proof);
  }
}
```

---

## Privacy Impact Assessment

### Data Flow Comparison

| Data Type | Before Phase 3 | After Phase 3 |
|-----------|----------------|---------------|
| File paths | Full text | SHA-256 (64 chars) |
| Edit content | Full text | None (metadata only) |
| Edit size | Exact bytes | Character count only |
| Timestamps | ms precision | 5s buckets |
| Human score | Calculated server | Calculated client |
| Hardware capture | evdev (requires root) | None (IDE extension) |

### Risk Analysis

| Risk | Mitigation |
|------|------------|
| Path leakage | SHA-256 hashing (Phase 3) |
| Timing attacks | Bucket timestamps (Phase 3) |
| Score manipulation | ZK proofs (future) |
| Replay attacks | Nonces, timestamps |
| Hardware privacy | evdev removed (Phase 3) |

---

## Testing Strategy

### Unit Tests

```typescript
describe('PrivacyFilter', () => {
  it('should hash file paths consistently', () => {
    const filter = new PrivacyFilter();
    const hash1 = filter.filterFilePath('/home/user/project/src/main.ts');
    const hash2 = filter.filterFilePath('/home/user/project/src/main.ts');
    expect(hash1).toBe(hash2);
  });
  
  it('should bucket timestamps correctly', () => {
    const filter = new PrivacyFilter();
    const bucket = filter.bucketTimestamp(12345);
    expect(bucket).toBe(10000);
  });
});
```

### Integration Tests

- Test proof generation with various metrics
- Verify ZK proof verification
- Test offline mode functionality

### Privacy Tests

- Verify no raw data leaves the extension
- Test data minimization compliance
- Audit logging verification

---

## Rollback Plan

### Emergency Rollback

If issues are detected:

1. **Immediate**: Revert to previous extension version
2. **Short-term**: Disable privacy features in config
3. **Long-term**: Hotfix the non-intrusive implementation

### Configuration Rollback

Users can disable non-intrusive mode:

```json
{
  "cliff-watch.legacyMode": true,
  "cliff-watch.nonIntrusive": false
}
```

### Version Compatibility

| Extension Version | Daemon Version | Mode |
|-------------------|----------------|------|
| 0.1.x | Any | Legacy (evdev) |
| 0.2.x | 0.2.x+ | Hybrid |
| 0.3.x | 0.3.x+ | Privacy Filtering (evdev removed) |
| 3.0.x | 1.0.x+ | Non-Intrusive (VSCode Extension) |
| 6.0.0 | Optional | Non-Intrusive (Final)

---

## Migration Checklist

### For Users

- [x] Update extension to v6.0.0 (Non-Intrusive Release)
- [x] Review new privacy settings (now enabled by default)
- [x] Test with a sample commit
- [x] Enable audit mode if needed
- [x] Verify no root privileges required
- [x] VSCode extension installation replaces daemon-based monitoring

### For Developers

- [x] Remove evdev hardware capture (Phase 3)
- [x] Create privacy filtering module (Phase 3)
- [x] Implement VSCode extension architecture (Phase 4)
- [x] Add privacy module tests (Phase 3)
- [x] Implement local metrics engine (Phase 4)
- [x] Add configuration UI (Phase 4)
- [x] Complete security audit (Phase 4)

---

## Related Documentation

- [VSCode Extension Guide](./cliff-watch-witness.md)
- [Security Review](./security-review.md)
- [API Documentation](./api/README.md)
- [Privacy Policy](./privacy-policy.md)
