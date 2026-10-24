mod common;
use spire_report::{create_trace,OutputReservation};
use std::{fs,io::Write};

#[test]
fn output_exclusive() {
    let scratch=common::Scratch::new();let html=scratch.file("report.html");let trace=scratch.file("trace.jsonl");
    fs::write(&html,b"keep html").unwrap();fs::write(&trace,b"keep trace").unwrap();
    assert!(OutputReservation::new(&html).is_err());assert!(create_trace(&trace).is_err());
    assert_eq!(fs::read(&html).unwrap(),b"keep html");assert_eq!(fs::read(&trace).unwrap(),b"keep trace");
    for name in ["dangling.html","dangling.jsonl"] {
        let path=scratch.file(name);std::os::unix::fs::symlink("missing-target",&path).unwrap();
        assert!(OutputReservation::new(&path).is_err());assert!(create_trace(&path).is_err());
        assert!(fs::symlink_metadata(path).unwrap().file_type().is_symlink());
    }
    let raced=scratch.file("race.html");let mut reservation=OutputReservation::new(&raced).unwrap();
    reservation.writer().write_all(b"new html").unwrap();fs::write(&raced,b"concurrent html").unwrap();
    assert!(reservation.publish().is_err());assert_eq!(fs::read(raced).unwrap(),b"concurrent html");
}

#[test]
fn output_publishes_only_finished_html_and_retains_failed_trace() {
    let scratch=common::Scratch::new();let html=scratch.file("report.html");let trace=scratch.file("trace.jsonl");
    create_trace(&trace).unwrap().write_all(b"recoverable trace\n").unwrap();
    {
        let mut reservation=OutputReservation::new(&html).unwrap();
        reservation.writer().write_all(b"incomplete").unwrap();assert!(!html.exists());
    }
    assert_eq!(fs::read(&trace).unwrap(),b"recoverable trace\n");assert!(!html.exists());
    let mut reservation=OutputReservation::new(&html).unwrap();reservation.writer().write_all(b"finished html").unwrap();
    reservation.publish().unwrap();assert_eq!(fs::read(html).unwrap(),b"finished html");
    assert_eq!(fs::read_dir(&scratch.0).unwrap().count(),2);
}
