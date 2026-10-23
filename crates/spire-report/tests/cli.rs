mod common;
use std::{fs,process::Command};

fn command(scratch:&common::Scratch,args:&[&str])->std::process::Output{
    Command::new(env!("CARGO_BIN_EXE_spire-report")).current_dir(&scratch.0).args(args).output().unwrap()
}

#[test]
fn cli_validation() {
    let scratch=common::Scratch::new();
    assert!(command(&scratch,&["--help"]).status.success());
    assert_eq!(fs::read_dir(&scratch.0).unwrap().count(),0);
    for args in [vec![],vec!["--unknown"],vec!["--trace"],vec!["--trace","a","--trace","b","--output","out.html"],
        vec!["--trace","missing","--output","out.html"]]{assert!(!command(&scratch,&args).status.success());}
    fs::write(scratch.file("bad.jsonl"),b"{broken}\n").unwrap();
    assert!(!command(&scratch,&["--trace","bad.jsonl","--output","out.html"]).status.success());
    assert!(!scratch.file("out.html").exists());
    fs::write(scratch.file("trace.jsonl"),common::trace("Real displayed values")).unwrap();
    fs::write(scratch.file("out.html"),b"existing").unwrap();
    assert!(!command(&scratch,&["--trace","trace.jsonl","--output","out.html"]).status.success());
    assert_eq!(fs::read(scratch.file("out.html")).unwrap(),b"existing");
    assert!(!command(&scratch,&["--trace","trace.jsonl","--output","trace.jsonl"]).status.success());
    fs::write(scratch.file("bad-assets.json"),b"{}").unwrap();
    assert!(!command(&scratch,&["--trace","trace.jsonl","--output","absent.html","--assets","bad-assets.json"]).status.success());
    assert!(!scratch.file("absent.html").exists());
}

#[test]
fn cli_interrupted() {
    let scratch=common::Scratch::new();
    let mut records=common::trace("Unfinished recorded prefix").split(|b|*b==b'\n').filter(|line|!line.is_empty()).take(2)
        .flat_map(|line|line.iter().copied().chain(std::iter::once(b'\n'))).collect::<Vec<_>>();
    records.extend(b"{torn");fs::write(scratch.file("trace.jsonl"),&records).unwrap();
    let result=command(&scratch,&["--trace","trace.jsonl","--output","report.html"]);
    assert!(result.status.success(),"{}",String::from_utf8_lossy(&result.stderr));
    let html=fs::read_to_string(scratch.file("report.html")).unwrap();
    let data=html.split("<script id=\"spire-report-data\" type=\"application/json\">").nth(1).unwrap().split("</script>").next().unwrap();
    let report:serde_json::Value=serde_json::from_str(data).unwrap();
    assert_eq!(report["capture"]["status"],"interrupted");assert!(report["game"]["outcome"].is_null());
    assert_eq!(fs::read(scratch.file("trace.jsonl")).unwrap(),records);
}
