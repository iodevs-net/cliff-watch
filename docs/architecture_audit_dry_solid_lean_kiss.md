# Cliff-Watch Architecture Audit
## DRY, SOLID, LEAN, and KISS Compliance Report

**Date:** 2026-02-16  
**Scope:** Full cliff-watch architecture analysis  
**Version:** v2.0/v3.0 transition phase

---

## Executive Summary

The cliff-watch architecture exhibits significant violations across all four quality principles (DRY, SOLID, LEAN, KISS). The most critical issues are:

1. **Monolithic CLI** (788 lines) - Single Responsibility violation
2. **Duplicate metric calculations** - DRY violations across modules
3. **Unused dependencies** - LEAN violation (bulletproofs, tss-es-api)
4. **Complex formulas** - KISS violation (LDLJ calculation)
5. **Hardcoded sensor types** - Open/Closed violation
6. **Future architecture not integrated** - Architectural debt

**Recommendation Priority:** High - Address before v3.0 release

---

## 1. DRY (Don't Repeat Yourself) Violations

### 1.1 Duplicate Metric Calculations

**Severity:** High  
**Files Affected:**
- [`crates/cliff-watch-core/src/stats.rs`](crates/cliff-watch-core/src/stats.rs)
- [`feature/future-architecture/temporal.rs`](feature/future-architecture/temporal.rs)
- [`feature/future-architecture/structural.rs`](feature/future-architecture/structural.rs)
- [`feature/future-architecture/kinematic.rs`](feature/future-architecture/kinematic.rs)
- [`crates/cliff-watch-core/src/mouse_sentinel.rs`](crates/cliff-watch-core/src/mouse_sentinel.rs)

**Issue:** Burstiness calculation is duplicated:

```rust
// In stats.rs (line 8-26)
pub fn calculate_burstiness(times: &[f64]) -> f64 {
    let intervals = get_intervals(times);
    let mean = intervals.iter().sum::<f64>() / intervals.len() as f64;
    let std_dev = calculate_std_dev(&intervals, mean);
    (std_dev - mean) / (std_dev + mean)
}

// In temporal.rs (line 90-95)
let burstiness = if mean + stddev == 0.0 {
    0.0
} else {
    (stddev - mean) / (stddev + mean)
};
```

**Recommendation:** Consolidate into a single module `crates/cliff-watch-core/src/metrics/temporal.rs`

---

### 1.2 NCD Calculation Duplication

**Severity:** Medium  
**Files Affected:**
- [`crates/cliff-watch-core/src/stats.rs`](crates/cliff-watch-core/src/stats.rs:89-113)
- [`feature/future-architecture/structural.rs`](feature/future-architecture/structural.rs:67-94)
- [`crates/cliff-watch-core/src/mouse_sentinel.rs`](crates/cliff-watch-core/src/mouse_sentinel.rs)

**Issue:** NCD formula implemented three times with slight variations.

**Recommendation:** Create `crates/cliff-watch-core/src/metrics/structural.rs` with canonical implementation.

---

### 1.3 Kinematic Analysis Duplication

**Severity:** Medium  
**Files Affected:**
- [`crates/cliff-watch-core/src/mouse_sentinel.rs`](crates/cliff-watch-core/src/mouse_sentinel.rs:150-363)
- [`feature/future-architecture/kinematic.rs`](feature/future-architecture/kinematic.rs)

**Issue:** Velocity, acceleration, and jerk calculations duplicated.

**Recommendation:** Use `feature/future-architecture/kinematic.rs` as canonical source and deprecate `mouse_sentinel.rs` kinematic logic.

---

## 2. SOLID Violations

### 2.1 Single Responsibility - CLI Monolith

**Severity:** Critical  
**File:** [`crates/cliff-watch-cli/src/main.rs`](crates/cliff-watch-cli/src/main.rs:1-788)

**Issue:** Single file handles:
- Command-line parsing (lines 9-114)
- System checks (lines 186-200)
- Repository initialization (lines 202-236)
- Git hook management (lines 238-249)
- Report generation (lines 132-184)
- Metrics display (lines 250+)
- Configuration management
- Daemon lifecycle management

**Recommendation:** Split into:
```
crates/cliff-watch-cli/src/
├── main.rs (entry point only)
├── commands/
│   ├── mod.rs
│   ├── system_check.rs
│   ├── init.rs
│   ├── report.rs
│   ├── metrics.rs
│   └── daemon.rs
└── config.rs
```

---

### 2.2 Single Responsibility - Monitor Monolith

**Severity:** High  
**File:** [`crates/cliff-watch-core/src/monitor.rs`](crates/cliff-watch-core/src/monitor.rs:1-957)

**Issue:** Single module handles:
- File system watching (lines 1-450)
- Git monitoring (lines 450+)
- Focus tracking
- Mouse event processing
- Metrics aggregation

**Recommendation:** Split into:
```
crates/cliff-watch-core/src/monitor/
├── mod.rs
├── file_watcher.rs
├── git_monitor.rs
├── focus_tracker.rs
└── metrics_aggregator.rs
```

---

### 2.3 Open/Closed - Hardcoded Sensor Types

**Severity:** High  
**File:** [`crates/cliff-watch-core/src/backend/ide_sensor.rs`](crates/cliff-watch-core/src/backend/ide_sensor.rs:1-256)

**Issue:** Sensor event types are hardcoded in enum. Adding new sensor types requires modifying core code.

```rust
// Hardcoded event types (lines 27-91)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SensorEvent {
    FocusGained { ... },
    FocusLost { ... },
    EditBurst { ... },
    Navigation { ... },
    Heartbeat { ... },
    Disconnect { ... },
    Keystroke { ... },
}
```

**Recommendation:** Use trait-based sensor registration:
```rust
pub trait Sensor: Send + Sync {
    fn event_type(&self) -> &'static str;
    fn serialize(&self) -> Result<Vec<u8>>;
}

pub struct SensorRegistry {
    sensors: HashMap<String, Box<dyn Sensor>>,
}
```

---

### 2.4 Dependency Inversion - Direct evdev Dependency

**Severity:** Medium  
**File:** [`crates/cliff-watch-core/src/backend/linux.rs`](crates/cliff-watch-core/src/backend/linux.rs:1-126)

**Issue:** Direct dependency on `evdev` crate violates Dependency Inversion Principle. Core depends on concrete Linux-specific implementation.

**Recommendation:** Create abstraction:
```rust
// In backend/mod.rs
pub trait InputDevice: Send + Sync {
    fn read_events(&self) -> Result<Vec<InputEvent>>;
    fn device_name(&self) -> &str;
}

// Linux implementation depends on abstraction, not vice versa
pub struct LinuxInputDevice {
    device: evdev::Device,
}

impl InputDevice for LinuxInputDevice { ... }
```

---

### 2.5 Single Responsibility - IPC Module

**Severity:** Medium  
**File:** [`crates/cliff-watch-daemon/src/ipc.rs`](crates/cliff-watch-daemon/src/ipc.rs:1-267)

**Issue:** IPC module handles:
- Socket management
- Protocol serialization
- Metrics calculation
- Business logic
- Response formatting

**Recommendation:** Split into:
```
crates/cliff-watch-daemon/src/
├── ipc/
│   ├── mod.rs
│   ├── server.rs (socket management)
│   ├── protocol.rs (serialization)
│   └── handlers.rs (business logic)
```

---

## 3. LEAN Violations

### 3.1 Unused Dependencies

**Severity:** Medium  
**File:** [`crates/cliff-watch-core/Cargo.toml`](crates/cliff-watch-core/Cargo.toml:32-36)

**Issue:** Heavy dependencies included but minimally used:
- `bulletproofs = "5.0"` - Only used in optional ZKP module
- `tss-esapi = "7.5"` - Only used in optional TPM module
- `syn = "2.0"` - Optional feature, adds significant compile time

**Recommendation:** 
1. Move `bulletproofs` and `tss-esapi` to optional features
2. Create separate crate `cliff-watch-crypto` for advanced crypto operations
3. Default build should not include these dependencies

```toml
[features]
default = []
zkp = ["bulletproofs", "merlin"]
tpm = ["tss-esapi"]
ast-analysis = ["syn"]
```

---

### 3.2 Future Architecture Not Integrated

**Severity:** High  
**Directory:** [`feature/future-architecture/`](feature/future-architecture/)

**Issue:** 12 modules exist but are not integrated:
- `entropy_fusion.rs`
- `kinematic.rs`
- `provenance.rs`
- `sentinel_builder.rs`
- `sentinel_collector.rs`
- `sentinel_finalize.rs`
- `sentinel_hash.rs`
- `sentinel_pow.rs`
- `sentinel_report.rs`
- `sentinel_sign.rs`
- `sentinel_signer.rs`
- `stats.rs`
- `structural.rs`
- `temporal.rs`

**Recommendation:** 
1. Integrate or remove - "In progress" code is waste
2. Create migration plan from current modules to future architecture
3. Update [`docs/roadmaps/architecture_correction_roadmap.md`](docs/roadmaps/architecture_correction_roadmap.md) with concrete timeline

---

### 3.3 Legacy Hardware Capture Code

**Severity:** Low  
**File:** [`crates/cliff-watch-core/src/backend/linux.rs`](crates/cliff-watch-core/src/backend/linux.rs:1-126)

**Issue:** Legacy evdev backend is deprecated but still present, adding maintenance burden.

**Recommendation:** Move to separate crate `cliff-watch-legacy` with feature flag, remove from core.

---

## 4. KISS Violations

### 4.1 Complex LDLJ Formula

**Severity:** Medium  
**File:** [`crates/cliff-watch-core/src/mouse_sentinel.rs`](crates/cliff-watch-core/src/mouse_sentinel.rs:150-363)

**Issue:** Log-Dimensional Less Jerk (LDLJ) calculation is overly complex and difficult to understand.

**Recommendation:** Simplify to standard kinematic metrics:
```rust
pub struct KinematicMetrics {
    pub mean_velocity: f64,
    pub mean_acceleration: f64,
    pub velocity_variance: f64,
    pub path_smoothness: f64,  // Simpler than LDLJ
}
```

---

### 4.2 Overly Complex IPC Protocol

**Severity:** Low  
**File:** [`crates/cliff-watch-core/src/protocol.rs`](crates/cliff-watch-core/src/protocol.rs:1-62)

**Issue:** Protocol has redundant response types and nested structures.

**Recommendation:** Simplify to minimal required messages:
```rust
pub enum Request {
    GetStatus,
    GetMetrics,
    GetWitness,
}

pub enum Response {
    Status(StatusData),
    Metrics(MetricsData),
    Witness(WitnessData),
    Error(String),
}
```

---

### 4.3 Multiple Abstraction Layers

**Severity:** Low  
**Files:**
- [`crates/cliff-watch-core/src/backend/mod.rs`](crates/cliff-watch-core/src/backend/mod.rs)
- [`crates/cliff-watch-core/src/backend/ide_sensor.rs`](crates/cliff-watch-core/src/backend/ide_sensor.rs)
- [`crates/cliff-watch-daemon/src/main.rs`](crates/cliff-watch-daemon/src/main.rs)

**Issue:** Backend system has unnecessary abstraction layers for simple socket communication.

**Recommendation:** Flatten to direct socket communication with clear interface.

---

## 5. Module Boundary Issues

### 5.1 Core Lacks Clear Separation

**Severity:** High  
**File:** [`crates/cliff-watch-core/src/lib.rs`](crates/cliff-watch-core/src/lib.rs:1-14)

**Issue:** Core module exposes everything without clear public API boundary.

```rust
pub mod backend;
pub mod protocol;
pub mod focus_protocol;
pub mod focus_session;
pub mod crypto;
pub mod entropy;
pub mod git;
pub mod monitor;
pub mod mouse_sentinel;
pub mod stats;
pub mod complexity;
pub mod config;
pub mod ui_templates;
```

**Recommendation:** Define clear public API:
```rust
// Public API
pub mod crypto;
pub mod config;
pub mod protocol;

// Internal modules (not re-exported)
mod backend;
mod monitor;
mod mouse_sentinel;
// etc.
```

---

### 5.2 Daemon Tightly Coupled to Core Internals

**Severity:** Medium  
**File:** [`crates/cliff-watch-daemon/src/main.rs`](crates/cliff-watch-daemon/src/main.rs:1-128)

**Issue:** Daemon directly accesses internal core types:
```rust
use cliff_watch_core::monitor::GitMonitorConfig;
use cliff_watch_core::monitor::FileMonitor;
use cliff_watch_core::monitor::GitMonitor;
```

**Recommendation:** Create daemon-specific API in core:
```rust
// In cliff-watch-core/src/daemon_api.rs
pub fn create_daemon(config: DaemonConfig) -> Result<DaemonHandle>;

pub struct DaemonHandle {
    pub metrics: MetricsRef,
    pub shutdown: ShutdownSignal,
}
```

---

### 5.3 CLI Depends on Core Implementation Details

**Severity:** Medium  
**File:** [`crates/cliff-watch-cli/src/main.rs`](crates/cliff-watch-cli/src/main.rs:2)

**Issue:** CLI directly uses internal core functions:
```rust
use cliff_watch_core::{sentinel_self_check, git::{open_repository}, crypto::generate_keypair};
```

**Recommendation:** Create CLI-specific API layer.

---

## 6. Abstraction Layer Problems

### 6.1 Backend Abstraction Incomplete

**Severity:** High  
**Files:**
- [`crates/cliff-watch-core/src/backend/mod.rs`](crates/cliff-watch-core/src/backend/mod.rs:1-81)
- [`crates/cliff-watch-core/src/backend/ide_sensor.rs`](crates/cliff-watch-core/src/backend/ide_sensor.rs)

**Issue:** Backend trait exists but `IdeSensorBackend` doesn't implement it. Two parallel backend systems.

**Recommendation:** Unify under single `Backend` trait:
```rust
pub trait Backend: Send + Sync {
    async fn start(&self, tx: mpsc::Sender<SensorEvent>, shutdown: CancellationToken) -> Result<()>;
}
```

---

### 6.2 Crypto Abstraction Mixed Concerns

**Severity:** Medium  
**Files:**
- [`crates/cliff-watch-core/src/crypto.rs`](crates/cliff-watch-core/src/crypto.rs:1-101)
- [`crates/cliff-watch-core/src/crypto/zkp.rs`](crates/cliff-watch-core/src/crypto/zkp.rs)
- [`crates/cliff-watch-core/src/crypto/tpm.rs`](crates/cliff-watch-core/src/crypto/tpm.rs)

**Issue:** Crypto module mixes:
- Key generation/management
- Signing/verification
- ZKP proofs
- TPM integration
- Hashing

**Recommendation:** Split into:
```
crates/cliff-watch-core/src/crypto/
├── mod.rs
├── keys.rs (key generation, storage)
├── signing.rs (sign/verify)
├── hashing.rs (SHA256)
├── zkp/ (optional feature)
└── tpm/ (optional feature)
```

---

### 6.3 No Clear Interface Between Daemon and Extension

**Severity:** Medium  
**Files:**
- [`crates/cliff-watch-daemon/src/ipc.rs`](crates/cliff-watch-daemon/src/ipc.rs)
- [`clients/cliff-watch-witness/src/transport.ts`](clients/cliff-watch-witness/src/transport.ts)

**Issue:** Protocol defined in Rust but TypeScript implementation has hardcoded paths and no shared schema.

**Recommendation:** Generate TypeScript types from Rust protocol definition using `ts-rs` or similar tool.

---

## 7. Data Flow Issues

### 7.1 IPC Design Complexity

**Severity:** Medium  
**File:** [`crates/cliff-watch-daemon/src/ipc.rs`](crates/cliff-watch-daemon/src/ipc.rs:1-267)

**Issue:** IPC server has 12 Arc<RwLock<>> references passed around, creating complex data flow.

**Recommendation:** Use single state object with clear access patterns:
```rust
pub struct DaemonState {
    metrics: Arc<RwLock<Metrics>>,
    // ... other state
}

impl DaemonState {
    pub fn get_metrics(&self) -> Metrics { ... }
    pub fn update_metrics(&self, m: Metrics) { ... }
}
```

---

### 7.2 Multiple Event Channels

**Severity:** Low  
**File:** [`crates/cliff-watch-daemon/src/main.rs`](crates/cliff-watch-daemon/src/main.rs:37-39)

**Issue:** Three separate channels for different event types:
```rust
let (_input_tx, input_rx) = mpsc::channel(monitor_config.mouse_buffer_size);
let (sensor_tx, sensor_rx) = mpsc::channel(100);
let (file_tx, file_rx) = mpsc::channel(100);
```

**Recommendation:** Use single typed event channel:
```rust
pub enum DaemonEvent {
    Mouse(InputEvent),
    Sensor(SensorEvent),
    File(FileEvent),
}

let (event_tx, event_rx) = mpsc::channel(1000);
```

---

## 8. Recommended New Architecture

### 8.1 Proposed Module Structure

```
cliff-watch/
├── crates/
│   ├── cliff-watch-core/              # Core library (public API only)
│   │   ├── src/
│   │   │   ├── lib.rs                # Public API exports
│   │   │   ├── crypto/               # Cryptography (split)
│   │   │   │   ├── mod.rs
│   │   │   │   ├── keys.rs
│   │   │   │   ├── signing.rs
│   │   │   │   └── hashing.rs
│   │   │   ├── metrics/              # Consolidated metrics
│   │   │   │   ├── mod.rs
│   │   │   │   ├── temporal.rs       # From future-architecture
│   │   │   │   ├── structural.rs     # From future-architecture
│   │   │   │   └── kinematic.rs     # From future-architecture
│   │   │   ├── protocol/             # Shared protocols
│   │   │   │   ├── mod.rs
│   │   │   │   ├── ipc.rs
│   │   │   │   └── sensor.rs
│   │   │   └── config.rs
│   │   └── Cargo.toml                # Minimal dependencies
│   │
│   ├── cliff-watch-daemon/            # Daemon (simplified)
│   │   ├── src/
│   │   │   ├── main.rs               # Entry point only
│   │   │   ├── daemon.rs             # Daemon logic
│   │   │   ├── state.rs              # State management
│   │   │   └── ipc/                  # IPC split
│   │   │       ├── mod.rs
│   │   │       ├── server.rs
│   │   │       └── handlers.rs
│   │   └── Cargo.toml
│   │
│   ├── cliff-watch-cli/               # CLI (split)
│   │   ├── src/
│   │   │   ├── main.rs               # Entry point only
│   │   │   ├── commands/             # Command handlers
│   │   │   │   ├── mod.rs
│   │   │   │   ├── init.rs
│   │   │   │   ├── report.rs
│   │   │   │   ├── metrics.rs
│   │   │   │   └── daemon.rs
│   │   │   └── output.rs             # Formatting
│   │   └── Cargo.toml
│   │
│   ├── cliff-watch-legacy/           # Legacy code (optional)
│   │   └── src/
│   │       └── linux.rs              # evdev backend
│   │
│   └── cliff-watch-crypto/           # Advanced crypto (optional)
│       ├── src/
│       │   ├── zkp.rs
│       │   └── tpm.rs
│       └── Cargo.toml                # bulletproofs, tss-esapi
│
├── clients/
│   └── cliff-watch-witness/
│       ├── src/
│       │   ├── extension.ts
│       │   ├── transport.ts
│       │   └── types.ts             # Generated from Rust
│       └── package.json
│
└── docs/
    ├── architecture/
    │   ├── ADR-001-metrics-consolidation.md
    │   ├── ADR-002-module-split.md
    │   └── ADR-003-dependency-cleanup.md
    └── roadmaps/
        └── refactoring-roadmap.md
```

---

### 8.2 Migration Path

#### Phase 1: Critical Fixes (Week 1)
1. Split `main.rs` in CLI into command modules
2. Remove unused dependencies from core
3. Consolidate duplicate metric calculations
4. Create Architecture Decision Records

#### Phase 2: Module Separation (Week 2-3)
1. Split `monitor.rs` into focused modules
2. Create `metrics/` module with consolidated implementations
3. Define public API for core library
4. Split `crypto/` into focused modules

#### Phase 3: Integration (Week 4)
1. Integrate future-architecture modules or remove
2. Create daemon-specific API in core
3. Generate TypeScript types from Rust protocol
4. Update documentation

#### Phase 4: Cleanup (Week 5)
1. Move legacy code to separate crate
2. Move advanced crypto to separate crate
3. Finalize architecture documentation
4. Create integration tests

---

## 9. Priority Recommendations

### Immediate (Before v3.0 Release)
1. **Split CLI main.rs** - 788 lines violates Single Responsibility
2. **Remove unused dependencies** - bulletproofs, tss-es-api should be optional
3. **Consolidate metric calculations** - Eliminate DRY violations
4. **Define public API** - Core should expose only what's needed

### High Priority (v3.1)
1. **Split monitor.rs** - 957 lines violates Single Responsibility
2. **Integrate future-architecture** - Decide: integrate or remove
3. **Create crypto split** - Separate concerns in crypto module
4. **Unify backend abstraction** - Single trait for all backends

### Medium Priority (v3.2)
1. **Simplify LDLJ formula** - Use standard kinematic metrics
2. **Create daemon API** - Reduce coupling to core internals
3. **Generate TypeScript types** - Shared protocol definitions
4. **Move legacy code** - Separate crate with feature flag

### Low Priority (Future)
1. **Simplify IPC protocol** - Remove redundant response types
2. **Consolidate event channels** - Single typed channel
3. **Flatten backend layers** - Remove unnecessary abstraction

---

## 10. Architecture Decision Records Needed

1. **ADR-001: Metrics Consolidation** - How to consolidate duplicate metric calculations
2. **ADR-002: Module Split Strategy** - How to split monolithic modules
3. **ADR-003: Dependency Management** - How to handle optional dependencies
4. **ADR-004: Future Architecture Integration** - Integrate or remove future-architecture modules
5. **ADR-005: Public API Definition** - What should core expose publicly

---

## 11. Conclusion

The cliff-watch architecture requires significant refactoring to comply with DRY, SOLID, LEAN, and KISS principles. The most critical issues are:

1. **Monolithic modules** (CLI, monitor) violate Single Responsibility
2. **Duplicate code** violates DRY
3. **Unused dependencies** violate LEAN
4. **Complex formulas** violate KISS
5. **Future architecture not integrated** creates technical debt

**Estimated Effort:** 5 weeks for full migration  
**Risk:** Medium - Breaking changes required  
**Benefit:** High - Improved maintainability, testability, and extensibility

---

## Appendix: File References

### Critical Files
- [`crates/cliff-watch-cli/src/main.rs`](crates/cliff-watch-cli/src/main.rs) - 788 lines, monolithic
- [`crates/cliff-watch-core/src/monitor.rs`](crates/cliff-watch-core/src/monitor.rs) - 957 lines, monolithic
- [`crates/cliff-watch-core/src/backend/ide_sensor.rs`](crates/cliff-watch-core/src/backend/ide_sensor.rs) - Hardcoded sensor types
- [`crates/cliff-watch-core/src/stats.rs`](crates/cliff-watch-core/src/stats.rs) - Duplicate calculations
- [`crates/cliff-watch-core/Cargo.toml`](crates/cliff-watch-core/Cargo.toml) - Unused dependencies

### Future Architecture Files
- [`feature/future-architecture/temporal.rs`](feature/future-architecture/temporal.rs)
- [`feature/future-architecture/structural.rs`](feature/future-architecture/structural.rs)
- [`feature/future-architecture/kinematic.rs`](feature/future-architecture/kinematic.rs)
- [`feature/future-architecture/sentinel_report.rs`](feature/future-architecture/sentinel_report.rs)
- [`feature/future-architecture/entropy_fusion.rs`](feature/future-architecture/entropy_fusion.rs)

### Documentation
- [`docs/principios/dry_lean_solid.md`](docs/principios/dry_lean_solid.md) - Existing principles
- [`docs/roadmaps/architecture_correction_roadmap.md`](docs/roadmaps/architecture_correction_roadmap.md) - Existing roadmap

---

**Report Generated:** 2026-02-16  
**Next Review:** After Phase 1 completion
