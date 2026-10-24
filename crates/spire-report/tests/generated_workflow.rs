mod common;
use spire_report::read_report;
use std::{process::Command,path::PathBuf};

// The native examples are prebuilt by `cargo build -p spire-report --examples`.
// This protects the real process exit status, not just a helper's return value.
#[test]
fn native_budget_process_retains_unfinished_trace() {
    let bin=PathBuf::from(env!("CARGO_BIN_EXE_spire-report")).parent().unwrap().join("examples/capture_run");
    assert!(bin.is_file(),"build native examples first: cargo build -p spire-report --examples --locked --offline");
    let scratch=common::Scratch::new();
    for budget in [0,1] {
        let path=scratch.file(&format!("budget-{budget}.jsonl"));
        let result=Command::new(&bin).args(["--seed","7","--budget",&budget.to_string(),"--scenario","prefix","--trace"]).arg(&path).output().unwrap();
        assert!(!result.status.success());assert!(String::from_utf8_lossy(&result.stderr).contains("unfinished"));
        let bytes=std::fs::read(path).unwrap();let r=read_report(&bytes[..],64*1024*1024).unwrap();
        assert_eq!(r["capture"]["status"],"unfinished");assert!(r["game"]["outcome"].is_null());assert_eq!(r["frames"].as_array().unwrap().len(),budget+1);
    }
}
