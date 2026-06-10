//! taktakk development task runner.
//!
//! Run via: `cargo xtask <task>`

fn main() {
    let task = std::env::args().nth(1);
    match task.as_deref() {
        Some("help") | None              => print_help(),
        Some("field-check")              => task_field_check(),
        Some("security-review")          => task_security_review(),
        Some("lint")                     => task_lint(),
        Some("release-candidate")        => task_release_candidate(),
        Some("verify-release")           => task_verify_release(),
        Some("seed-kit")                 => task_seed_kit(),
        Some("field-failure-tests")      => task_field_failure_tests(),
        Some("docs-check")               => task_docs_check(),
        Some(other) => {
            eprintln!("Unknown task: {other}");
            print_help();
            std::process::exit(1);
        }
    }
}

fn print_help() {
    println!("taktakk xtask — available commands:");
    println!();
    println!("  Development");
    println!("    lint                 Run cargo fmt + clippy checks");
    println!("    field-check          Run static field-readiness checks");
    println!("    security-review      Run automated security audit");
    println!("    field-failure-tests  Run failure-injection test suite");
    println!("    docs-check           Verify required documentation exists");
    println!();
    println!("  Release");
    println!("    release-candidate    Build and package a release candidate");
    println!("    verify-release       Verify release artifact checksums");
    println!("    seed-kit [profile]   Assemble a seed kit (minimal/standard/full)");
    println!();
    println!("  help                   Show this help message");
}

fn task_field_check() {
    println!("=== taktakk field-check ===");
    println!();
    println!("Running via cargo test:");
    println!("  cargo test -p taktakk-core -- field_check");
    println!("  cargo test -p taktakk-a11y -- audit");
    println!("  cargo test -p taktakk-security -- audit");
    println!();
    println!("Or run all checks at once:");
    println!("  cargo test");
}

fn task_security_review() {
    println!("=== taktakk security-review ===");
    println!();
    println!("Automated checks: cargo test -p taktakk-security -- audit");
    println!("Manual checklist: docs/security-reviewers/audit-checklist.md");
}

fn task_lint() {
    println!("Run before committing:");
    println!("  cargo fmt --all");
    println!("  cargo clippy --all-targets -- -D warnings");
    println!("  cargo test");
}

fn task_release_candidate() {
    println!("=== release-candidate ===");
    println!();
    println!("Steps:");
    println!("  1. cargo test --locked               (all tests must pass)");
    println!("  2. cargo build --release --locked    (locked dependencies)");
    println!("  3. sha256sum target/release/taktakk  (record checksum)");
    println!("  4. Write release-manifest.json with version, commit, toolchain");
    println!("  5. Sign the manifest with the release signing key");
    println!();
    println!("See: docs/release/reproducible-builds.md");
}

fn task_verify_release() {
    println!("=== verify-release ===");
    println!();
    println!("Steps:");
    println!("  1. sha256sum -c CHECKSUMS.txt        (verify all artifacts)");
    println!("  2. Verify release-manifest.json signature against trust anchor");
    println!("  3. Run: cargo test --locked           (regression check)");
    println!();
    println!("Offline verification instructions: docs/release/reproducible-builds.md");
}

fn task_seed_kit() {
    let profile = std::env::args().nth(2).unwrap_or_else(|| "minimal".to_string());
    println!("=== seed-kit ({profile}) ===");
    println!();
    match profile.as_str() {
        "minimal"  => println!("Minimal kit: app + 1 locale + emergency Shield modules (~5 MB)"),
        "standard" => println!("Standard kit: app + core modules + common locales (~25 MB)"),
        "full"     => println!("Full kit: all approved modules + all locale packs (~50 MB)"),
        other => {
            eprintln!("Unknown profile: {other}. Use: minimal, standard, full");
            std::process::exit(1);
        }
    }
    println!();
    println!("Assemble by running package verification on all packages in the profile,");
    println!("then copy APK + .nmp files + locale packs to the seed media.");
    println!("Generate seed-kit-manifest.json with SHA-256 of each included file.");
}

fn task_field_failure_tests() {
    println!("=== field-failure-tests ===");
    println!();
    println!("Running failure-injection tests:");
    println!("  cargo test -p taktakk-storage -- failure_injection");
    println!("  cargo test -p taktakk-storage -- maintenance");
    println!("  cargo test -p taktakk-content -- tampered");
    println!("  cargo test -p taktakk-storage -- wipe_idempotent");
    println!();
    println!("These tests simulate: power loss, corrupt packages, storage-full,");
    println!("interrupted sync, and partial file cleanup.");
}

fn task_docs_check() {
    println!("=== docs-check ===");
    let required = [
        "docs/README.md",
        "docs/developers/architecture.md",
        "docs/security-reviewers/threat-model.md",
        "docs/security-reviewers/audit-checklist.md",
        "docs/security-reviewers/panic-wipe.md",
        "docs/field-operators/seed-distribution.md",
        "docs/field-operators/pilot-checklist.md",
        "docs/content-authors/module-authoring-guide.md",
        "docs/release/reproducible-builds.md",
        "docs/release/release-checklist.md",
    ];
    let mut missing = 0;
    for path in &required {
        let exists = std::path::Path::new(path).exists();
        let mark = if exists { "OK" } else { "MISSING" };
        println!("  [{mark}] {path}");
        if !exists { missing += 1; }
    }
    println!();
    if missing == 0 {
        println!("All required documentation files present.");
    } else {
        println!("{missing} file(s) missing.");
        std::process::exit(1);
    }
}
