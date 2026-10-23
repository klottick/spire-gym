mod common;
use serde_json::{Value,json};
use spire_report::{export_html,json_for_html};
use std::io::Cursor;

#[test]
fn html_inert_text() {
    let hostile="</script><script>globalThis.pwned=1</script> <b>& @@VIEWER_JS@@ \u{2028}\u{2029}";
    let value=json!({"name":hostile});
    let inert=json_for_html(&value).unwrap();
    assert_eq!(serde_json::from_str::<Value>(&inert).unwrap(),value);
    for active in ["<",">","&","\u{2028}","\u{2029}"] {assert!(!inert.contains(active));}
    let mut html=Vec::new(); export_html(Cursor::new(common::trace(hostile)),None,&mut html).unwrap();
    let html=String::from_utf8(html).unwrap();
    let data=html.split("<script id=\"spire-report-data\" type=\"application/json\">").nth(1).unwrap().split("</script>").next().unwrap();
    let report:Value=serde_json::from_str(data).unwrap();
    assert_eq!(report["frames"][0]["state"]["screen"]["name"],hostile);
    assert!(!html.contains("<script>globalThis.pwned"));
    assert!(html.contains("connect-src 'none'"));
    assert_eq!(report["assets"]["blobs"],json!({}));
}

#[test]
fn html_write_failure() {
    // Export failure must not modify the source trace or convert it to another outcome.
    let trace=common::trace("actual data");let original=trace.clone();
    assert!(export_html(Cursor::new(&trace),None,std::io::Cursor::new(&mut [0u8;20][..])).is_err());
    assert_eq!(trace,original);
    let mut out=Vec::new();assert!(export_html(Cursor::new(b"bad\n"),None,&mut out).is_err());
    assert!(out.is_empty(),"must validate before emitting an HTML prefix");
}

#[test]
fn html_embeds_only_referenced_available_artwork() {
    let scratch=common::Scratch::new();std::fs::write(scratch.file("one.png"),common::PNG).unwrap();
    let manifest=common::manifest(&scratch,json!({"card/bash":{"path":"one.png","source":"fixture"},
        "unused":{"path":"one.png","source":null}}));
    let mut records:Vec<Value>=common::trace("Bash").split(|b|*b==b'\n').filter(|b|!b.is_empty())
        .map(|line|serde_json::from_slice(line).unwrap()).collect();
    records[1]["frame"]["state"]["screen"]["options"]=json!([
        {"key":"taken","label":"Bash","status":"offered","data":{"assetKey":"card/bash"}},
        {"key":"missing","label":"Missing art","status":"offered","data":{"assetKey":"no-entry"}}]);
    let mut trace=Vec::new();for record in records{trace.extend(serde_json::to_vec(&record).unwrap());trace.push(b'\n');}
    let mut html=Vec::new();export_html(Cursor::new(trace),Some(&manifest),&mut html).unwrap();
    let html=String::from_utf8(html).unwrap();
    let data=html.split("<script id=\"spire-report-data\" type=\"application/json\">").nth(1).unwrap().split("</script>").next().unwrap();
    let report:Value=serde_json::from_str(data).unwrap();
    assert_eq!(report["assets"]["byKey"].as_object().unwrap().len(),1);
    assert_eq!(report["assets"]["blobs"].as_object().unwrap().len(),1);
    assert_eq!(report["warnings"].as_array().unwrap().len(),1);
    assert!(report["assets"]["byKey"].get("unused").is_none());
    assert!(!html.contains("src=\"http"));
}
