//! taktakk development task runner.
//!
//! Run via: `cargo xtask <task>`

fn main() {
    let task = std::env::args().nth(1);
    match task.as_deref() {
        Some("help") | None      => print_help(),
        Some("field-check")      => task_field_check(),
        Some("security-review")  => task_security_review(),
        Some("lint")             => task_lint(),
        Some(other) => {
            eprintln!("Unknown task: {other}");
            print_help();
            std::process::exit(1);
        }
    }
}

fn print_help() {
    println!("taktakk xtask — available commands:");
    println!("  field-check      Run static field-readiness health checks");
    println!("  security-review  Print the security audit checklist");
    println!("  lint             Remind developer to run cargo clippy + fmt");
    println!("  help             Show this help message");
}

fn task_field_check() {
    println!("=== taktakk field-check ===");
    println!("Run: cargo test -p taktakk-core -- field_check");
    println!("     cargo test -p taktakk-a11y -- audit");
    println!("     cargo test -p taktakk-security -- audit");
    println!();
    println!("All static health checks are embedded in the test suite.");
    println!("Run `cargo test` to execute all checks including field readiness.");
}

fn task_security_review() {
    println!("=== taktakk security-review ===");
    println!("Run: cargo test -p taktakk-security -- audit");
    println!();
    println!("See docs/security-reviewers/audit-checklist.md for the full list.");
}

fn task_lint() {
    println!("Run the following before committing:");
    println!("  cargo fmt --all");
    println!("  cargo clippy --all-targets -- -D warnings");
    println!("  cargo test");
}
