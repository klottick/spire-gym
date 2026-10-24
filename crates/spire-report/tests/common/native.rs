#![allow(dead_code)]
use spire_core::{run::*,combat::*,content::{Character,CardId,RelicId,PotionId,EncounterId}};
#[path="../../../spire-core/examples/m4_support/mod.rs"]
pub mod m4;
pub fn header()->serde_json::Value {serde_json::json!({"record":"header","schemaId":"spire-report-v1","schemaVersion":1,
    "source":{"kind":"run_prefix","profile":"native test profile","producerRevision":"5b1cd19+viewer","contentId":CONTENT_ID,"evidenceGrade":"source-grounded"},
    "run":{"character":"ironclad","ascension":0,"seed":{"encoding":"native-u64-le","value":"7"},"caller":null}})}
pub fn lethal()->Run {
    let mut run=Run::from_authored_combat(RunConfig{character:Character::Ironclad,ascension:0},AuthoredRunStart {
        hp:40,max_hp:80,gold:99,deck:vec![spire_core::run::CardInstance{instance:1,id:CardId::Strike,upgrades:0}],
        relics:vec![RelicId::BurningBlood],potions:vec![Some(PotionId::FirePotion);3],relic_counters:Default::default()},
        RunRng::new([7;32],rng::Partition::Test,0)).unwrap();
    run.enter_combat(EncounterId::JawWorm).unwrap();run
}
pub fn route()->Vec<(GameAction,RunTransition,PublicDecision)> {
    let mut source=m4::run(7);let mut caller=m4::Caller::new(512);
    m4::rollout(&mut source,&mut caller,m4::Scenario::Route).unwrap();caller.trace
}
