use serde_json::{Value, json};
use spire_report::{ReportWriter, read_report};
use std::{cell::Cell, io::{self, Cursor, Write}, rc::Rc};

fn header() -> Value {
    json!({"record":"header","schemaId":"spire-report-v1","schemaVersion":1,
        "source":{"kind":"run_prefix","profile":"m6","producerRevision":"5b1cd19",
            "contentId":"native","evidenceGrade":"source-grounded"},
        "run":{"character":"ironclad","ascension":0,
            "seed":{"encoding":"decimal","value":"18446744073709551616"},"caller":null}})
}

fn frame(id: &str, previous: Option<&str>) -> Value {
    json!({"id":id,"previousFrameId":previous,"kind":if previous.is_none(){"initial"}else{"boundary"},
        "act":1,"floor":0,"combatId":null,"turn":null,"action":null,"beforeScreen":null,
        "state":{"screen":{"kind":"map","name":"Map","options":[],"selection":null},
            "player":{"hp":80,"maxHp":80,"gold":99,"block":null,"energy":null,"statuses":[]},
            "inventory":null,"map":null,"combat":null,"terminal":null},
        "effects":[],"combatFinal":null,"stateValidity":"settled"})
}

fn lines(records: &[Value]) -> Vec<u8> {
    let mut out=Vec::new();
    for record in records { out.extend(serde_json::to_vec(record).unwrap()); out.push(b'\n'); }
    out
}

fn footer() -> Value {
    json!({"record":"footer","capture":{"status":"complete","reason":null},
        "game":{"outcome":null,"boundary":"act2_prefix_complete"}})
}

fn complete() -> Vec<u8> {
    lines(&[header(),json!({"record":"frame","frame":frame("0",None)}),footer()])
}

#[test]
fn protocol_round_trip() {
    // Losing string IDs, catalog definitions or a historical boundary corrupts display evidence.
    let mut writer=ReportWriter::new(Vec::new(),header()).unwrap();
    writer.catalog(json!({"card/strike":{"name":"Strike","assetKey":"card/strike"}})).unwrap();
    writer.frame(frame("0",None)).unwrap();
    writer.frame(frame("1",Some("0"))).unwrap();
    writer.finish(footer()["capture"].clone(),footer()["game"].clone()).unwrap();
    let report=read_report(Cursor::new(writer.into_inner()),65536).unwrap();
    assert_eq!(report["schemaId"],"spire-report-v1");
    assert_eq!(report["run"]["seed"]["value"],"18446744073709551616");
    assert_eq!(report["frames"][1]["previousFrameId"],"0");
    assert_eq!(report["catalog"]["card/strike"]["name"],"Strike");
    assert_eq!(report["capture"]["status"],"complete");
    assert!(report["game"]["outcome"].is_null());
}

#[test]
fn protocol_partial() {
    // A torn tail must not promote a capture to success or discard complete prior records.
    let bytes=complete();
    for tail in [b"{\"record\":".as_slice(), &serde_json::to_vec(&footer()).unwrap()] {
        let mut partial=lines(&[header(),json!({"record":"frame","frame":frame("0",None)})]);
        partial.extend(tail);
        let report=read_report(Cursor::new(partial),65536).unwrap();
        assert_eq!(report["frames"].as_array().unwrap().len(),1);
        assert_eq!(report["capture"]["status"],"interrupted");
        assert!(report["game"]["boundary"].is_null());
        assert!(!report["warnings"].as_array().unwrap().is_empty());
    }
    for partial in [lines(&[header()]),bytes[..bytes.len()-1].to_vec()] {
        assert_eq!(read_report(Cursor::new(partial),65536).unwrap()["capture"]["status"],"interrupted");
    }
    let mut interior=lines(&[header()]); interior.extend(b"{broken}\n"); interior.extend(lines(&[footer()]));
    assert!(read_report(Cursor::new(interior),65536).is_err());
    assert!(read_report(Cursor::new(b"{broken}\n"),65536).is_err());
    assert!(read_report(Cursor::new(b"{\"record\":"),65536).is_err());
}

#[test]
fn protocol_invalid() {
    // Both recording and import reject forged histories, numeric identifiers and incompatible schemas.
    for bad in [frame("0",Some("0")),frame("1",None),frame("00",None)] {
        let mut writer=ReportWriter::new(Vec::new(),header()).unwrap();
        assert!(writer.frame(bad.clone()).is_err());
        assert!(read_report(Cursor::new(lines(&[header(),json!({"record":"frame","frame":bad})])),65536).is_err());
    }
    let mut writer=ReportWriter::new(Vec::new(),header()).unwrap(); writer.frame(frame("0",None)).unwrap();
    for bad in [frame("0",None),frame("2",Some("0")),frame("1",Some("9"))] {
        assert!(writer.frame(bad).is_err());
    }
    for (pointer,value) in [("/schemaVersion",json!(2)),("/schemaId",json!("trace-v1")),
        ("/run/seed/value",json!(7)),("/source/kind",json!("snapshot"))] {
        let mut bad=header(); *bad.pointer_mut(pointer).unwrap()=value;
        assert!(ReportWriter::new(Vec::new(),bad.clone()).is_err());
        assert!(read_report(Cursor::new(lines(&[bad])),65536).is_err());
    }
    for (pointer,value) in [("/id",json!(0)),("/combatId",json!(9007199254740993u64)),
        ("/action",json!({"type":"play","targets":[{"ref":4}]}))] {
        let mut bad=frame("0",None); *bad.pointer_mut(pointer).unwrap()=value;
        assert!(ReportWriter::new(Vec::new(),header()).unwrap().frame(bad).is_err());
    }
    let mut writer=ReportWriter::new(Vec::new(),header()).unwrap();
    writer.catalog(json!({"card/bash":{"name":"Bash"}})).unwrap();
    assert!(writer.catalog(json!({"card/bash":{"name":"Not Bash"}})).is_err());
    let bad_catalog=lines(&[header(),json!({"record":"catalog","entries":{"card/bash":{"name":"Bash"}}}),
        json!({"record":"catalog","entries":{"card/bash":{"name":"Not Bash"}}})]);
    assert!(read_report(Cursor::new(bad_catalog),65536).is_err());
    assert!(read_report(Cursor::new([complete(),lines(&[footer()])].concat()),65536).is_err());
    assert!(read_report(Cursor::new(complete()),10).is_err());
    assert!(read_report(Cursor::new(Vec::<u8>::new()),65536).is_err());
}

struct SwitchSink { bytes:Vec<u8>, fail:Rc<Cell<bool>>, flushes:Rc<Cell<usize>> }
impl Write for SwitchSink {
    fn write(&mut self,bytes:&[u8])->io::Result<usize> {
        if self.fail.get() { return Err(io::Error::other("disk unavailable")); }
        self.bytes.extend(bytes); Ok(bytes.len())
    }
    fn flush(&mut self)->io::Result<()> {
        self.flushes.set(self.flushes.get()+1);
        if self.fail.get(){ Err(io::Error::other("flush unavailable")) }else{Ok(())}
    }
}

#[test]
fn protocol_write_failure() {
    // Once I/O fails, recovering the sink cannot silently complete a broken trace.
    let fail=Rc::new(Cell::new(false)); let flushes=Rc::new(Cell::new(0));
    let mut writer=ReportWriter::new(SwitchSink{bytes:vec![],fail:fail.clone(),flushes:flushes.clone()},header()).unwrap();
    assert_eq!(flushes.get(),1);
    writer.frame(frame("0",None)).unwrap(); assert_eq!(flushes.get(),2);
    writer.checkpoint().unwrap(); assert_eq!(flushes.get(),3);
    fail.set(true); assert!(writer.checkpoint().is_err());
    fail.set(false);
    assert!(writer.frame(frame("1",Some("0"))).is_err());
    assert!(writer.finish(footer()["capture"].clone(),footer()["game"].clone()).is_err());
    let report=read_report(Cursor::new(writer.into_inner().bytes),65536).unwrap();
    assert_eq!(report["capture"]["status"],"interrupted");
}

#[test]
fn protocol_prefix_is_not_victory() {
    let report=read_report(Cursor::new(complete()),65536).unwrap();
    assert_eq!(report["capture"]["status"],"complete");
    assert_eq!(report["game"]["boundary"],"act2_prefix_complete");
    assert!(report["game"]["outcome"].is_null());
    let mut writer=ReportWriter::new(Vec::new(),header()).unwrap();
    writer.finish(footer()["capture"].clone(),footer()["game"].clone()).unwrap();
    assert!(writer.checkpoint().is_err());
    assert!(writer.frame(frame("0",None)).is_err());
    assert!(writer.catalog(json!({})).is_err());
    assert!(writer.finish(footer()["capture"].clone(),footer()["game"].clone()).is_err());
}

#[test]
fn protocol_rejects_missing_state_fields_and_dangling_parent_actions() {
    // Shape errors and nonexistent continuation parents must not become apparently valid slides.
    for (pointer,value) in [("/state/screen",json!(null)),("/state/player",json!([])),
        ("/effects",json!([{"sequence":"0","kind":"damage"}])),
        ("/action",json!({"type":"selection","parentActionId":"25"}))] {
        let mut bad=frame("1",Some("0")); *bad.pointer_mut(pointer).unwrap()=value;
        let mut writer=ReportWriter::new(Vec::new(),header()).unwrap();
        writer.frame(frame("0",None)).unwrap();
        assert!(writer.frame(bad.clone()).is_err(),"accepted malformed {pointer}");
        assert!(read_report(Cursor::new(lines(&[header(),
            json!({"record":"frame","frame":frame("0",None)}),json!({"record":"frame","frame":bad})])),65536).is_err());
    }
    let mut bad=frame("0",None); bad["state"].as_object_mut().unwrap().remove("inventory");
    assert!(ReportWriter::new(Vec::new(),header()).unwrap().frame(bad).is_err());
}

struct LimitedSink { bytes:Vec<u8>, remaining:usize }
impl Write for LimitedSink {
    fn write(&mut self,bytes:&[u8])->io::Result<usize> {
        if self.remaining==0 {return Err(io::Error::other("full"));}
        let count=bytes.len().min(self.remaining); self.bytes.extend(&bytes[..count]);
        self.remaining-=count; Ok(count)
    }
    fn flush(&mut self)->io::Result<()> {Ok(())}
}

#[test]
fn protocol_partial_write_never_appends_a_success_footer() {
    let prefix=lines(&[header()]);
    let mut writer=ReportWriter::new(LimitedSink{bytes:vec![],remaining:prefix.len()+20},header()).unwrap();
    assert!(writer.frame(frame("0",None)).is_err());
    assert!(writer.finish(footer()["capture"].clone(),footer()["game"].clone()).is_err());
    let report=read_report(Cursor::new(writer.into_inner().bytes),65536).unwrap();
    assert!(report["frames"].as_array().unwrap().is_empty());
    assert_eq!(report["capture"]["status"],"interrupted");
    assert!(ReportWriter::new(LimitedSink{bytes:vec![],remaining:3},header()).is_err());
}
