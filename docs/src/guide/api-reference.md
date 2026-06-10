# API Reference

This page documents the public interface of each taktakk crate. Full
rustdoc is generated with `cargo doc --open`.

## Crate overview

| Crate | Purpose | Key public types |
|---|---|---|
| `taktakk-core` | Domain types, use cases, port traits | `CoreError`, `Module`, `LessonState`, `WipeScope` |
| `taktakk-storage` | SQLite repositories, object store, wipe | `Database`, `FsObjectStore`, `wipe::*` |
| `taktakk-content` | `.nmp` parser, Ed25519 verification, install | `install_package`, `InstallOutcome`, `samples::*` |
| `taktakk-module-engine` | Lesson runner, exercise evaluation, catalog | `LessonRunner`, `DashboardView`, `GestureConfig` |
| `taktakk-facade-clock` | Clock domain, stealth unlock/duress parser | `GestureParser`, `GestureOutcome`, `ClockTime` |
| `taktakk-security` | Key slots, Argon2id verifier, audit | `overwrite_all_keys`, `run_security_audit` |
| `taktakk-sync` | Inventory diff, chunk transfer, transport | `LocalInventory`, `build_transfer_plan`, `LocalFsTransport` |
| `taktakk-i18n` | BCP 47 locale, RTL/LTR, string lookup | `I18nBundle`, `LocaleTag`, `TextDirection` |
| `taktakk-a11y` | Accessibility settings, ABDD audit | `A11ySettings`, `audit` |

---

## `taktakk-core`

### Domain: curriculum

```rust
// Curriculum axis
pub enum CurriculumAxis { Shield, Spear }

// Module record (from installed package)
pub struct Module {
    pub module_id: String,
    pub category_id: String,
    pub title_key: String,
    pub version: ModuleVersion,
    pub status: ModuleStatus,
    pub estimated_minutes: Option<u16>,
}

// Lesson record
pub struct Lesson {
    pub lesson_id: String,
    pub module_id: String,
    pub sort_order: u32,
    pub step_count: u32,
}
```

### Domain: progress

```rust
pub struct ResumeState {
    pub profile_id: String,
    pub lesson_id: String,
    pub last_completed_step_order: u32,
    pub updated_at: i64,
}
```

### Use cases

```rust
// Panic wipe
pub fn execute_panic_wipe(
    wipe_coordinator: &dyn WipeCoordinator,
    scope: WipeScope,
) -> CoreResult<WipeResult>;

// Field health check
pub fn run_static_checks() -> HealthReport;

// Sync diff
pub fn plan_download(
    remote: &[SyncInventoryItem],
    local: &[SyncInventoryItem],
) -> Vec<SyncInventoryItem>;
```

### Port traits

```rust
// Storage ports (implemented in taktakk-storage)
pub trait CurriculumRepository: Send + Sync { ... }
pub trait PackageRepository: Send + Sync { ... }
pub trait ProgressRepository: Send + Sync { ... }

// Crypto ports (implemented in taktakk-security)
pub trait HashProvider: Send + Sync { ... }
pub trait SignatureVerifier: Send + Sync { ... }
pub trait WipeCoordinator: Send + Sync { ... }

// Package store port (implemented in taktakk-storage)
pub trait ObjectStore: Send + Sync { ... }
```

---

## `taktakk-storage`

```rust
// Open both databases (runs migrations automatically)
pub struct Database {
    pub facade: SqlitePool,
    pub core: SqlitePool,
}
impl Database {
    pub async fn open(base_dir: &Path) -> StorageResult<Self>;
}

// Content-addressed filesystem object store
pub struct FsObjectStore { ... }
impl ObjectStore for FsObjectStore { ... }
```

### Wipe operations

```rust
pub async fn state_wipe(core: &SqlitePool) -> StorageResult<()>;
pub async fn destroy_key_slots(facade: &SqlitePool) -> StorageResult<()>;
pub async fn hard_wipe(facade: &SqlitePool, core: &SqlitePool) -> StorageResult<()>;
pub async fn factory_reset(facade: &SqlitePool, core: &SqlitePool) -> StorageResult<()>;
pub async fn enforce_log_retention(
    core: &SqlitePool, now: i64, max_age_seconds: i64,
) -> StorageResult<u64>;
```

---

## `taktakk-content`

```rust
// Install a .nmp byte buffer
pub fn install_package(
    raw: &[u8],
    package_id: &str,
    trust_anchors: &[TrustAnchor],
    object_store: &dyn ObjectStore,
    now: i64,
) -> InstallOutcome;

pub enum InstallOutcome {
    Installed { package: ContentPackage },
    Quarantined { reason: String },
}
```

---

## `taktakk-module-engine`

```rust
// Lesson runner
pub struct LessonRunner {
    pub state: LessonState,
}
impl LessonRunner {
    pub fn new(state: LessonState, steps: Vec<StepContent>) -> Self;
    pub fn handle(&mut self, event: RunnerEvent) -> EngineResult<RunnerResponse>;
    pub fn current_step(&self) -> EngineResult<&StepContent>;
}

pub enum RunnerEvent { Advance, Answer(ExerciseAnswer), Back }

pub enum RunnerResponse {
    StepAdvanced { new_order: u32 },
    AnswerCorrect { new_order: u32 },
    AnswerIncorrect { attempts_used: u32, max_attempts: u32 },
    MaxAttemptsReached { new_order: u32 },
    StepBack { new_order: u32 },
    AtFirstStep,
    LessonComplete,
}
```

---

## `taktakk-facade-clock`

```rust
// Gesture parser (stealth unlock)
pub struct GestureParser { ... }
impl GestureParser {
    pub fn new(config: GestureConfig) -> Self;
    pub fn process(&mut self, input: FacadeInput) -> GestureOutcome;
    pub fn reset(&mut self);
}

pub enum GestureOutcome { Idle, Pending, Unlock, Duress }
```

---

## `taktakk-i18n`

```rust
// String lookup with 3-tier fallback
pub struct I18nBundle { ... }
impl I18nBundle {
    pub fn new(fallback: impl Into<String>) -> Self;
    pub fn add_locale(&mut self, locale_tag: impl Into<String>, map: StringMap);
    pub fn get(&self, locale: &LocaleTag, key: &str) -> Option<&str>;
    pub fn t<'a>(&'a self, locale: &LocaleTag, key: &'a str) -> &'a str;
}

// RTL/LTR detection
impl LocaleTag {
    pub fn direction(&self) -> TextDirection;
}
```
