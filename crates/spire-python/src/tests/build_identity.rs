use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use super::super::identity::{file_digest, source_digest};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap().to_path_buf()
}

/// Every `include_str!`/`include_bytes!` target of the extension's non-test sources, canonicalized.
fn include_targets(source: &str, dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for needle in ["include_str!(\"", "include_bytes!(\""] {
        let mut rest = source;
        while let Some((_, after)) = rest.split_once(needle) {
            let Some((target, tail)) = after.split_once("\")") else { break };
            out.push(fs::canonicalize(dir.join(target)).unwrap_or_else(|_| dir.join(target)));
            rest = tail;
        }
    }
    out
}

/// Design §3.5: a non-test include target outside the digested set fails. Test-only modules (any `tests` directory
/// under `src/`, compiled only under `cfg(test)`) are skipped: they never reach the extension.
#[test]
fn source_digest_covers_includes() {
    let root = workspace_root();
    // Positive control: the scanner finds a target in a fixture source.
    assert_eq!(include_targets("let x = include_str!(\"a.json\");", Path::new("/r/s")), vec![PathBuf::from("/r/s/a.json")]);
    let mut included = BTreeSet::new();
    let mut scanned = 0;
    for crate_name in ["spire-core", "spire-env", "spire-python"] {
        let src = root.join("crates").join(crate_name).join("src");
        for entry in walkdir(&src) {
            if entry.extension().is_none_or(|e| e != "rs") { continue }
            if entry.strip_prefix(&src).unwrap().components().any(|c| c.as_os_str() == "tests") { continue }
            scanned += 1;
            included.extend(include_targets(&fs::read_to_string(&entry).unwrap(), entry.parent().unwrap()));
        }
    }
    assert!(scanned > 100, "the scan reads the sources ({scanned} files)");
    let vocab = fs::canonicalize(root.join("crates/spire-env/src/standard_vocab_v1.json")).unwrap();
    assert!(included.contains(&vocab), "positive control: the Standard vocabulary include is found");
    // The shipped rule (`identity::digest_inputs`), not a test-local copy.
    let digest_paths = super::super::identity::digest_inputs(&root).0.into_iter().map(|p| fs::canonicalize(&p).unwrap_or(p)).collect::<BTreeSet<_>>();
    for target in &included {
        assert!(digest_paths.contains(target), "include target is not digested: {}", target.display());
    }
}

#[test]
fn source_digest_changes_with_features() {
    let file = "abc";
    let base = ["spire-python/extension"];
    let with_dep = ["spire-env/fuzz-diagnostics", "spire-python/extension"];
    assert_ne!(source_digest(file, &base, "debug"), source_digest(file, &with_dep, "debug"));
    assert_eq!(source_digest(file, &with_dep, "debug"), source_digest(file, &["spire-python/extension", "spire-env/fuzz-diagnostics"], "debug"));
    let actual = super::super::identity::features(&[], &[], &["spire-python/extension"]);
    assert!(!actual.iter().any(|feature| feature.ends_with("/default")));
}

#[test]
fn enabled_features_list_every_feature() {
    for (manifest, source) in [
        ("crates/spire-core/Cargo.toml", "crates/spire-core/src/lib.rs"),
        ("crates/spire-env/Cargo.toml", "crates/spire-env/src/lib.rs"),
        ("crates/spire-python/Cargo.toml", "crates/spire-python/build.rs"),
    ] {
        let manifest_text = fs::read_to_string(workspace_root().join(manifest)).unwrap();
        let feature_section = manifest_text.split("[features]").nth(1).unwrap().split("\n[").next().unwrap();
        let source_text = fs::read_to_string(workspace_root().join(source)).unwrap();
        for line in feature_section.lines() {
            let Some((name, _)) = line.split_once('=') else { continue };
            let name = name.trim();
            if name == "default" || name.is_empty() || name.starts_with('#') { continue; }
            if manifest.contains("spire-python") {
                assert!(source_text.contains("CARGO_FEATURE_") && source_text.contains("spire-python/{name}"), "{} does not enumerate feature {} from Cargo's feature environment", source, name);
            } else {
                assert!(source_text.contains(&format!("{}/{}", manifest.split('/').nth(1).unwrap(), name)), "{} missing feature {}", source, name);
            }
        }
    }
}

#[test]
fn file_digest_matches_build_script() {
    assert_eq!(file_digest(&workspace_root()), env!("SPIRE_FILE_DIGEST"));
}

fn walkdir(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(dir) = pending.pop() {
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() { pending.push(path); } else { out.push(path); }
        }
    }
    out.sort();
    out
}
