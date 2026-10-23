#![allow(dead_code)]
use serde_json::{Value,json};
use std::{fs,path::PathBuf,sync::atomic::{AtomicU64,Ordering}};

static NEXT:AtomicU64=AtomicU64::new(0);
pub struct Scratch(pub PathBuf);
impl Scratch {
    pub fn new()->Self {
        loop {
            let path=PathBuf::from("/tmp/opencode").join(format!("spire-report-test-{}-{}",std::process::id(),NEXT.fetch_add(1,Ordering::Relaxed)));
            match fs::create_dir(&path) {Ok(())=>return Self(path),Err(e) if e.kind()==std::io::ErrorKind::AlreadyExists=>continue,Err(e)=>panic!("{e}")}
        }
    }
    pub fn file(&self,name:&str)->PathBuf {self.0.join(name)}
}
impl Drop for Scratch {fn drop(&mut self){let _=fs::remove_dir_all(&self.0);}}

pub fn trace(name:&str)->Vec<u8> {
    let records=[json!({"record":"header","schemaId":"spire-report-v1","schemaVersion":1,
        "source":{"kind":"authored_fixture","profile":"security-test","producerRevision":null,"contentId":null,"evidenceGrade":"fixture"},
        "run":{"character":"ironclad","ascension":0,"seed":{"encoding":"decimal","value":"7"},"caller":null}}),
        json!({"record":"frame","frame":{"id":"0","previousFrameId":null,"kind":"initial",
            "act":null,"floor":null,"combatId":null,"turn":null,"action":null,"beforeScreen":null,
            "state":{"screen":{"kind":"event","name":name,"options":[],"selection":null},
                "player":{"hp":10,"maxHp":10},"inventory":null,"map":null,"combat":null,"terminal":null},
            "effects":[],"combatFinal":null,"stateValidity":"settled"}}),
        json!({"record":"footer","capture":{"status":"complete","reason":null},"game":{"outcome":null,"boundary":null}})];
    let mut bytes=Vec::new(); for record in records {bytes.extend(serde_json::to_vec(&record).unwrap());bytes.push(b'\n');} bytes
}

// A generated 1x1 transparent PNG, not game artwork.
pub const PNG:&[u8]=&[137,80,78,71,13,10,26,10,0,0,0,13,73,72,68,82,0,0,0,1,0,0,0,1,8,6,0,0,0,31,21,196,137,0,0,0,11,73,68,65,84,120,156,99,96,0,2,0,0,5,0,1,165,246,69,64,0,0,0,0,73,69,78,68,174,66,96,130];
pub fn manifest(scratch:&Scratch,assets:Value)->PathBuf {
    let path=scratch.file("assets.json");
    fs::write(&path,serde_json::to_vec(&json!({"schemaVersion":1,"assets":assets,"attribution":["Mega Crit owns original game artwork"]})).unwrap()).unwrap();path
}
