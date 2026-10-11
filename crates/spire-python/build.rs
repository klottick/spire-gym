//! Build-time metadata for the native extension.
//!
//! Emits `SPIRE_REVISION` and `SPIRE_RUST_VERSION` as compile-time
//! constants for `build_info()`. The revision is taken from the `SPIRE_REVISION`
//! environment variable when set, otherwise from `git rev-parse HEAD`,
//! falling back to `unknown` (e.g. source archives).

use std::path::Path;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-env-changed=SPIRE_REVISION");
    println!("cargo:rerun-if-env-changed=RUSTC");
    if Path::new(".git").exists() {
        println!("cargo:rerun-if-changed=.git");
    }

    let revision = std::env::var("SPIRE_REVISION").unwrap_or_else(|_| {
        Command::new("git")
            .args(["rev-parse", "HEAD"])
            .output()
            .ok()
            .filter(|output| output.status.success())
            .and_then(|output| {
                let head = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if head.chars().count() == 40 {
                    Some(head)
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "unknown".to_string())
    });

    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string());
    let mut rust_version = String::from("unknown");
    if let Ok(output) = Command::new(&rustc).arg("--version").output()
        && output.status.success()
    {
        // "rustc 1.99.0 (b940084d7 2026-09-28)" -> "1.99.0"
        let mut tokens = std::str::from_utf8(&output.stdout)
            .unwrap_or("")
            .split_whitespace();
        if tokens.next().is_some()
            && let Some(version) = tokens.next()
        {
            rust_version = version.to_string();
        }
    }

    println!("cargo:rustc-env=SPIRE_REVISION={revision}");
    println!("cargo:rustc-env=SPIRE_RUST_VERSION={rust_version}");
}
