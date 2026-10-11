//! Spire Lab developer CLI.
//!
//! Phase 1a scaffolding: the only supported command reports versions.

fn print_version() {
    // The initial core/env crates are documentation-only; the CLI reports
    // its own package metadata only.
    println!("spire {}", env!("CARGO_PKG_VERSION"));
}

fn main() -> std::process::ExitCode {
    match std::env::args().skip(1).next().as_deref() {
        Some("version") | Some("--version") | Some("-V") => {
            print_version();
            std::process::ExitCode::SUCCESS
        }
        Some(other) => {
            eprintln!("spire: unknown command '{other}'");
            eprintln!("usage: spire version");
            std::process::ExitCode::from(2)
        }
        None => {
            eprintln!("usage: spire version");
            std::process::ExitCode::from(2)
        }
    }
}
