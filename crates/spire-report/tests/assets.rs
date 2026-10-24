mod common;
use serde_json::json;
use spire_report::{embed_referenced,load_assets};
use std::{collections::BTreeSet,fs};

#[test]
fn assets_reference_dedup() {
    let scratch=common::Scratch::new();fs::write(scratch.file("one.png"),common::PNG).unwrap();
    let path=common::manifest(&scratch,json!({"card/strike":{"path":"one.png","source":"local original import"},
        "card/bash":{"path":"one.png","source":"local original import"},"card/unused":{"path":"missing.png","source":null}}));
    let assets=embed_referenced(&load_assets(&path).unwrap(),&BTreeSet::from(["card/strike".into(),"card/bash".into()])).unwrap();
    assert_eq!(assets["blobs"].as_object().unwrap().len(),1);
    assert_eq!(assets["byKey"]["card/strike"],assets["byKey"]["card/bash"]);
    assert!(assets["byKey"].get("card/unused").is_none());
    assert!(assets["blobs"].as_object().unwrap().values().next().unwrap().as_str().unwrap().starts_with("data:image/png;base64,iVBORw0KGgo"));
    assert!(assets["attribution"].as_array().unwrap().iter().any(|v|v.as_str().unwrap().contains("Mega Crit")));
}

#[test]
fn assets_paths_and_signatures() {
    let scratch=common::Scratch::new();let outside=common::Scratch::new();
    fs::write(outside.file("private.png"),common::PNG).unwrap();
    std::os::unix::fs::symlink(&outside.0,scratch.file("escape")).unwrap();
    for path in ["../outside.png","/etc/passwd","escape/private.png","escape/missing.png"] {
        let manifest=common::manifest(&scratch,json!({"bad":{"path":path,"source":null}}));
        assert!(load_assets(&manifest).is_err(),"accepted unsafe {path}");
    }
    for (name,bytes) in [("active.svg",b"<svg onload=\"alert(1)\"/>".as_slice()),("broken.png",b"\x89PNG\r\n\x1a\n")]{
        fs::write(scratch.file(name),bytes).unwrap();
        let path=common::manifest(&scratch,json!({"bad":{"path":name,"source":null}}));
        assert!(embed_referenced(&load_assets(&path).unwrap(),&BTreeSet::from(["bad".into()])).is_err());
    }
    let path=common::manifest(&scratch,json!({"missing":{"path":"absent.png","source":null}}));
    let assets=embed_referenced(&load_assets(&path).unwrap(),&BTreeSet::from(["missing".into(),"unknown".into()])).unwrap();
    assert!(assets["blobs"].as_object().unwrap().is_empty());
    assert_eq!(assets["warnings"].as_array().unwrap().len(),2);
    fs::create_dir(scratch.file("not-an-image")).unwrap();
    let path=common::manifest(&scratch,json!({"bad":{"path":"not-an-image","source":null}}));
    assert!(load_assets(&path).is_err(),"reject nonregular artwork before opening it");
}

#[test]
fn assets_accept_local_jpeg_and_webp_headers() {
    // Small format-header fixtures, not a claim of browser decoding or original artwork.
    let scratch=common::Scratch::new();
    let jpeg=b"\xff\xd8\xff\xe0\x00\x10JFIF\x00\x01\x01\x00\x00\x01\x00\x01\x00\x00\xff\xd9";
    let webp=b"RIFF\x16\x00\x00\x00WEBPVP8L\x09\x00\x00\x00\x2f\x00\x00\x00\x00\x00\x00\x00\x00\x00";
    for (name,bytes,mime) in [("a.jpg",jpeg.as_slice(),"image/jpeg"),("a.webp",webp.as_slice(),"image/webp")] {
        fs::write(scratch.file(name),bytes).unwrap();
        let path=common::manifest(&scratch,json!({"art":{"path":name,"source":null}}));
        let result=embed_referenced(&load_assets(&path).unwrap(),&BTreeSet::from(["art".into()])).unwrap();
        assert!(result["blobs"].as_object().unwrap().values().next().unwrap().as_str().unwrap().starts_with(&format!("data:{mime};base64,")));
    }
}
