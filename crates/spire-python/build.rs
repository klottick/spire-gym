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
//! is checked out, the branch's loose ref (if present), the closest existing
//! ref ancestor (so ref and directory *creation* after `git pack-refs
//! --all --prune` invalidates too), and `packed-refs` in the common git dir
//! (commits change one of them).

use std::path::Path;
use std::process::Command;

#[path = "src/identity.rs"]
#[allow(dead_code)] // build.rs uses only the file-digest half
mod identity;

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

    let manifest_dir = std::env::var_os("CARGO_MANIFEST_DIR").expect("Cargo sets manifest dir");
    let root = Path::new(&manifest_dir).parent().and_then(Path::parent).expect("crate within workspace");
    let (digest_files, digest_dirs) = identity::digest_inputs(root);
    for path in &digest_files {
        println!("cargo:rerun-if-changed={}", path.display());
    }
    for path in &digest_dirs {
        println!("cargo:rerun-if-changed={}", path.display());
    }
    let file_digest = identity::file_digest(root);
    let profile = std::env::var("PROFILE").unwrap_or_else(|_| "unknown".to_string());
    let python_features = python_features(root);
    let dirty = dirty_status(root, &digest_files);
    println!("cargo:rustc-env=SPIRE_FILE_DIGEST={file_digest}");
    println!("cargo:rustc-env=SPIRE_BUILD_PROFILE={profile}");
    println!("cargo:rustc-env=SPIRE_PYTHON_FEATURES={}", python_features.join("\n"));
    println!("cargo:rustc-env=SPIRE_DIRTY={dirty}");

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
            let ref_path = common_dir.join(refname);
            let first_ancestor = ref_path.parent().map(Path::to_path_buf);
            if ref_path.exists() {
                watched.push(ref_path);
            }
            // After `git pack-refs --all --prune` neither the loose ref nor
            // any of its ref directories may exist; a later commit recreates
            // them. Watch the closest *existing* ancestor of the ref path
            // (bounded at the common git dir): cargo watches directory
            // contents recursively, so creating the ref or any intermediate
            // directory under it invalidates the build. This stays within the
            // repository's refs area and never widens to the object database.
            let mut ancestor = first_ancestor;
            while let Some(candidate) = ancestor {
                if candidate.is_dir() {
                    watched.push(candidate);
                    break;
                }
                ancestor = candidate.parent().map(Path::to_path_buf);
            }
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

fn python_features(root: &Path) -> Vec<String> {
    let manifest = std::fs::read_to_string(root.join("crates/spire-python/Cargo.toml"))
        .expect("spire-python manifest readable");
    let section = manifest.split("[features]").nth(1).unwrap_or("").split("\n[").next().unwrap_or("");
    let mut features = Vec::new();
    for line in section.lines() {
        let Some((name, _)) = line.split_once('=') else { continue };
        let name = name.trim();
        if name.is_empty() || name == "default" || name.starts_with('#') { continue; }
        let env_name = format!("CARGO_FEATURE_{}", name.replace('-', "_").to_ascii_uppercase());
        println!("cargo:rerun-if-env-changed={env_name}");
        if std::env::var_os(&env_name).is_some() {
            features.push(format!("spire-python/{name}"));
        }
    }
    features.sort();
    features
}

fn dirty_status(root: &Path, files: &[std::path::PathBuf]) -> &'static str {
    let mut command = Command::new("git");
    command.current_dir(root).args(["status", "--porcelain", "--"]);
    for path in files {
        let Ok(relative) = path.strip_prefix(root) else { return "unknown"; };
        command.arg(relative);
    }
    match command.output() {
        Ok(output) if output.status.success() => {
            if output.stdout.is_empty() { "false" } else { "true" }
        }
        _ => "unknown",
    }
}
