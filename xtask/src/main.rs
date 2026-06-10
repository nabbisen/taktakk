//! taktakk development task runner.
//!
//! Run via: `cargo xtask <task>`

fn main() {
    let task = std::env::args().nth(1);
    match task.as_deref() {
        Some("help") | None => print_help(),
        Some(other) => {
            eprintln!("Unknown task: {other}");
            print_help();
            std::process::exit(1);
        }
    }
}

fn print_help() {
    println!("Available tasks:");
    println!("  help  - Show this help message");
}
