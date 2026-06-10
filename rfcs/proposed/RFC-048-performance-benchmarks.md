# RFC-048: Performance Benchmarks (Real Measurement)

| Field | Value |
|---|---|
| **Status** | Proposed |
| **Created** | 2026-05-09 |
| **Milestone** | M11 |
| **Priority** | P1 |
| **Review finding** | Non-functional §10 |

## Problem

RFC-025 defines budget constants (`BUDGET_FACADE_COLD_START_MS = 1500`,
etc.) but `run_static_checks()` only verifies that budget < limit — it
does not measure actual elapsed time for any operation.

The "performance budget is defined" claim in M7 is therefore a promise,
not a proof. M7's field readiness acceptance criteria explicitly require
"profiling and memory leak resolution on 5–10-year-old Android".

Without real measurements:
- No evidence that cold start meets the 3000ms hard limit.
- No evidence that package verification fits the 50 MiB + streaming
  requirement (RFC-039).
- Battery consumption is unverified.
- Memory peaks are unknown.

## Design

### Benchmark harness (`benches/` in taktakk-storage and taktakk-content)

Use `criterion` for Rust-level micro-benchmarks:

```toml
# taktakk-storage/Cargo.toml
[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }

[[bench]]
name = "storage_perf"
harness = false
```

Benchmarks to implement:

```rust
// storage bench
fn bench_resume_state_write(c: &mut Criterion) { ... }
fn bench_object_store_put_64kb(c: &mut Criterion) { ... }
fn bench_object_store_get_64kb(c: &mut Criterion) { ... }

// content bench
fn bench_nmp_verify_1mb(c: &mut Criterion) { ... }
fn bench_nmp_verify_50mb(c: &mut Criterion) { ... }
fn bench_install_package_5steps(c: &mut Criterion) { ... }
```

Each bench asserts against the RFC-025 budget constant and fails the
bench suite if exceeded (using `criterion.BenchmarkGroup.sample_size`
and a manual threshold check).

### `xtask bench` command

```
cargo bench --workspace 2>&1 | tee bench-results.txt
Parse criterion output → extract median times
Compare against RFC-025 budget constants
Generate bench-report.json:
  {
    "resume_write_ms": 42,
    "object_put_ms": 18,
    "nmp_verify_50mb_ms": 890,
    ...
  }
Fail if any value exceeds its hard limit.
```

### Android field measurement

For the release candidate, the following must be measured on a reference
low-end device (e.g. Android 8, 1 GB RAM, ARMv8):

| Metric | Budget | Hard limit |
|---|---|---|
| Facade cold start | 1500 ms | 3000 ms |
| Unlock transition | 1000 ms | 2500 ms |
| Dashboard render | 1500 ms | 3000 ms |
| Step transition | 250 ms | 750 ms |
| Resume state write | 75 ms | 250 ms |
| Package verify (50 MB) | — | < 30 s |
| Memory peak (lesson running) | — | < 200 MB |

Results are included in `release-manifest.json` under `"field_measurements"`.

### Power model

Minimum viable: measure battery percentage drop over a 30-minute learning
session with audio on the reference device. Document result in
`docs/src/contributing/local-development.md`.

## Acceptance criteria

1. `cargo bench -p taktakk-storage` completes without exceeding budget
   constants on CI hardware.
2. `cargo bench -p taktakk-content` verifies a 50 MB package stream
   benchmark within hard limit.
3. `xtask bench` generates `bench-report.json` and exits non-zero if any
   hard limit is exceeded.
4. Release candidate `release-manifest.json` includes
   `"field_measurements"` with at least facade-cold-start and
   package-verify results from Android hardware.
5. `docs/src/contributing/local-development.md` documents how to run the
   bench suite locally.
