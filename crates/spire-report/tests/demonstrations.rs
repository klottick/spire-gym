#[path="common/native.rs"] mod native;
#[path="../examples/run_driver.rs"] mod driver;
use spire_report::*;
use serde_json::Value;
fn report(budget:usize,death:bool)->Value {
    let mut writer=ReportWriter::new(Vec::new(),native::header()).unwrap();
    driver::capture_source(7,budget,death,&mut writer).unwrap();
    read_report(&writer.into_inner()[..],64*1024*1024).unwrap()
}
#[test]
fn demo_run_prefix() {
    let r=report(4096,false);assert_eq!(r["capture"]["status"],"complete");assert_eq!(r["game"]["boundary"],"act2_prefix_complete");assert!(r["game"]["outcome"].is_null());
    let frames=r["frames"].as_array().unwrap();assert!(frames.len()>300);
    assert!(frames.iter().any(|f|f["state"]["combat"]["enemies"].as_array().is_some_and(|a|a.len()>1)));
    assert!(frames.iter().any(|f|f["state"]["screen"]["kind"]=="map"));
    assert!(frames.iter().any(|f|f["combatFinal"]["enemies"].is_array()));
    assert_eq!(frames.last().unwrap()["floor"],26);
}
#[test]
fn demo_death() {let r=report(4096,true);assert_eq!(r["game"]["outcome"],"death");assert_eq!(r["capture"]["status"],"complete");assert_eq!(r["frames"].as_array().unwrap().last().unwrap()["state"]["player"]["hp"],0);}
#[test]
fn demo_budget() {for budget in [0,1] {let r=report(budget,false);assert_eq!(r["capture"]["status"],"unfinished");assert!(r["game"]["outcome"].is_null());assert!(r["game"]["boundary"].is_null());assert_eq!(r["frames"].as_array().unwrap().len(),budget+1);}}
