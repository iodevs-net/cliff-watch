# Cliff-Watch API Documentation

> **Version**: 5.2 (Elite)  
> **Language**: Rust  
> **Edition**: 2021

This document provides comprehensive API documentation for the Cliff-Watch core libraries.

---

## Table of Contents

1. [Overview](#overview)
2. [Modules](#modules)
3. [Core Functions](#core-functions)
4. [Data Structures](#data-structures)
5. [Configuration](#configuration)
6. [Examples](#examples)

---

## Overview

Cliff-Watch is organized as a Rust workspace with three main crates:

| Crate | Type | Purpose |
|-------|------|---------|
| `cliff-watch-core` | Library | Core logic, metrics, cryptography |
| `cliff-watch-cli` | Binary | Command-line interface |
| `cliff-watch-daemon` | Binary | Background monitoring service |

### Key Dependencies

```toml
[workspace.dependencies]
git2 = "0.20"           # Git operations
notify = "8.2"          # File system monitoring
zstd = "0.13"           # Compression (NCD calculation)
stat.18"         rs = "0 # Statistical functions
ed25519-dalek = "2.1"   # Cryptographic signatures
sha2 = "0.10"           # Hashing
serde = "1.0"           # Serialization
```

---

## Modules

### `cliff_watch_core::entropy`

Statistical entropy calculations for measuring information density.

```rust
use cliff_watch_core::entropy::{shannon_entropy, normalized_entropy};
```

#### Functions

##### `shannon_entropy(data: &[u8`

Comput]) -> f64es Shannon entropy in bits per byte.

**Parameters:**
- `data`: Input byte slice

 `**Returns:**
-f64`: Entropy value (0.0 to 8.0)

**Example:**
```rust
use cliff_watch_core::entropy::shannon_entropy;

let data = b"Hello, World!";
let entropy = shannon_entropy(data);
println!("Entropy: {:.2} bits/byte", entropy);
```

##### `normalized_entropy(data: &[u8]) -> f64`

Computes normalized entropy in range [0.0, 1.0].

**Parameters:**
- `data`: Input byte slice

**Returns:**
- `f64`: Normalized entropy value

---

### `cliff_watch_core::stats`

Statistical metrics for human/AI detection.

```rust
use cliff_watch_core::stats::{
    calculate_burstiness,
    calculate_human_score,
    calculate_ncd,
    is_synthetic_pattern,
    ThermodynamicReport,
};
```

#### Functions

##### `calculate_burstiness(times: &[f64]) -> f64`

Calculates the burstiness of edit intervals.

**Parameters:**
- `times`: Array of timestamps (in seconds)

**Returns:**
- `f64`: Burstiness coefficient (-1.0 to 1.0)
  - `-1.0`: Perfectly regular (AI-like)
  - `0.0`: Random
  - `> 0.0`: Bursty (human-like)

**Example:**
```rust
use cliff_watch_core::stats::calculate_burstiness;

// Human-like: sporadic editing
let human_times = vec![0.0, 0.5, 10.0, 10.5, 25.0];
let human_burst = calculate_burstiness(&human_times);

// AI-like: regular intervals
let ai_times = vec![1.0, 2.0, 3.0, 4.0, 5.0];
let ai_burst = calculate_burstiness(&ai_times);

println!("Human: {:.2}, AI: {:.2}", human_burst, ai_burst);
```

##### `calculate_ncd(x: &[u8], y: &[u8]) -> f64`

Computes Normalized Compression Distance between two byte sequences.

**Parameters:**
- `x`: First byte sequence
- `y`: Second byte sequence

**Returns:**
- `f64`: NCD value (0.0 to 1.0)
  - `0.0`: Identical
  - `1.0`: Completely different

**Example:**
```rust
use cliff_watch_core::stats::calculate_ncd;

let original = b"fn main() { println!(\"Hello\"); }";
let copy = b"fn main() { println!(\"Hello\"); }";
let different = b"def main():\n    print('Hello')";

let ncd_copy = calculate_ncd(original, copy);
let ncd_diff = calculate_ncd(original, different);

println!("Same: {:.3}, Different: {:.3}", ncd_copy, ncd_diff);
```

##### `calculate_human_score(...) -> f64`

Calculates the composite human probability score.

**Parameters:**
- `burstiness`: Burstiness coefficient
- `ncd`: Normalized Compression Distance
- `focus_mins`: Focus time in minutes
- `nav_events`: Number of navigation events
- `is_synthetic`: Whether pattern is synthetic

**Returns:**
- `f64`: Human score (0.0 to 1.0)

**Formula:**
```
Score = 0.4 * FocusScore + 0.4 * BurstinessScore + 0.2 * NCDScore
```

**Example:**
```rust
use cliff_watch_core::stats::calculate_human_score;

// High human score
let high_score = calculate_human_score(
    0.9,   // High burstiness
    0.2,   // Low NCD (original)
    5.0,   // 5 minutes of focus
    20,    // 20 navigation events
    false  // Not synthetic
);

// Low human score
let low_score = calculate_human_score(
    -0.9,  // Regular intervals
    0.7,   // High NCD (copied)
    0.1,   // Little focus
    2,     // Few navigation events
    false
);
```

##### `is_synthetic_pattern(times: &[f64]) -> bool`

Detects if timing pattern is mechanical/bot-like.

**Parameters:**
- `times`: Array of timestamps

**Returns:**
- `bool`: `true` if synthetic pattern detected

**Note:** Uses Coefficient of Variation (CV < 0.15 indicates synthetic)

---

### `cliff_watch_core::crypto`

Cryptographic operations for proof generation.

```rust
use cliff_watch_core::crypto::{sign_manifest, verify_signature};
```

---

### `cliff_watch_core::config`

Configuration management.

```rust
use cliff_watch_core::config::{Config, Difficulty};
```

---

### `cliff_watch_core::git`

Git repository operations.

```rust
use cliff_watch_core::git::{get_commit_diff, inject_trailer};
```

---

### `cliff_watch_core::monitor`

File system monitoring for the daemon.

```rust
use cliff_watch_core::monitor::{start_monitoring, stop_monitoring};
```

---

## Core Functions

### `sentinel_self_check()`

Verifies system integrity and dependencies.

```rust
use cliff_watch_core::sentinel_self_check;

fn main() {
    match sentinel_self_check() {
        Ok(report) => println!("{}", report),
        Err(e) => eprintln!("System check failed: {}", e),
    }
}
```

**Output:**
```
Sentinel System Integrity Check:
✔ Git2 (libgit2): Static Linked
✔ Crypto (Ed25519): Key Generation Subsystem Active
✔ Hashing (SHA256): Engine Online
✔ Entropy Engine (Zstd): Compression Active (Ratio 3.50x)
✔ Statistics (Statrs): Compute Modules Loaded (Mean: 3.0, StdDev: 1.4142)
```

---

## Data Structures

### `ThermodynamicReport`

```rust
use cliff_watch_core::stats::ThermodynamicReport;

let report = ThermodynamicReport {
    h_score: 0.85,           // Human probability score
    burstiness: 0.72,        // Burstiness coefficient
    pareto_alpha: 2.1,       // Pareto distribution alpha
    ncd_ratio: 0.23,         // Normalized compression distance
    is_human: true,          // Pass/fail decision
    zkp_commitment: Some("abc123..."),  // Optional ZK commitment
};

// Serialize to JSON
let json = serde_json::to_string_pretty(&report).unwrap();
```

### `Config`

```rust
use cliff_watch_core::config::{Config, Difficulty};

let config = Config {
    governance: GovernanceConfig {
        difficulty: Difficulty::Normal,
        min_entropy: 2.5,
        audit_mode: true,
    },
    monitoring: MonitoringConfig {
        debounce_window_ms: 500,
        ignore_extensions: vec!["log".to_string(), "lock".to_string()],
    },
};
```

---

## Configuration

### `cliff-watch.toml`

```toml
[governance]
# Difficulty: Easy, Normal, Hardcore
difficulty = "Normal"
# Minimum entropy required (2.5 bits/byte default)
min_entropy = 2.5
# Audit Mode: warn instead of block
audit_mode = true

[monitoring]
debounce_window_ms = 500
ignore_extensions = ["log", "lock", "tmp", "json"]
```

### Difficulty Levels

| Level | Min Entropy | Description |
|-------|-------------|-------------|
| `Easy` | 1.5 | Relaxed validation |
| `Normal` | 2.5 | Balanced (recommended) |
| `Hardcore` | 4.0 | Strict validation |

---

## Examples

### Basic Usage

```rust
use cliff_watch_core::stats::{calculate_human_score, calculate_burstiness};

fn main() {
    // Simulate edit timestamps
    let edit_times = vec![
        0.0,   // Start
        0.2,   // Quick edit
        5.0,   // Think...
        5.3,   // Quick edit
        15.0,  // Think more
        15.2,  // Another edit
    ];
    
    let burstiness = calculate_burstiness(&edit_times);
    let score = calculate_human_score(
        burstiness,
        0.3,       // NCD against repo
        1.0,       // 1 minute focus
        5,         // 5 navigation events
        false
    );
    
    println!("Burstiness: {:.2}", burstiness);
    println!("Human Score: {:.2}%", score * 100.0);
}
```

### Integration with Git

```rust
use cliff_watch_core::git::{get_repository, get_commit_diff};

fn verify_commit(repo_path: &str, commit_hash: &str) -> Result<()> {
    let repo = git2::Repository::open(repo_path)?;
    let commit = repo.find_commit(git2::Oid::from_str(commit_hash)?)?;
    
    let diff = get_commit_diff(&repo, &commit)?;
    let ncd = calculate_ncd(diff.as_bytes(), reference.as_bytes());
    
    Ok(())
}
```

### Running Tests

```bash
# Run all tests
cargo test --workspace

# Run specific module tests
cargo test -p cliff-watch-core stats

# Run with output
cargo test -p cliff-watch-core -- --nocapture
```

---

## Related Documentation

- [Main README](../../README.md)
- [VSCode Extension Guide](../cliff-watch-witness.md)
- [Scientific Validation](../validation_report.md)
- [Configuration Guide](../configuration.md)
