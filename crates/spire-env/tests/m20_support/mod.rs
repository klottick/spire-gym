#![allow(dead_code)]
use spire_core::{combat::rng::Partition, content::*, run::*};
use spire_env::*;
pub fn config(act1: RunAct1BossTarget, act2: RunAct2BossTarget, act3:RunAct3BossTarget) -> RunSessionConfig {
    RunSessionConfig { character: Character::Defect, ascension: 0, partition: Partition::Test,
        boss_history: RunBossHistory::M20DefectHeart { act1, act2, act3 },
        capacities: RunCapacities::m10_default(), record_replay: true }
}
pub fn default_config() -> RunSessionConfig { config(RunAct1BossTarget::Guardian, RunAct2BossTarget::Champ, RunAct3BossTarget::AwakenedOne) }
// Short supplied transport diagnostic, independent of the held six outcome runs.
pub fn diagnostic_run() -> Run {
    let mut root=[0;32];root[..8].copy_from_slice(&7u64.to_le_bytes());
    let mut creation=CreationProfile::experienced();creation.neow_eligible=false;
    creation.seen_bosses[0]=vec!["Hexaghost","SlimeBoss"];
    creation.seen_bosses[1]=vec!["Automaton","Collector"];
    creation.seen_bosses[2]=vec!["TimeEater","DonuAndDeca"];
    Run::new_room_scenario(RunConfig{character:Character::Defect,ascension:0},creation,
        RunRng::new(root,Partition::Test,0),RoomScenario::M20DefectHeart).unwrap()
}
pub fn combat_diagnostic() -> (PublicDecision, Option<map::PublicMap>) {
    let mut run=diagnostic_run();let d=run.decision().unwrap();run.step(&d.candidates[0].action).unwrap();
    let d=run.decision().unwrap();let action=d.candidates.iter().find(|c|matches!(c.action,GameAction::Move(_))).unwrap().action.clone();
    run.step(&action).unwrap();(run.decision().unwrap(),run.public_map())
}

/// Supplied actual encounter diagnostic. The returned public decision is a
/// projection harness, never a normal Run traversal or a session checkpoint.
pub fn supplied_combat(encounter: EncounterId, cards: Vec<spire_core::combat::LiveCard>) -> spire_core::combat::Combat {
    supplied_combat_with_opening(encounter,cards.clone(),cards.len().min(10))
}
pub fn supplied_combat_with_opening(encounter:EncounterId,cards:Vec<spire_core::combat::LiveCard>,opening_draw:usize)->spire_core::combat::Combat{
    use spire_core::combat::{enemies::{self,SetupContext},PlayerState};
    let context=SetupContext {
        player:PlayerState {hp:75,max_hp:75,block:0,energy:3,max_energy:3,draw_per_turn:5,powers:vec![]},
        player_powers:vec![],character:Character::Defect,opening_draw,cards,encounter_visit:0,
    };
    let mut rng=RunRng::new([37;32],Partition::Test,0);
    let setup=enemies::setup(encounter,if matches!(encounter,EncounterId::ShieldAndSpear|EncounterId::CorruptHeart){4}else if ACT3.contains(&encounter){3}else{2},0,&mut rng,context).unwrap();
    let mut items=Some(spire_core::items::CombatItemContext::new(vec![],vec![None;3]));
    spire_core::combat::Combat::enter_with_item_context(setup,&mut items).unwrap()
}
pub fn project_combat(combat:&spire_core::combat::Combat, objects:Vec<PublicObject>) -> PublicDecision {
    let mut d=diagnostic_run().decision().unwrap();
    d.act=3;d.floor=35;d.objects=objects;d.combat=Some(combat.public_full().unwrap());
    d.room.as_mut().unwrap().kind=RoomViewKind::Combat;d.room.as_mut().unwrap().choices.clear();
    d.offers.clear();d.candidates=combat.legal_actions_full().unwrap().into_iter().enumerate().map(|(i,action)|Candidate {
        id:ChoiceId(i as u64+1),action:GameAction::Combat(action),label:"supplied diagnostic",neow_offer:None,object:None,
    }).collect();d
}

const ACT3: &[EncounterId] = &[EncounterId::ThreeDarklings,EncounterId::AwakenedOne,EncounterId::TimeEater,EncounterId::DonuAndDeca];
pub fn diagnostic_run_for_config(cfg:&RunSessionConfig,seed:u64)->Run {
    let RunBossHistory::M20DefectHeart{act1,act2,act3}=cfg.boss_history else{panic!("M20 config required")};
    let mut root=[0;32];root[..8].copy_from_slice(&seed.to_le_bytes());
    let mut creation=CreationProfile::experienced();creation.neow_eligible=false;
    creation.seen_bosses[0]=match act1 {
        RunAct1BossTarget::Guardian=>vec!["Hexaghost","SlimeBoss"],RunAct1BossTarget::Hexaghost=>vec!["Guardian","SlimeBoss"],RunAct1BossTarget::SlimeBoss=>vec!["Guardian","Hexaghost"]};
    creation.seen_bosses[1]=match act2 {
        RunAct2BossTarget::Champ=>vec!["Automaton","Collector"],RunAct2BossTarget::BronzeAutomaton=>vec!["Champ","Collector"],RunAct2BossTarget::Collector=>vec!["Champ","Automaton"]};
    creation.seen_bosses[2]=match act3 {
        RunAct3BossTarget::AwakenedOne=>vec!["TimeEater","DonuAndDeca"],RunAct3BossTarget::TimeEater=>vec!["AwakenedOne","DonuAndDeca"],RunAct3BossTarget::DonuAndDeca=>vec!["AwakenedOne","TimeEater"]};
    Run::new_room_scenario(RunConfig{character:Character::Defect,ascension:0},creation,RunRng::new(root,cfg.partition,0),RoomScenario::M20DefectHeart).unwrap()
}
