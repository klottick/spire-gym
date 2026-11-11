//! Versioned source identity framing shared by the build script and runtime.

use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

const FILE_PREFIX: &[u8] = b"spire-file-digest-v1\0";
const SOURCE_PREFIX: &[u8] = b"spire-source-digest-v1\0";

/// Digest inputs are sorted by workspace-relative UTF-8 path and framed as
/// `u32 LE path length || path || u64 LE content length || bytes`, preceded by FILE_PREFIX.
pub fn file_digest(root: &Path) -> String {
    let (files, _) = digest_inputs(root);
    let mut hash = Sha256::new();
    hash.update(FILE_PREFIX);
    for path in files {
        let name = relative_name(&path, root);
        let content = fs::read(&path).expect("digest input readable");
        hash.update(u32::try_from(name.len()).expect("path length fits u32").to_le_bytes());
        hash.update(name.as_bytes());
        hash.update(u64::try_from(content.len()).expect("file length fits u64").to_le_bytes());
        hash.update(content);
    }
    hex_digest(hash.finalize().as_slice())
}

/// SHA-256 framing is `SOURCE_PREFIX || file_digest || NUL || sorted features joined by LF || NUL || profile`.
pub fn source_digest(file_digest: &str, features: &[&str], profile: &str) -> String {
    let mut sorted = features.to_vec();
    sorted.sort_unstable();
    sorted.dedup();
    let mut hash = Sha256::new();
    hash.update(SOURCE_PREFIX);
    hash.update(file_digest.as_bytes());
    hash.update([0]);
    hash.update(sorted.join("\n").as_bytes());
    hash.update([0]);
    hash.update(profile.as_bytes());
    hex_digest(hash.finalize().as_slice())
}

/// Return the sorted union of compile-time feature labels from the three crates.
pub fn features(core: &[&str], env: &[&str], python: &[&str]) -> Vec<String> {
    let mut all = core.iter().chain(env).chain(python)
        .copied().filter(|feature| !feature.ends_with("/default"))
        .map(str::to_owned).collect::<Vec<_>>();
    all.sort_unstable();
    all.dedup();
    all
}

/// Exact file list plus all traversed directories, for content hashing and Cargo watches.
pub fn digest_inputs(root: &Path) -> (Vec<PathBuf>, Vec<PathBuf>) {
    let mut files = Vec::new();
    let mut directories = Vec::new();
    for crate_name in ["spire-core", "spire-env", "spire-python"] {
        let base = root.join("crates").join(crate_name);
        files.push(base.join("Cargo.toml"));
        walk_rust(&base.join("src"), root, &mut files, &mut directories);
    }
    files.extend([
        root.join("crates/spire-python/build.rs"),
        root.join("crates/spire-env/src/standard_vocab_v1.json"),
        root.join("crates/spire-env/src/standard_event_constants_v1.json"),
        root.join("Cargo.toml"),
        root.join("Cargo.lock"),
        root.join("pyproject.toml"),
    ]);
    // Byte order of the `/`-joined relative path string (the Python identity test sorts the same way).
    files.sort_by_key(|path| relative_name(path, root));
    files.dedup();
    directories.sort();
    directories.dedup();
    (files, directories)
}

fn walk_rust(dir: &Path, root: &Path, files: &mut Vec<PathBuf>, directories: &mut Vec<PathBuf>) {
    directories.push(dir.to_path_buf());
    let mut entries = fs::read_dir(dir).expect("source directory readable")
        .map(|entry| entry.expect("source entry readable").path()).collect::<Vec<_>>();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            walk_rust(&path, root, files, directories);
        } else if path.extension().is_some_and(|ext| ext == "rs")
            && !path.strip_prefix(root).unwrap().starts_with("crates/spire-core/src/tests")
            && !path.strip_prefix(root).unwrap().starts_with("crates/spire-env/src/tests")
            && !path.strip_prefix(root).unwrap().starts_with("crates/spire-python/src/tests")
        {
            files.push(path);
        }
    }
}

fn relative_name(path: &Path, root: &Path) -> String {
    let relative = path.strip_prefix(root).expect("digest path under workspace root");
    relative.to_str().expect("workspace paths are UTF-8").replace('\\', "/")
}

fn hex_digest(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}
