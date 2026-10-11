//! Build-time metadata for the native extension.
//!
//! Emits `SPIRE_REVISION` and `SPIRE_RUST_VERSION` as compile-time constants
//! for `build_info()`. The revision is taken from the `SPIRE_REVISION`
//! environment variable when set, otherwise from `git rev-parse HEAD`,
//! falling back to `unknown` (e.g. source archives).
//!
//! Git inputs are watched so incremental rebuilds refresh the revision when
//! the repository changes, without `cargo clean`: build scripts run from the
//! crate directory, so this watches the *resolved* git dir (worktree-aware),
//! its `HEAD` (branch switches and detachment change it), and, when a branch
//! is checked out, the branch's loose ref plus `packed-refs` in the common
//! git dir (commits change one of them).

use std::path::Path;
use std::process::Command;

/// Run `git` with `args`, returning trimmed stdout on success.
fn git_output(args: &[&str]) -> Option<String> {
    let output = Command::new("git").args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
    (!value.is_empty()).then_some(value)
}

fn main() {
    println!("cargo:rerun-if-env-changed=SPIRE_REVISION");
    println!("cargo:rerun-if-env-changed=RUSTC");

    let revision = std::env::var("SPIRE_REVISION")
        .unwrap_or_else(|_| git_output(&["rev-parse", "HEAD"]).unwrap_or_else(|| "unknown".to_string()));

    let mut watched = Vec::new();
    if let Some(git_dir) = git_output(&["rev-parse", "--absolute-git-dir"]) {
        let git_path = Path::new(&git_dir);
        watched.push(git_path.join("HEAD"));
        let common_dir = git_output(&["rev-parse", "--git-common-dir"])
            .as_ref()
            .and_then(|raw| Path::new(raw).canonicalize().ok());
        if let (Some(common_dir), Some(refname)) = (
            common_dir,
            git_output(&["symbolic-ref", "-q", "HEAD"]),
        ) {
            watched.push(common_dir.join(refname));
            watched.push(common_dir.join("packed-refs"));
        }
    }
    for input in &watched {
        if input.exists() {
            println!("cargo:rerun-if-changed={}", input.display());
        }
    }
    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string());
    let mut rust_version = String::from("unknown");
    if let Ok(output) = Command::new(&rustc).arg("--version").output() {
        if output.status.success() {
            // "rustc 1.99.0 (b940084d7 2026-09-28)" -> "1.99.0"
            let tokens = std::str::from_utf8(&output.stdout)
                .unwrap_or("")
                .split_whitespace()
                .collect::<Vec<&str>>();
            if tokens.len() >= 2 {
                rust_version = tokens[1].to_string();
            }
        }
    }

    println!("cargo:rustc-env=SPIRE_REVISION={revision}");
    println!("cargo:rustc-env=SPIRE_RUST_VERSION={rust_version}");
}
