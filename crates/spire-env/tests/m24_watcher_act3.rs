//! M24 finite Watcher A0 Acts1–3 native profile: descriptor, encoder, session, replay, batch and coexistence.
//! Provenance labels: "normal" = canonical M24 initializer driven only by retained recorded actions (the exact M23/M22 Act1
//! prefix); "supplied" = a default-off m24-diagnostics supplied start, then real owners; "constructed" = an owner checkpoint
//! built directly; "relabelled" = an actual M21-owned Watcher combat whose public view is placed at an Act3 floor to drive
//! the pure encoder. Constructed-stage session/replay/batch checks live in the owning crate modules (run_session.rs,
//! run_batch.rs) because the diagnostic session hooks are cfg(test) only.
use spire_env::*;
use spire_core::{combat::{rng::Partition,CombatAction,Stance,TargetId},content::*,run::{*,diagnostics::{m24_diagnostic,m24_played_fixture,m24_supplied_boss,M24Diagnostic,M24DiagnosticBoss,M24PlayedFixture}}};
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use std::collections::BTreeSet;

// Retained pre-change bytes: every predecessor descriptor on core snapshot v3 (h1 baseline probe, target/h1/logs/00-baseline-probe.log).
const FROZEN:&[(&str,&str)]=&[
 ("m9","0893bce86e4b3abf11aa384c501503a91ae37f688ec605ab40b814ad365cb999"),("m10","7664874a7a483d1f2e0439ea7b319d1a4be4b9d276d0cb0f908654feda9393db"),
 ("m12","366c572af8141146995e849acc737a33484623790e2b3d661e290fadfb6c9fac"),("m13","3f1e21d50106b8ca71d7cd67a2de7bfa4074455dbbb459629d8e2ac24cf39cf8"),
 ("m14","50a685721e5db56ed7a5332160f3747087f96ce738da020415b1427159377aee"),("m15","254fb75b404fc558070468c6397324e68f962f23b33aca819b54fecf142ea11a"),
 ("m17","2176f835eca1384b2070f3d7d001158ef1fead23be5d6d7eea138b57f6bff999"),("m18","e097a8202f7cf7433f0ce57a39807bb74d370fba75a9aa84af65d726269564bb"),
 ("m19","149092e11432e182920b5f07e0728dac45e880296815d9b84fba9b6180c2817b"),("m20","8c03211b4a3c589cfb55ba1078ddd447a444f3f53d8e9bfdab9ab85ab80e512d"),
 ("m22","5d1c77658141b2dfefbe95e17b85899ad6b77fb370e402bcbbe771146821e087"),("m23","5d5524ff7f60575dea528d42808f7a0a5e67c41bfd0e3b36f7f83b663c9f9d7e"),
 ("policy","30905db1b7a520482d3ea8aa2e63ddb7f78b7c081de89d87268151f29cf4c7f8"),
 ("combat_v1","02009686282c592f971eece3d5853eb54f2a482dde9dae1d0cf380ecdaf8fd78"),("combat_v2","02433813e1297244060d9852f342c6eb4c9c4d62f3dde9763ff82191867eceec"),
 ("m21","e7e49e88d50c947fde064a764153a5846ba3dc6c8298116f03e8126383b51e34"),("m16","2b53f59fac1899e231300914b0b88aabc4e26add3492584de479b60e5fafce0a"),
];
// Oracles from the A1 source sweep of the 16 Act3 setup/AI owners (path:line table in the delivery evidence), not the implementation.
const ACT3_INTENTS:&[&str]=&["orb_claw","orb_laser","repulsor_attack","repulsor_daze","exploder_attack","exploder_explode","spiker_attack","spiker_buff","growth_quick","growth_constrict","growth_smash","transient_attack","maw_roar","maw_nom","maw_drool","maw_slam","darkling_harden","darkling_nip","darkling_chomp","darkling_regrowing","darkling_reincarnate","writhing_multi","writhing_block","writhing_debuff","writhing_parasite","writhing_big","head_count","head_glare","head_time","nemesis_tri","nemesis_burn","nemesis_scythe","repto_spawn","repto_snake","repto_bite","dagger_wound","dagger_explode","incantation","attack","awa_slash","awa_echo","awa_soul","awa_sludge","awa_tackle","awa_rebirth","time_haste","time_reverb","time_slam","time_ripple","deca_beam","deca_square","donu_beam","donu_circle","chomp","thrash","bellow","sphere_activate","sphere_debuff","sphere_slam","sphere_harden"];
const ACT3_POWERS:&[&str]=&["artifact","barricade","constricted","curiosity","draw_reduction","explosive","fading","frail","intangible","life_link","malleable","metallicize","minion","reactive","regenerate","regeneration","ritual","shackled","shifting","slow","strength","strength_up","thorns","time_warp","unawakened","vulnerable","weak"];
const ACT3_ENCOUNTERS:[EncounterId;16]=[EncounterId::OrbWalker,EncounterId::ThreeShapes,EncounterId::SpireGrowth,EncounterId::Transient,EncounterId::FourShapes,EncounterId::Maw,EncounterId::SphereAndTwoShapes,EncounterId::JawWormHorde,EncounterId::ThreeDarklings,EncounterId::WrithingMass,EncounterId::GiantHead,EncounterId::Nemesis,EncounterId::Reptomancer,EncounterId::AwakenedOne,EncounterId::TimeEater,EncounterId::DonuAndDeca];
const NEW_ACT3_ENEMIES:[&str;17]=["orb_walker","repulsor","exploder","spiker","darkling","spire_growth","transient","maw","writhing_mass","giant_head","nemesis","reptomancer","dagger","awakened_one","time_eater","deca","donu"];

fn hex(b:[u8;32])->String{b.iter().map(|b|format!("{b:02x}")).collect()}
fn caps()->RunCapacities{RunCapacities::m10_default()}
fn cfg(act1:RunAct1BossTarget,act2:RunAct2BossTarget,act3:RunAct3BossTarget)->RunSessionConfig{RunSessionConfig{character:Character::Watcher,ascension:0,partition:Partition::Test,boss_history:RunBossHistory::M24WatcherAct3{act1,act2,act3},capacities:caps(),record_replay:true}}
fn config()->RunSessionConfig{cfg(RunAct1BossTarget::Guardian,RunAct2BossTarget::Champ,RunAct3BossTarget::AwakenedOne)}
fn latest(s:&RunSession)->PublicDecision{s.reproduction().unwrap().latest_decision.unwrap().unwrap()}
fn index(s:&RunSession,a:&GameAction)->RunActionIndex{RunActionIndex(latest(s).candidates.iter().position(|c|&c.action==a).expect("semantic action is currently offered") as i64)}
fn act(s:&mut RunSession,a:GameAction)->RunStepResult{let i=index(s,&a);let t=s.decision().unwrap().token;s.step(t,i).unwrap()}
fn first_move(d:&PublicDecision)->GameAction{d.candidates.iter().find(|c|matches!(c.action,GameAction::Move(_))).expect("map move").action.clone()}
/// Fixed public chooser: Neow/room row 0, first Move, EndTurn whenever offered. No strategy or search.
fn passive(d:&PublicDecision)->GameAction{d.candidates.iter().find(|c|c.action==GameAction::Combat(CombatAction::EndTurn)).unwrap_or(&d.candidates[0]).action.clone()}
fn vocab_id(d:&Value,key:&str,name:&str)->i64{d["vocabularies"][key].as_array().unwrap().iter().position(|v|v==name).unwrap_or_else(||panic!("{key} {name} absent")) as i64}
fn group<'a>(o:&'a RunObservation,key:&str)->&'a RunFeatureGroup{o.group(key).unwrap()}
fn cell(g:&RunFeatureGroup,row:usize,col:usize)->(i64,i8){let cols=*g.shape.last().unwrap();(g.values[row*cols+col],g.known[row*cols+col])}
fn col(spec:&Value,key:&str,field:&str)->usize{spec["tensors"].as_array().unwrap().iter().find(|t|t["key"]==key).unwrap()["fields"].as_array().unwrap().iter().position(|f|f==field).unwrap()}
fn occupied(g:&RunFeatureGroup)->usize{g.occupied.as_ref().unwrap().iter().filter(|x|**x==1).count()}
fn spec()->Value{m24_run_observation_spec(&caps()).unwrap().descriptor()}
/// Actual M21-owned Watcher combat setup for an authored encounter. Relabelled only to drive the pure M24 encoder.
fn fixture(encounter:EncounterId,start:M21WatcherCombatStart)->Run{let mut root=[0;32];root[..8].copy_from_slice(&7u64.to_le_bytes());let mut r=Run::prepare_m21_watcher_combat(start,encounter,RunRng::new(root,Partition::Test,0)).unwrap();r.enter_training_combat(encounter).unwrap();r}
fn supplied(cards:&[CardId])->M21WatcherCombatStart{M21WatcherCombatStart::Supplied(AuthoredRunStart{hp:72,max_hp:72,gold:99,deck:cards.iter().enumerate().map(|(i,id)|CardInstance{instance:31+i as u64,id:*id,upgrades:0,misc:0}).collect(),relics:vec![RelicId::PureWater],potions:vec![None;3],relic_counters:Default::default()})}
/// Constructed (labelled) placement of an actual standalone combat view at an Act3 floor; gameplay values untouched.
fn relabel(r:&Run)->PublicDecision{let mut d=r.decision().unwrap();d.content_profile=Some(RoomScenario::M24WatcherAct3);d.act=3;d.floor=40;d}
fn play(r:&mut Run,kind:CardId)->bool{let d=r.decision().unwrap();let c=d.combat.as_ref().unwrap();let Some(a)=d.candidates.iter().find(|row|matches!(row.action,GameAction::Combat(CombatAction::PlayCard{instance,..}) if c.hand.iter().any(|card|card.instance.0==instance&&card.id==kind))).map(|c|c.action.clone()) else{return false};r.step(&a).unwrap();true}
fn end_turn(r:&mut Run)->bool{let d=r.decision().unwrap();if d.terminal.is_some()||!d.candidates.iter().any(|c|c.action==GameAction::Combat(CombatAction::EndTurn)){return false}r.step(&GameAction::Combat(CombatAction::EndTurn)).unwrap();true}
/// Fixed public combat driver (no strategy): stance row 0, Confirm an open selection (else select its first card),
/// else the first offered PlayCard row, else EndTurn. Returns false outside combat.
fn drive_once(r:&mut Run)->bool{
 let d=r.decision().unwrap();let Some(c)=d.combat.as_ref() else{return false};if d.terminal.is_some(){return false}
 let is=|f:&dyn Fn(&CombatAction)->bool|d.candidates.iter().find(|x|matches!(&x.action,GameAction::Combat(a) if f(a))).map(|x|x.action.clone());
 let pick=if c.stance_choice.is_some(){is(&|a|matches!(a,CombatAction::ChooseStance{..}))}else if c.selection.is_some(){is(&|a|matches!(a,CombatAction::ConfirmSelection)).or_else(||is(&|a|matches!(a,CombatAction::SelectCard{..})))}else{is(&|a|matches!(a,CombatAction::PlayCard{..})).or_else(||is(&|a|matches!(a,CombatAction::EndTurn)))};
 match pick{Some(a)=>{r.step(&a).unwrap();true},None=>false}
}
fn stance_code(s:Stance)->i64{match s{Stance::Neutral=>0,Stance::Wrath=>1,Stance::Calm=>2,Stance::Divinity=>3}}
/// Exact public projection of one combat decision: candidate i is action row i (exact mask), enemy row i is enemy i
/// (identity, intent, half-dead, slot), and every installed public power amount (player owner1, enemy owner5) in order.
fn check_combat(o:&RunObservation,d:&PublicDecision,spec:&Value,powers:&mut BTreeSet<String>,enemies:&mut BTreeSet<String>,intents:&mut BTreeSet<String>){
 let n=d.candidates.len();assert_eq!(o.action_mask.iter().filter(|m|**m==1).count(),n);assert!(o.action_mask[..n].iter().all(|m|*m==1));assert_eq!(occupied(group(o,"actions")),n);
 let Some(c)=d.combat.as_ref() else{return};let e=group(o,"enemies");assert_eq!(occupied(e),c.enemies.len());
 for (row,en) in c.enemies.iter().enumerate(){
  assert_eq!(cell(e,row,0).0,vocab_id(spec,"enemy",en.enemy_id.as_str()));assert_eq!(cell(e,row,col(spec,"enemies","intent_kind")).0,vocab_id(spec,"intent",en.intent.as_str()));
  assert_eq!(cell(e,row,col(spec,"enemies","half_dead")),(i64::from(en.half_dead),1));assert_eq!(cell(e,row,col(spec,"enemies","logical_slot")).0,i64::from(en.logical_slot));
  enemies.insert(en.enemy_id.as_str().to_owned());intents.insert(en.intent.as_str().to_owned());
 }
 let p=group(o,"powers");let mut row=0;
 for (owner,idx,list) in std::iter::once((1,0,&c.player.powers)).chain(c.enemies.iter().enumerate().map(|(i,e)|(5,i,&e.powers))){for x in list{
  assert_eq!([cell(p,row,0).0,cell(p,row,1).0,cell(p,row,2).0,cell(p,row,3).0],[owner,idx as i64,vocab_id(spec,"power",x.id.0),i64::from(x.amount)],"{}",x.id.0);powers.insert(x.id.0.to_owned());row+=1;}}
 assert_eq!(occupied(p),row,"no truncated or invented power rows");
}
fn choice_name(k:&RoomChoiceKind)->&'static str{match k{RoomChoiceKind::Proceed=>"proceed",RoomChoiceKind::Act3Continue{..}=>"act3_continue",RoomChoiceKind::FallingLose{..}=>"falling_lose",RoomChoiceKind::TombGold=>"tomb_gold",RoomChoiceKind::TombOfferGold=>"tomb_offer_gold",RoomChoiceKind::Leave=>"leave",RoomChoiceKind::OpenChest=>"open_chest",RoomChoiceKind::Relic(_)=>"relic",RoomChoiceKind::Sapphire=>"sapphire",RoomChoiceKind::Skip=>"skip",RoomChoiceKind::ConfirmSkip=>"confirm_skip",RoomChoiceKind::CancelSkip=>"cancel_skip",RoomChoiceKind::Gold(_)=>"gold",other=>panic!("unexpected room choice {other:?}")}}
/// Room decision projection: room kind, event kind, terminal, and candidate i's choice kind at action row i.
fn check_room(o:&RunObservation,d:&PublicDecision,spec:&Value,room:&str){
 let ctx=group(o,"context");assert_eq!(cell(ctx,0,col(spec,"context","room_kind")).0,vocab_id(spec,"room",room));assert_eq!(cell(ctx,0,col(spec,"context","content_profile")),(1,1));
 let n=d.candidates.len();assert_eq!(o.action_mask.iter().filter(|m|**m==1).count(),n);assert!(o.action_mask[..n].iter().all(|m|*m==1));
 let a=group(o,"actions");let ck=col(spec,"actions","choice_kind");
 for (i,c) in d.candidates.iter().enumerate(){if let GameAction::Choose(id)=c.action{let kind=&d.room.as_ref().unwrap().choices.iter().find(|x|x.id==id).unwrap().kind;assert_eq!(cell(a,i,ck).0,vocab_id(spec,"choice",choice_name(kind)),"candidate {i} row {i}");}}
}
fn d_for_parasite()->PublicDecision{let mut s=RunSession::new_m24(config()).unwrap();s.reset(Some(7)).unwrap();latest(&s)}
fn choose(r:&mut Run,pred:impl Fn(&RoomChoiceKind)->bool){let d=r.decision().unwrap();let id=d.room.unwrap().choices.iter().find(|c|pred(&c.kind)&&c.status==RoomChoiceStatus::Offered).expect("offered").id;r.step(&GameAction::Choose(id)).unwrap();}

// Retained recorded transcripts in compact semantic form (h1-v1/tools/compact.py); never chooser output.
// C choose, M move, S select, K confirm, X cancel, u/d room potion, P play, L select, D deselect, F confirm_selection, E end_turn, U potion, W/A stance.
// seed-7-slime-boss-normal-budget-32768.attempt.json (M22 original normal, Act2Ready after 273) sha256 5157f25cbb63556ab1a536b1574c579feaee4e2738bb92fc35beec4d0a5fd6fe, 273 actions.
const M22_SEED7_SLIME_BOSS:&str="C1 M4 P11 P6 P10 P5 E P3@1 P9@1 P4@1 P2@1 E P4@1 C2 C3 C6 C4 C5 M12 U0 P13 P12@1 P8 P2@1 P7 E P1@1 P5 P6 E P4@1 P5 P3@1 E P12@1 P7 P8 E P1@1 P7 P4@1 E P9@1 C10 C11 C14 C12 C13 M18 C18 S20 M25 C44 S46 M32 C70 C76 M38 C78 S80 M44 U0 P15 P4@1 P6 P14 P13@1 F P8 P2@1 E P16@2 P12@2 C104 C105 C108 C106 C107 M50 U0 P16 P9@1 P6 P4@1 E P8 P14 P5 E P19@1 P15 F P3@1 E P20@2 P8 P2@3 C112 C113 C116 C114 C115 M58 C120 C122 C124 M65 C125 M71 U0 P17 P8 P15 F P12@1 P14 E P22@1 P4@1 P2@1 E P25@1 P10 P16@2 F E P28@2 P13@2 F P8 E P31@2 P7 P15 F E P34@2 P2@3 P4@3 E P35@3 P16@3 F P12@3 P15 F E P38@3 C128 C129 C133 C130 C131 C132 M79 C137 M85 U0 A P18 P9@1 P12@1 P16@2 F P17@2 P2@2 E P8 P15 F P5 E P4@2 P10 P17@3 E P4@3 P14 P15 F E P19@3 C140 C141 C144 C142 C143 M92 U0 P19 P18 P6 P2@1 P7 P5 E P16@2 F P8 P14 P17@2 E P20@2 P10 P17@3 E P21@3 P15 F P3@4 P18 E P22@4 P5 P8 P17@5 E P23@5 C148 C149 C152 C150 C151 M98 C156 M108 U0 P20 P18 P5 P12@1 P6 P3@1 P21 P22 E P4@1 P8 P13@1 F P16@1 F E P10 P19@1 P14 P2@1 E P26@1 P17@1 P9@1 P17@1 E P27@2 P12@2 P3@3 P17@3 E P28@3 P19@3 P13@3 F P18 P16@3 F C159 C160 C163 C161 C162 C167 C169 C175";
// seed-7-guardian-normal-budget-32768.attempt.json (M22 original normal, Death after 146) sha256 2811bfda8d6c301725c4cad4af7118f752acd50eb5930f23861d47078c95806c, 146 actions.
const M22_SEED7_GUARDIAN:&str="C1 M4 P11 P6 P10 P5 E P3@1 P9@1 P4@1 P2@1 E P4@1 C2 C3 C6 C4 C5 M12 U0 P13 P12@1 P8 P2@1 P7 E P1@1 P5 P6 E P4@1 P5 P3@1 E P12@1 P7 P8 E P1@1 P7 P4@1 E P9@1 C10 C11 C14 C12 C13 M18 C18 S20 M25 C44 S46 M32 C70 C76 M38 C78 S80 M44 U0 P15 P4@1 P6 P14 P13@1 F P8 P2@1 E P16@2 P12@2 C104 C105 C108 C106 C107 M50 U0 P16 P9@1 P6 P4@1 E P8 P14 P5 E P19@1 P15 F P3@1 E P20@2 P8 P2@3 C112 C113 C116 C114 C115 M58 C120 C122 C124 M65 C125 M71 U0 P17 P16@1 F P7 P5 P15 F P3@1 E P12@1 P6 P2@1 E P4@1 P10 E P5 P15 F P13@1 F E P2@1 P16@1 F P6 P3@1 E P10 P15 F E P14 P4@1 P5 E";
// m15-coexistence-prefix-input-v1/prefix-input.json (first 32 retained M15 actions) sha256 b98de6a89b2b07d5d3f37dc582f3b8954baa9dd01ff31b31be9163655a376477, 32 actions.
const M15_SEED7_PREFIX:&str="C0 M1 P4@1 C2 C3 C6 C4 C5 M9 U0@1 C10 C11 C17 C12 C13 M17 U0 P14 P11 L6 P3@1 E P12@2 C18 C19 C23 C20 C21 M24 P12@1 P11 L6";
// seed-7-SlimeBoss-Collector-DonuAndDeca-normal.attempt.json (first 32 retained M20 actions) sha256 a561fb8575e32bb61c8af99f9dd6bd23953dd1994f700c0bba4d3518c88c8377, 32 actions.
const M20_SEED7_PREFIX:&str="C1 M1 P2@1 P1@1 P4@1 E P8 P5 P9 E P6 P5 P9 E P7 P8 P2@1 E C2 C3 C7 C4 C5 M9 U0@1 P5 P8 P9 E P12 P1@1 E";
// seed-11-guardian-normal-budget-32768.attempt.json (M22 original normal, Death after 114) sha256 9549c61f0bacaa8bf3d5c5972545bdfcdf8f162f7191a1b45e54b950dd8f43bd, 114 actions.
const M22_SEED11_GUARDIAN:&str="C1 M3 P11 P5 P2@1 P10 E P4@1 P8 P1@1 E P4@1 P5 P2@1 E P7 P8 P6 E P8 P7 P4@1 E P10 P5 E P5 P3@1 P1@1 E P7 P4@1 C2 C3 C6 C4 C5 M11 C10 S12 M19 C34 S36 M27 U0 P13 P4@1 P12@1 P2@1 P7 E P3@1 P6 P5 E P8 P4@1 P2@1 E P10 P5 E P3@1 P2@1 C58 C59 C62 C60 C61 M34 U0 P14 P5 P10 P3@1 P9@1 P7 E P2@1 P8 P13@2 C66 C67 C70 C68 C69 M41 C75 S77 M48 U0 P15 P3@1 P5 P2@1 P6 E P8 P13@1 F P4@1 E P7 P12@1 P13@1 F E P9@1 P14 F E P2@1 P10 E";
const M15_PREFIX_REWARDS:[i32;32]=[0,1,0,0,0,0,0,0,1,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0];
const M20_PREFIX_REWARDS:[i32;32]=[0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0];
const M22_SLIME_PREFIX_REWARDS:[i32;32]=[0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0];
fn decode(t:&str)->GameAction{
 let (head,rest)=t.split_at(1);let mut it=rest.split('@');let n=it.next().filter(|s|!s.is_empty()).map(|s|s.parse::<u64>().unwrap());let target=it.next().map(|s|TargetId(s.parse().unwrap()));
 match head{"C"=>GameAction::Choose(ChoiceId(n.unwrap())),"M"=>GameAction::Move(NodeId(n.unwrap())),"S"=>GameAction::Select(SelectionId(n.unwrap())),"K"=>GameAction::Confirm,"X"=>GameAction::Cancel,
  "u"=>GameAction::UsePotion{slot:n.unwrap() as u32,target},"d"=>GameAction::DiscardPotion{slot:n.unwrap() as u32},
  "P"=>GameAction::Combat(CombatAction::PlayCard{instance:n.unwrap(),target}),"L"=>GameAction::Combat(CombatAction::SelectCard{instance:n.unwrap()}),"D"=>GameAction::Combat(CombatAction::DeselectCard{instance:n.unwrap()}),
  "F"=>GameAction::Combat(CombatAction::ConfirmSelection),"E"=>GameAction::Combat(CombatAction::EndTurn),"U"=>GameAction::Combat(CombatAction::UsePotion{slot:n.unwrap() as usize,target}),
  "W"=>GameAction::Combat(CombatAction::ChooseStance{stance:Stance::Wrath}),"A"=>GameAction::Combat(CombatAction::ChooseStance{stance:Stance::Calm}),_=>panic!("token {t}")}
}

#[test]
fn m24_descriptor_covers_union_and_preserves_old(){
 let spec=m24_run_observation_spec(&caps()).unwrap();let d=spec.descriptor();
 let mut h=Sha256::new();h.update(b"spire-gym/run-schema/m24-watcher-act3/v1\0");h.update(canonical_json(&d).unwrap());assert_eq!(spec.fingerprint(),<[u8;32]>::from(h.finalize()));
 assert_eq!((d["schema_id"].as_str(),d["profile_id"].as_str()),(Some(RUN_OBSERVATION_M24_SCHEMA_ID),Some("M24WatcherAct3")));assert_eq!(RUN_OBSERVATION_M24_SCHEMA_ID,"spire-gym-m24-watcher-act3-observation-v1");
 let names=|ids:&[CardId]|ids.iter().map(|id|json!(id.as_str())).collect::<Vec<_>>();let f=&d["semantics"]["finite_content"];
 let m22=m22_run_observation_spec(&caps()).unwrap().descriptor();let m22f=&m22["semantics"]["finite_content"];
 // 29 = M22's 28 + Parasite; 19 = M22's 18 + Red Mask; the 7 potions are unchanged.
 assert_eq!(f["permanent_cards"].as_array().unwrap(),&names(M24_WATCHER_ACT3_CARDS));assert_eq!(M24_WATCHER_ACT3_CARDS.len(),29);
 let mut want=m22f["permanent_cards"].as_array().unwrap().clone();want.push(json!("parasite"));assert_eq!(f["permanent_cards"].as_array().unwrap(),&want);
 let mut want=m22f["relics"].as_array().unwrap().clone();want.push(json!("red_mask"));assert_eq!(f["relics"].as_array().unwrap(),&want);assert_eq!(want.len(),19);
 assert_eq!(f["potions"],m22f["potions"]);assert_eq!(f["potions"].as_array().unwrap().len(),7);
 assert_eq!(f["generated_cards"],json!(["miracle","smite","safety"]));assert_eq!(f["status_cards"],json!(["dazed","burn","slimed","wound","void"]));
 let mut physical=vec![json!("padding")];physical.extend(names(M24_WATCHER_ACT3_CARDS));physical.extend(names(m24_content::GENERATED_ONLY));physical.extend(names(m24_content::STATUSES));
 assert_eq!(d["vocabularies"]["card"].as_array().unwrap(),&physical);assert_eq!(physical.len(),1+29+3+5);
 assert_eq!(d["vocabularies"]["relic"].as_array().unwrap().len(),1+19);assert_eq!(d["vocabularies"]["potion"].as_array().unwrap().len(),1+7);
 assert_eq!(f["playable_encounters"].as_array().unwrap().len(),40);assert_eq!(f["playable_enemies"].as_array().unwrap().len(),54);assert_eq!(f["map_only_boss_icons"],json!([]));
 assert_eq!(d["vocabularies"]["encounter"].as_array().unwrap().len(),1+40);assert_eq!(d["vocabularies"]["enemy"].as_array().unwrap().len(),1+54);
 for e in ACT3_ENCOUNTERS{assert!(f["playable_encounters"].as_array().unwrap().contains(&json!(e.as_str())),"{e:?}");vocab_id(&d,"encounter",e.as_str());}
 for e in NEW_ACT3_ENEMIES{assert!(f["playable_enemies"].as_array().unwrap().contains(&json!(e)),"{e}");vocab_id(&d,"enemy",e);}
 // Relic taxonomy = the frozen M22 rows, then the M24 addition Red Mask (tier special).
 let tax=f["relic_taxonomy"].as_array().unwrap();assert_eq!(tax.len(),19);assert_eq!(&tax[..18],&m22f["relic_taxonomy"].as_array().unwrap()[..]);assert_eq!(tax[18],json!({"id":"red_mask","tier":"special"}));
 // Producer/capture metadata is a projection of the single content owners; Act3 has its own capture scope.
 assert_eq!(f["capture_law"]["scope"],json!({"act1":m22_content::RELIC_CAPTURE_SCOPE_NAME,"act2":m23_content::ACT2_RELIC_CAPTURE_SCOPE_NAME,"act3":m24_content::ACT3_RELIC_CAPTURE_SCOPE_NAME}));
 assert_eq!(f["capture_law"]["scope"]["act3"],"run.m24.relic.capture");assert_eq!(f["capture_law"]["key_fields"],m22f["capture_law"]["key_fields"]);assert_eq!(f["producers"],m22f["producers"]);
 for i in ACT3_INTENTS{vocab_id(&d,"intent",i);}for p in ACT3_POWERS.iter().chain(&["mantra","devotion","foresight","nirvana","establishment","master_reality","battle_hymn","mental_fortress","rushdown","like_water"]){vocab_id(&d,"power",p);}
 for ev in ["neow","joust","falling","tomb_of_lord_red_mask"]{vocab_id(&d,"event",ev);}for r in ["act3_boss_complete","act3_ending","act4_ready"]{vocab_id(&d,"room",r);}vocab_id(&d,"choice","act3_continue");
 assert_eq!(d["vocabularies"]["terminal"],json!(["ongoing","death","empty_deck","act3_no_keys"]));assert_eq!(d["vocabularies"]["stance"],json!(["neutral","wrath","calm","divinity"]));
 assert!(d["semantics"]["cutoff"].as_str().unwrap().contains("before the Act4 door"));
 // Every Act3 setup is representable from the actual setup/AI owners over real turns (relabelled M21-owned combat, fixed driver).
 let (mut powers,mut enemies,mut intents)=(BTreeSet::new(),BTreeSet::new(),BTreeSet::new());
 for e in ACT3_ENCOUNTERS{let mut r=fixture(e,M21WatcherCombatStart::Canonical);for _ in 0..80{let dd=relabel(&r);if dd.combat.is_none()||dd.terminal.is_some(){break}
  let o=encode_run_m24(&dd,None,&caps()).unwrap_or_else(|err|panic!("{e:?}: {err:?}"));check_combat(&o,&dd,&d,&mut powers,&mut enemies,&mut intents);if !drive_once(&mut r){break}}}
 println!("relabelled setups: powers {powers:?}\nenemies {enemies:?}\nintents {intents:?}");
 for x in NEW_ACT3_ENEMIES{assert!(enemies.contains(x),"{x} actually produced");}
 for x in ["curiosity","unawakened","regenerate","time_warp","slow","intangible","malleable","reactive","shifting","fading","explosive","thorns","life_link","artifact","strength_up"]{assert!(powers.contains(x),"{x} actually installed");}
 for x in &intents{assert!(ACT3_INTENTS.contains(&x.as_str())||vocab_id(&d,"intent",x)>0);}
 // Foreign authored setups outside the captured union fail closed when they name an unknown actor; so does a foreign permanent.
 let mut foreign=0;for (id,_,_) in spire_core::combat::enemies::A0_ENCOUNTER_SETUPS.iter(){if f["playable_encounters"].as_array().unwrap().contains(&json!(id.as_str())){continue}let r=fixture(*id,M21WatcherCombatStart::Canonical);let dd=relabel(&r);if dd.combat.as_ref().unwrap().enemies.iter().any(|en|!f["playable_enemies"].as_array().unwrap().contains(&json!(en.enemy_id.as_str()))){assert!(encode_run_m24(&dd,None,&caps()).is_err(),"{id:?}");foreign+=1;}}
 assert!(foreign>0,"Act4 actors are foreign to M24");
 let r=fixture(EncounterId::JawWorm,M21WatcherCombatStart::Canonical);let mut dd=relabel(&r);dd.combat.as_mut().unwrap().hand[0].id=CardId::Strike;assert!(encode_run_m24(&dd,None,&caps()).is_err(),"foreign Ironclad permanent");
 // Older descriptors and fingerprints stay byte-identical.
 let c=caps();let p=combat_policy_capacities();
 let now=[("m9",run_observation_spec(&RunCapacities::m9_default()).unwrap().fingerprint()),("m10",m10_run_observation_spec(&c).unwrap().fingerprint()),("m12",m12_run_observation_spec(&c).unwrap().fingerprint()),("m13",m13_run_observation_spec(&c).unwrap().fingerprint()),("m14",m14_run_observation_spec(&c).unwrap().fingerprint()),("m15",m15_run_observation_spec(&c).unwrap().fingerprint()),("m17",m17_run_observation_spec(&c).unwrap().fingerprint()),("m18",m18_run_observation_spec(&c).unwrap().fingerprint()),("m19",m19_run_observation_spec(&c).unwrap().fingerprint()),("m20",m20_run_observation_spec(&c).unwrap().fingerprint()),("m22",m22_run_observation_spec(&c).unwrap().fingerprint()),("m23",m23_run_observation_spec(&c).unwrap().fingerprint()),("policy",policy_run_observation_spec(&c).unwrap().fingerprint()),("combat_v1",combat_policy_observation_spec().unwrap().fingerprint()),("combat_v2",combat_policy_observation_spec_v2().unwrap().fingerprint()),("m21",watcher_combat_observation_spec(&p).unwrap().fingerprint()),("m16",defect_combat_observation_spec(&p).unwrap().fingerprint())];
 assert_eq!(now.len(),FROZEN.len());for ((a,fp),(b,want)) in now.iter().zip(FROZEN){assert_eq!(a,b);assert_eq!(hex(*fp),*want,"{a} descriptor changed");}
 assert!(FROZEN.iter().all(|(_,f)|*f!=hex(spec.fingerprint())),"M24 has its own fingerprint");
}

#[test]
fn m24_encoding_current_watcher_and_act3_state(){
 let spec=spec();let (stance,mantra)=(col(&spec,"player","stance"),col(&spec,"player","mantra_gained"));
 // Stance choice (constructed M24 stage, M24 identity, no relabel): nonphysical Wrath/Calm rows referenced by actions; Calm is current stance 2.
 let mut r=m24_diagnostic(M24Diagnostic::StanceChoice).unwrap();let d=r.decision().unwrap();assert_eq!(d.content_profile,Some(RoomScenario::M24WatcherAct3));assert_eq!(d.act,3);
 let o=encode_run_m24(&d,r.public_map().as_ref(),&caps()).unwrap();assert_eq!(group(&o,"stance_choices").values,vec![1,2]);let refc=col(&spec,"actions","stance_choice_ref");
 assert_eq!((0..d.candidates.len()).map(|i|cell(group(&o,"actions"),i,refc).0).filter(|v|*v>=0).collect::<Vec<_>>(),vec![0,1]);
 r.step(&GameAction::Combat(CombatAction::ChooseStance{stance:Stance::Calm})).unwrap();let d=r.decision().unwrap();let o=encode_run_m24(&d,r.public_map().as_ref(),&caps()).unwrap();assert_eq!(cell(group(&o,"player"),0,stance),(stance_code(Stance::Calm),1));
 // Mantra/Divinity: current stance and Mantra power versus gained history (relabelled M21-owned combat).
 let mut r=fixture(EncounterId::JawWorm,supplied(&[CardId::Worship;10]));assert!(play(&mut r,CardId::Worship));assert!(end_turn(&mut r));assert!(play(&mut r,CardId::Worship));let d=relabel(&r);let o=encode_run_m24(&d,None,&caps()).unwrap();let c=d.combat.as_ref().unwrap();
 assert_eq!(c.character.stance,Stance::Divinity);assert_eq!(cell(group(&o,"player"),0,stance).0,3);assert_eq!(cell(group(&o,"player"),0,mantra).0,i64::from(c.mantra_gained.unwrap()));
 // Scry (constructed M24 stage): reveal top-first and selected ordinals B,A after deselect/reselect; Confirm then encodes.
 let mut r=m24_diagnostic(M24Diagnostic::ScryChoice).unwrap();let sel=r.decision().unwrap().combat.unwrap().selection.unwrap();assert_eq!(sel.operation,spire_core::combat::SelectionOperation::Scry);let (a,b)=(sel.candidates[0].instance.0,sel.candidates[1].instance.0);
 for x in [CombatAction::SelectCard{instance:a},CombatAction::SelectCard{instance:b},CombatAction::DeselectCard{instance:a},CombatAction::SelectCard{instance:a}]{r.step(&GameAction::Combat(x)).unwrap();}
 let d=r.decision().unwrap();let o=encode_run_m24(&d,r.public_map().as_ref(),&caps()).unwrap();let g=group(&o,"selection");let (rp,so)=(col(&spec,"selection","reveal_position"),col(&spec,"selection","selected_ordinal"));
 assert_eq!((cell(g,0,rp),cell(g,1,rp)),((0,1),(1,1)));assert_eq!((cell(g,0,so).0,cell(g,1,so).0),(2,1));r.step(&GameAction::Combat(CombatAction::ConfirmSelection)).unwrap();encode_run_m24(&r.decision().unwrap(),r.public_map().as_ref(),&caps()).unwrap();
 // Meditate return (constructed M24 stage): one Watcher operation row per eligible card.
 let r=m24_diagnostic(M24Diagnostic::MeditateChoice).unwrap();let d=r.decision().unwrap();let sel=d.combat.as_ref().unwrap().selection.as_ref().unwrap();assert_eq!(sel.operation,spire_core::combat::SelectionOperation::MeditateReturn);
 let o=encode_run_m24(&d,r.public_map().as_ref(),&caps()).unwrap();assert_eq!(occupied(group(&o,"selection")),sel.candidates.len());
 // Supplied M24 rooms (fixtures one edge before the real Act3 room; full-HP supplied bosses), then only real owners and a fixed driver.
 let (mut powers,mut enemies,mut intents)=(BTreeSet::new(),BTreeSet::new(),BTreeSet::new());let (mut half_dead,mut summoned,mut burn)=(0,0,0);
 let starts:Vec<(String,Run)>=M24PlayedFixture::ALL.iter().map(|f|(f.name().to_owned(),m24_played_fixture(*f).unwrap())).chain(M24DiagnosticBoss::ALL.iter().map(|b|(b.name().to_owned(),m24_supplied_boss(*b,[false;3]).unwrap()))).collect();
 for (name,mut r) in starts{let mv=first_move(&r.decision().unwrap());r.step(&mv).unwrap();let first=r.decision().unwrap().combat.as_ref().map(|c|c.enemies.len()).expect("actual Act3 combat entered");
  for _ in 0..120{let d=r.decision().unwrap();if d.combat.is_none()||d.terminal.is_some(){break}assert_eq!(d.content_profile,Some(RoomScenario::M24WatcherAct3));assert_eq!(d.act,3);
   let o=encode_run_m24(&d,r.public_map().as_ref(),&caps()).unwrap_or_else(|e|panic!("{name}: {e:?}"));check_combat(&o,&d,&spec,&mut powers,&mut enemies,&mut intents);let c=d.combat.as_ref().unwrap();
   half_dead+=c.enemies.iter().filter(|e|e.half_dead).count();summoned+=usize::from(c.enemies.len()>first);burn+=c.hand.iter().chain(&c.discard).filter(|x|x.id==CardId::Burn).count();
   let pc=group(&o,"permanent_cards");for (row,obj) in d.objects.iter().enumerate(){assert_eq!(cell(pc,row,0).0,vocab_id(&spec,"card",obj.id.as_str()));}
   if !drive_once(&mut r){break}}}
 println!("supplied rooms: powers {powers:?}\nenemies {enemies:?}\nhalf_dead {half_dead} summoned {summoned} burn {burn}");
 for x in ["slow","intangible","time_warp","curiosity","artifact"]{assert!(powers.contains(x),"{x} installed amount encoded");}
 assert!(half_dead>0,"actual half-dead Darkling row");assert!(summoned>0,"actual summoned actor row");assert!(burn>0,"actual Nemesis/Orb Walker Burn row");
 // Parasite permanent row (constructed public view, labelled; actual Implant custody is core-lane and H4 transcript coverage):
 // a non-upgradable permanent Curse in the finite union encodes as parasite; an upgraded Parasite fails closed.
 let mut pd=d_for_parasite();let n=pd.objects.len();pd.objects.push(PublicObject{instance:999,id:CardId::Parasite,upgrades:0,misc:0});let o=encode_run_m24(&pd,None,&caps()).unwrap();assert_eq!(cell(group(&o,"permanent_cards"),n,0).0,vocab_id(&spec,"card","parasite"));
 pd.objects[n].upgrades=1;assert!(encode_run_m24(&pd,None,&caps()).is_err());
 // Void row (constructed public view, labelled): a generated Void encodes as the void status; an upgraded Void or a generated Strike fails closed.
 let mut r=m24_played_fixture(M24PlayedFixture::SlowGiantHead).unwrap();let mv=first_move(&r.decision().unwrap());r.step(&mv).unwrap();let d=r.decision().unwrap();
 let mut v=d.clone();{let card=&mut v.combat.as_mut().unwrap().hand[0];card.id=CardId::Void;card.upgrades=0;card.origin=spire_core::combat::CombatCardRef::CombatLocal(900);card.watcher=None;}
 match encode_run_m24(&v,r.public_map().as_ref(),&caps()){Ok(o)=>assert_eq!(cell(group(&o,"combat_cards"),0,col(&spec,"combat_cards","card_kind")).0,vocab_id(&spec,"card","void")),Err(e)=>panic!("Void row: {e:?}")}
 let mut up=v.clone();up.combat.as_mut().unwrap().hand[0].upgrades=1;assert!(encode_run_m24(&up,r.public_map().as_ref(),&caps()).is_err());
 // Falling, Tomb (with and without a mask) and the floor-43-style chest (constructed M24 stages): candidate i is row i with its choice kind.
 for (stage,room) in [(M24Diagnostic::Falling,"event"),(M24Diagnostic::Tomb{has_mask:false},"event"),(M24Diagnostic::Tomb{has_mask:true},"event"),(M24Diagnostic::Chest,"treasure")]{
  let r=m24_diagnostic(stage).unwrap();let d=r.decision().unwrap();let o=encode_run_m24(&d,r.public_map().as_ref(),&caps()).unwrap_or_else(|e|panic!("{stage:?}: {e:?}"));check_room(&o,&d,&spec,room);assert!(!d.candidates.is_empty());
  if room=="event"{let ev=cell(group(&o,"context"),0,col(&spec,"context","event_kind")).0;assert!(ev==vocab_id(&spec,"event","falling")||ev==vocab_id(&spec,"event","tomb_of_lord_red_mask"),"{stage:?}");}
 }
 // Ending views (constructed boss50 Victory record): Act3BossComplete, three Act3Ending stages, then Act4Ready (all keys) or stage 4 then Act3NoKeys.
 for boss in M24DiagnosticBoss::ALL{for keys in [[true;3],[true,true,false],[false;3]]{
  let mut r=m24_diagnostic(M24Diagnostic::BossSource{boss,keys}).unwrap();let d=r.decision().unwrap();check_room(&encode_run_m24(&d,r.public_map().as_ref(),&caps()).unwrap(),&d,&spec,"act3_boss_complete");
  choose(&mut r,|k|*k==RoomChoiceKind::Proceed);
  for stage in 1..=3u8{let d=r.decision().unwrap();check_room(&encode_run_m24(&d,r.public_map().as_ref(),&caps()).unwrap(),&d,&spec,"act3_ending");choose(&mut r,|k|*k==RoomChoiceKind::Act3Continue{stage});}
  let d=r.decision().unwrap();let o=encode_run_m24(&d,r.public_map().as_ref(),&caps()).unwrap();
  if keys==[true;3]{check_room(&o,&d,&spec,"act4_ready");assert_eq!((d.act,d.floor,d.terminal),(3,50,None));assert!(d.candidates.is_empty());assert_eq!(o.action_mask,vec![0;caps().actions]);assert_eq!(cell(group(&o,"context"),0,col(&spec,"context","terminal_kind")).0,0);}
  else{check_room(&o,&d,&spec,"act3_ending");choose(&mut r,|k|*k==RoomChoiceKind::Act3Continue{stage:4});let d=r.decision().unwrap();assert_eq!(d.terminal,Some(RunTerminal::Act3NoKeys));let o=encode_run_m24(&d,r.public_map().as_ref(),&caps()).unwrap();
   assert_eq!(cell(group(&o,"context"),0,col(&spec,"context","terminal_kind")).0,vocab_id(&spec,"terminal","act3_no_keys"));assert_eq!(o.action_mask,vec![0;caps().actions]);}
 }}
}

#[test]
fn m24_numeric_privacy_and_capacity(){
 // Same public start under different seeds/RNG: identical numeric result; physical handles never become features.
 let (mut a,mut b)=(RunSession::new_m24(config()).unwrap(),RunSession::new_m24(config()).unwrap());assert_eq!(a.reset(Some(7)).unwrap().observation,b.reset(Some(11)).unwrap().observation);
 let mut d=latest(&a);let original=encode_run_m24(&d,None,&caps()).unwrap();for c in &mut d.objects{c.instance+=1000;}assert_eq!(encode_run_m24(&d,None,&caps()).unwrap(),original);d.objects[0].id=CardId::Strike;assert!(encode_run_m24(&d,None,&caps()).is_err(),"foreign permanent fails closed");
 // Equal public Act3 combat state reached from the same supplied start encodes identically; hidden draw order, AI and RNG never enter.
 let mut x=m24_played_fixture(M24PlayedFixture::SlowGiantHead).unwrap();let mv=first_move(&x.decision().unwrap());x.step(&mv).unwrap();let y=x.clone();
 assert_eq!(encode_run_m24(&x.decision().unwrap(),x.public_map().as_ref(),&caps()).unwrap(),encode_run_m24(&y.decision().unwrap(),y.public_map().as_ref(),&caps()).unwrap());
 // Constructed public boundaries (labelled): Act2Ready/Act3Ready never publish in M24; Act4Ready must be healthy zero-action at 3/50; Act4 and Heart fail closed.
 let neow=latest(&a);act(&mut a,neow.candidates[0].action.clone());let map_view=latest(&a);
 let at=|act:u8,floor:u32,kind:RoomViewKind,keep:bool|{let mut d=map_view.clone();d.act=act;d.floor=floor;d.room.as_mut().unwrap().kind=kind;d.room.as_mut().unwrap().choices.clear();if !keep{d.candidates.clear();}d};
 let o=encode_run_m24(&at(3,50,RoomViewKind::Act4Ready,false),None,&caps()).unwrap();assert_eq!(o.action_mask,vec![0;caps().actions]);assert_eq!(occupied(group(&o,"actions")),0);
 for bad in [at(3,50,RoomViewKind::Act4Ready,true),at(3,49,RoomViewKind::Act4Ready,false),at(4,50,RoomViewKind::Act4Ready,false),at(3,34,RoomViewKind::Act3Ready,false),at(2,17,RoomViewKind::Act2Ready,false),at(4,51,RoomViewKind::Map,true),at(3,51,RoomViewKind::Map,true)]{
  assert!(matches!(encode_run_m24(&bad,None,&caps()),Err(RunSessionError::UnsupportedBoundary(_))),"{:?}/{}/{:?}",bad.act,bad.floor,bad.room.as_ref().unwrap().kind);}
 let mut heart=map_view.clone();heart.terminal=Some(RunTerminal::HeartVictory);heart.candidates.clear();assert!(encode_run_m24(&heart,None,&caps()).is_err());
 let mut over=map_view.clone();over.act=3;over.floor=50;over.terminal=Some(RunTerminal::Act3NoKeys);over.candidates.clear();encode_run_m24(&over,None,&caps()).expect("Act3NoKeys terminal encodes");
 // Each too-small capacity group returns its structured error without truncation, filtering or paging.
 let mut r=m24_played_fixture(M24PlayedFixture::DaggersReptomancer).unwrap();let mv=first_move(&r.decision().unwrap());r.step(&mv).unwrap();let d=r.decision().unwrap();let map=r.public_map().unwrap();let c=d.combat.as_ref().unwrap();
 let cards=c.hand.len()+c.discard.len()+c.exhaust.len();let powers=c.player.powers.len()+c.enemies.iter().map(|e|e.powers.len()).sum::<usize>();
 let mut checks:Vec<(&str,usize,Box<dyn Fn(&mut RunCapacities,usize)>)>=vec![("map_nodes",map.nodes.len(),Box::new(|c,n|c.map_nodes=n)),("permanent_cards",d.objects.len(),Box::new(|c,n|c.permanent_cards=n)),("combat_cards",cards,Box::new(|c,n|c.combat_cards=n)),("enemies",c.enemies.len(),Box::new(|c,n|c.enemies=n)),("actions",d.candidates.len(),Box::new(|c,n|c.actions=n))];
 if powers>0{checks.push(("powers",powers,Box::new(|c,n|c.powers=n)));}assert!(c.enemies.len()>1,"Reptomancer room has summoned actors");
 for (g,required,set) in &checks{let mut small=caps();set(&mut small,required-1);assert_eq!(encode_run_m24(&d,Some(&map),&small),Err(RunSessionError::Capacity{group:g,required:*required,capacity:required-1}),"{g}");}
 let r=m24_diagnostic(M24Diagnostic::ScryChoice).unwrap();let sd=r.decision().unwrap();let n=sd.combat.as_ref().unwrap().selection.as_ref().unwrap().candidates.len();let mut small=caps();small.selection=n-1;assert_eq!(encode_run_m24(&sd,r.public_map().as_ref(),&small),Err(RunSessionError::Capacity{group:"selection",required:n,capacity:n-1}));
 let r=m24_supplied_boss(M24DiagnosticBoss::TimeEater,[false;3]).unwrap();let pd=r.decision().unwrap();let k=pd.player.potions.len();let mut small=caps();small.potions=k-1;assert_eq!(encode_run_m24(&pd,r.public_map().as_ref(),&small),Err(RunSessionError::Capacity{group:"potions",required:k,capacity:k-1}));
 let k=pd.player.relics.len();assert_eq!(k,6);let mut small=caps();small.relics=k-1;assert_eq!(encode_run_m24(&pd,r.public_map().as_ref(),&small),Err(RunSessionError::Capacity{group:"relics",required:k,capacity:k-1}));
}
fn transcript(s:&str)->Vec<GameAction>{s.split(' ').map(decode).collect()}
fn drive(s:&mut RunSession,actions:&[GameAction])->Vec<RunStepResult>{actions.iter().enumerate().map(|(i,a)|{let d=latest(s);assert!(d.candidates.iter().any(|c|&c.action==a),"retained action {i} {a:?} not offered");act(s,a.clone())}).collect()}
fn boss1(a1:RunAct1BossTarget)->EncounterId{match a1{RunAct1BossTarget::Guardian=>EncounterId::Guardian,RunAct1BossTarget::Hexaghost=>EncounterId::Hexaghost,RunAct1BossTarget::SlimeBoss=>EncounterId::SlimeBoss}}
fn boss2(a2:RunAct2BossTarget)->EncounterId{match a2{RunAct2BossTarget::Champ=>EncounterId::Champ,RunAct2BossTarget::BronzeAutomaton=>EncounterId::BronzeAutomaton,RunAct2BossTarget::Collector=>EncounterId::Collector}}
const A1:[RunAct1BossTarget;3]=[RunAct1BossTarget::Guardian,RunAct1BossTarget::Hexaghost,RunAct1BossTarget::SlimeBoss];
const A2:[RunAct2BossTarget;3]=[RunAct2BossTarget::Champ,RunAct2BossTarget::BronzeAutomaton,RunAct2BossTarget::Collector];
const A3:[RunAct3BossTarget;3]=[RunAct3BossTarget::AwakenedOne,RunAct3BossTarget::TimeEater,RunAct3BossTarget::DonuAndDeca];
fn wire(r:&RunEpisodeRecord)->Value{serde_json::from_slice(&r.to_json().unwrap()).unwrap()}
fn import(v:&Value)->Result<RunEpisodeRecord,RunSessionError>{RunEpisodeRecord::from_json(&serde_json::to_vec(v).unwrap())}
fn m23_cfg(a1:RunAct1BossTarget,a2:RunAct2BossTarget)->RunSessionConfig{RunSessionConfig{boss_history:RunBossHistory::M23WatcherAct2{act1:a1,act2:a2},..config()}}

#[test]
fn m24_config_history_validation_precedes_initialization(){
 // All 27 typed triples are valid Watcher A0 configurations; the canonical Act1 map excludes the configured target's complement.
 for a1 in A1{for a2 in A2{for a3 in A3{let mut s=RunSession::new_m24(cfg(a1,a2,a3)).unwrap();s.reset(Some(7)).unwrap();assert_eq!(latest(&s).content_profile,Some(RoomScenario::M24WatcherAct3));
  let neow=latest(&s);act(&mut s,neow.candidates[0].action.clone());assert_eq!(s.reproduction().unwrap().latest_map.unwrap().boss,Some(boss1(a1)),"{a1:?}/{a2:?}/{a3:?} Act1 complement");}}}
 // Exact M23 Acts1–2 prefix (retained normal M22/M23 transcript) step by step, then the Act2 Map whose boss is the configured target.
 let act1=transcript(M22_SEED7_SLIME_BOSS);
 for a2 in A2{for a3 in [RunAct3BossTarget::AwakenedOne,RunAct3BossTarget::DonuAndDeca]{let mut s=RunSession::new_m24(cfg(RunAct1BossTarget::SlimeBoss,a2,a3)).unwrap();s.reset(Some(7)).unwrap();let mut m=RunSession::new_m23(m23_cfg(RunAct1BossTarget::SlimeBoss,a2)).unwrap();m.reset(Some(7)).unwrap();
  for (i,a) in act1.iter().enumerate(){let (mut x,mut y)=(latest(&s),latest(&m));x.content_profile=None;y.content_profile=None;assert_eq!(x,y,"M23 prefix diverged at {i}");let (p,q)=(act(&mut s,a.clone()),act(&mut m,a.clone()));assert_eq!((p.reward,p.terminated),(q.reward,q.terminated));}
  let (mut x,mut y)=(latest(&s),latest(&m));x.content_profile=None;y.content_profile=None;assert_eq!(x,y);assert_eq!((x.act,x.floor,x.room.as_ref().unwrap().kind),(2,17,RoomViewKind::Map));
  let map=s.reproduction().unwrap().latest_map.unwrap();assert_eq!((map.act,map.boss),(2,Some(boss2(a2))),"Act2 complement");assert_eq!(s.reproduction().unwrap().latest_map,m.reproduction().unwrap().latest_map);}}
 // Rejected before any Run/resource/RNG startup (initializer counts and Act3 complements are asserted in the owning module).
 let (g,c,a)=(RunAct1BossTarget::Guardian,RunAct2BossTarget::Champ,RunAct3BossTarget::AwakenedOne);
 let other=[RunBossHistory::Act1GuardianUnseen,RunBossHistory::AwakenedUnseen,RunBossHistory::M17DefectAct1{act1:g},RunBossHistory::M13SilentAct2{act1:g,act2:c},RunBossHistory::M18DefectAct2{act1:g,act2:c},RunBossHistory::M23WatcherAct2{act1:g,act2:c},RunBossHistory::M14SilentAct3{act1:g,act2:c,act3:a},RunBossHistory::M15SilentHeart{act1:g,act2:c,act3:a},RunBossHistory::M19DefectAct3{act1:g,act2:c,act3:a},RunBossHistory::M20DefectHeart{act1:g,act2:c,act3:a}];
 for h in other{assert_eq!(RunSession::new_m24(RunSessionConfig{boss_history:h,..config()}).err(),Some(RunSessionError::Input("boss history does not match Run profile")),"{h:?}");}
 for ch in [Character::Ironclad,Character::Silent,Character::Defect]{assert_eq!(RunSession::new_m24(RunSessionConfig{character:ch,..config()}).err(),Some(RunSessionError::Input("M24 requires Watcher A0")));}
 assert_eq!(RunSession::new_m24(RunSessionConfig{ascension:1,..config()}).err(),Some(RunSessionError::Input("M24 requires Watcher A0")));
 let mut bad=config();bad.capacities.actions=0;assert!(RunSession::new_m24(bad).is_err());
 // No normalization into an old profile; old profiles keep their own histories.
 assert!(RunSession::new_m23(config()).is_err());assert!(RunSession::new_m22(config()).is_err());assert!(RunSession::new_m14(RunSessionConfig{character:Character::Silent,..config()}).is_err());assert!(RunSession::new_m19(RunSessionConfig{character:Character::Defect,..config()}).is_err());
 let old:M9BossHistory=M9BossHistory::AwakenedUnseen;RunSession::new(RunSessionConfig{character:Character::Ironclad,ascension:0,partition:Partition::Test,boss_history:old,capacities:RunCapacities::m9_default(),record_replay:true}).unwrap();
 RunSession::new_m23(m23_cfg(g,c)).unwrap();
}

#[test]
fn m24_raw_json_strict(){
 let mut s=RunSession::new_m24(cfg(RunAct1BossTarget::SlimeBoss,RunAct2BossTarget::BronzeAutomaton,RunAct3BossTarget::TimeEater)).unwrap();s.reset(Some(7)).unwrap();drive(&mut s,&transcript(M22_SEED7_SLIME_BOSS)[..3]);s.fail_delivery("transport acknowledgement lost").unwrap();
 let r=s.record().unwrap().unwrap();let bytes=r.to_json().unwrap();let text=String::from_utf8(bytes.clone()).unwrap();
 // Shared target codecs: Act1/Act2 snake_case, Act3 the accepted kebab-case Act3 codec (as M14/M15/M19/M20).
 assert!(text.contains(r#""boss_history":{"m24-watcher-act3":{"act1":"slime_boss","act2":"bronze_automaton","act3":"time-eater"}}"#),"{text}");assert_eq!(RunEpisodeRecord::from_json(&bytes).unwrap(),r);
 assert_eq!((r.schema_id.as_str(),r.profile_id.as_str(),r.rules_id.as_str(),r.content_id.as_str()),("spire-gym-m24-watcher-act3-replay-v1","M24WatcherAct3","spire-gym-native-m24-rules-v1","spire-gym-m24-watcher-a0-acts1-3-captured-v1"));
 assert_eq!((RUN_RECORD_SCHEMA_ID_M24,RUN_RULES_ID_M24,RUN_CONTENT_ID_M24),(r.schema_id.as_str(),r.rules_id.as_str(),r.content_id.as_str()));
 for (a1,a2,a3) in [(RunAct1BossTarget::Guardian,RunAct2BossTarget::Champ,RunAct3BossTarget::AwakenedOne),(RunAct1BossTarget::Hexaghost,RunAct2BossTarget::Collector,RunAct3BossTarget::DonuAndDeca)]{let mut t=RunSession::new_m24(cfg(a1,a2,a3)).unwrap();t.reset(Some(7)).unwrap();let tr=t.record().unwrap().unwrap();assert_eq!(RunEpisodeRecord::from_json(&tr.to_json().unwrap()).unwrap(),tr);}
 // Byte-level duplicates at every nesting level reject before Value conversion.
 for (from,to) in [(r#""closed":false"#,r#""closed":false,"closed":false"#),(r#""character":"watcher""#,r#""character":"watcher","character":"watcher""#),(r#"{"act1":"slime_boss""#,r#"{"act1":"slime_boss","act1":"slime_boss""#),(r#""act2":"bronze_automaton""#,r#""act2":"bronze_automaton","act2":"collector""#),(r#""act3":"time-eater""#,r#""act3":"time-eater","act3":"time-eater""#),
  (r#"{"m24-watcher-act3":{"act1":"slime_boss","act2":"bronze_automaton","act3":"time-eater"}}"#,r#"{"m24-watcher-act3":{"act1":"slime_boss","act2":"bronze_automaton","act3":"time-eater"},"m24-watcher-act3":{"act1":"slime_boss","act2":"bronze_automaton","act3":"time-eater"}}"#),(r#""choice":1"#,r#""choice":1,"choice":1"#),(r#""kind":"delivery_fault""#,r#""kind":"delivery_fault","kind":"delivery_fault""#),(r#""actions":2048"#,r#""actions":2048,"actions":2048"#)]{
  let dup=text.replacen(from,to,1);assert_ne!(dup,text,"{from}");assert!(RunEpisodeRecord::from_json(dup.as_bytes()).is_err(),"duplicate {from}");}
 // Unknown/extra keys at every new nesting level; partial, mixed, wrong-type, unknown-value and foreign histories reject.
 let v=wire(&r);for pointer in ["","/config","/config/capacities","/config/boss_history","/config/boss_history/m24-watcher-act3","/steps/0","/steps/0/action","/fault","/fault/error"]{let mut w=v.clone();w.pointer_mut(pointer).unwrap().as_object_mut().unwrap().insert("extra".into(),json!(1));assert!(import(&w).is_err(),"extra at {pointer}");}
 for h in [json!("act1-slime-boss-unseen"),json!("awakened-unseen"),json!({"m23-watcher-act2":{"act1":"slime_boss","act2":"bronze_automaton"}}),json!({"m19-defect-act3":{"act1":"slime_boss","act2":"bronze_automaton","act3":"time-eater"}}),json!({"m14-silent-act3":{"act1":"slime_boss","act2":"bronze_automaton","act3":"time-eater"}}),
  json!({"m24-watcher-act3":{"act1":"slime_boss","act2":"bronze_automaton"}}),json!({"m24-watcher-act3":{"act2":"bronze_automaton","act3":"time-eater"}}),json!({"m24-watcher-act3":{"act1":"slime_boss","act3":"time-eater"}}),json!({"m24-watcher-act3":{"act1":"slime_boss","act2":"bronze_automaton","act3":"time_eater"}}),json!({"m24-watcher-act3":{"act1":"slime_boss","act2":"bronze_automaton","act3":"TimeEater"}}),
  json!({"m24-watcher-act3":{"act1":"slime_boss","act2":"automaton","act3":"time-eater"}}),json!({"m24-watcher-act3":{"act1":1,"act2":"bronze_automaton","act3":"time-eater"}}),json!({"m24-watcher-act3":{"act1":"slime_boss","act2":"bronze_automaton","act3":null}}),json!({"m24-watcher-act3":{"act1":"slime_boss","act2":"bronze_automaton","act3":"time-eater","act4":"heart"}}),json!({"m24-watcher-act3":"slime_boss"}),json!({"m24-watcher-act3":["slime_boss","bronze_automaton","time-eater"]}),json!({"m24-watcher-act3":{"act1":"slime_boss","act2":"bronze_automaton","act3":"time-eater"},"m23":1})]{
  let mut w=v.clone();w["config"]["boss_history"]=h.clone();assert!(import(&w).is_err(),"{h}");}
 // New-profile data is never normalized into an old profile, and old profiles keep their spellings.
 let mut w=v.clone();w["schema_id"]=json!(RUN_RECORD_SCHEMA_ID_M23);w["profile_id"]=json!("M23WatcherAct2");w["rules_id"]=json!(RUN_RULES_ID_M23);w["content_id"]=json!(RUN_CONTENT_ID_M23);assert!(import(&w).is_err());
 let mut w=v.clone();w["schema_id"]=json!(RUN_RECORD_SCHEMA_ID_M23);w["profile_id"]=json!("M23WatcherAct2");w["rules_id"]=json!(RUN_RULES_ID_M23);w["content_id"]=json!(RUN_CONTENT_ID_M23);w["config"]["boss_history"]=json!({"m23-watcher-act2":{"act1":"slime_boss","act2":"bronze_automaton"}});assert!(import(&w).is_err(),"an M24 fingerprint is not an M23 record");
 let mut m23=RunSession::new_m23(m23_cfg(RunAct1BossTarget::SlimeBoss,RunAct2BossTarget::BronzeAutomaton)).unwrap();m23.reset(Some(7)).unwrap();let mv=wire(&m23.record().unwrap().unwrap());assert_eq!(mv["config"]["boss_history"],json!({"m23-watcher-act2":{"act1":"slime_boss","act2":"bronze_automaton"}}));let mut w=mv.clone();w["config"]["boss_history"]=v["config"]["boss_history"].clone();assert!(import(&w).is_err());
 // Watcher semantic verbs are admitted by the M24 codec (shared Watcher action shape); Divinity is never a choice verb.
 let base={let mut t=RunSession::new_m24(config()).unwrap();t.reset(Some(7)).unwrap();t.record().unwrap().unwrap()};
 for a in [CombatAction::DeselectCard{instance:4},CombatAction::ChooseStance{stance:Stance::Wrath},CombatAction::ChooseStance{stance:Stance::Calm},CombatAction::ConfirmSelection,CombatAction::SelectCard{instance:4}]{let mut x=base.clone();x.steps.push(RunRecordedStep{action:GameAction::Combat(a.clone()),reward:0,terminal:None});let wv=x.to_json().unwrap_or_else(|e|panic!("{a:?}: {e:?}"));assert_eq!(RunEpisodeRecord::from_json(&wv).unwrap(),x);let mut j:Value=serde_json::from_slice(&wv).unwrap();j["steps"][0]["action"]["hidden"]=json!(1);assert!(import(&j).is_err());}
 let mut x=base.clone();x.steps.push(RunRecordedStep{action:GameAction::Combat(CombatAction::ChooseStance{stance:Stance::Divinity}),reward:0,terminal:None});assert!(x.to_json().is_err(),"Divinity is never a choice verb");
 // Typed export and raw import share capacity/structured-error validation.
 let mut t=base.clone();t.fault=Some(RunRecordedFault{action:None,committed:false,gameplay_recorded:false,error:RunRecordError{kind:RunRecordErrorKind::Capacity,message:"Run capacity exceeded".into(),capacity:None}});assert!(t.to_json().is_err());
 let mut ok=base.clone();ok.fault=Some(RunRecordedFault{action:None,committed:false,gameplay_recorded:false,error:RunRecordError{kind:RunRecordErrorKind::Capacity,message:"Run capacity exceeded".into(),capacity:Some(RunCapacityFailure{group:"map_nodes".into(),required:3,capacity:2})}});let okv=wire(&ok);assert_eq!(import(&okv).unwrap(),ok);
 for (field,value) in [("group",json!("hidden_rng")),("required",json!(2)),("capacity",json!(-1))]{let mut w=okv.clone();w["fault"]["error"]["capacity"][field]=value;assert!(import(&w).is_err(),"{field}");}
}

#[test]
fn m24_lifecycle_and_cutoffs(){
 let mut s=RunSession::new_m24(config()).unwrap();assert_eq!(s.decision(),Err(RunSessionError::NotReset));assert_eq!(s.step(RunDecisionToken::from([0,0]),RunActionIndex(0)),Err(RunSessionError::NotReset));assert!(s.record().unwrap().is_none());
 let first=s.reset(Some(7)).unwrap();let r0=s.record().unwrap().unwrap();assert_eq!((r0.seed,r0.ordinal),(7,0));
 let mut again=RunSession::new_m24(config()).unwrap();assert_eq!(again.reset(Some(7)).unwrap().observation,first.observation,"ordinal0 reproducible");
 let second=s.reset(None).unwrap();let r1=s.record().unwrap().unwrap();assert_eq!((r1.seed,r1.ordinal),(7,1));assert_ne!(first.token,second.token);assert_eq!(s.effective_seed(),Some(7));
 let mut peer=RunSession::new_m24(config()).unwrap();let pb=peer.reset(Some(7)).unwrap();let n=latest(&s).candidates.len() as i64;let before=s.reproduction().unwrap();
 for (token,i,want) in [(first.token,0,Some(RunSessionError::StaleToken)),(pb.token,0,Some(RunSessionError::StaleToken)),(second.token,-1,None),(second.token,n,None),(second.token,i64::MAX,None)]{let e=s.step(token,RunActionIndex(i)).unwrap_err();if let Some(want)=want{assert_eq!(e,want);}else{assert!(matches!(e,RunSessionError::Input(_)));}assert_eq!(s.reproduction().unwrap(),before);assert_eq!(s.preflight_step(token,RunActionIndex(i)).unwrap_err(),e);}
 // Actual normal Death from the retained M22/M23 Act1 transcript replayed in M24 at ordinal0 (the exact M23 prefix).
 let mut d=RunSession::new_m24(config()).unwrap();d.reset(Some(7)).unwrap();let steps=drive(&mut d,&transcript(M22_SEED7_GUARDIAN));let last=steps.last().unwrap();
 assert!(steps[..steps.len()-1].iter().all(|x|!x.terminated));assert!(last.terminated&&!last.truncated);assert_eq!(last.boundary.terminal,Some(RunTerminal::Death));assert!(last.boundary.observation.action_mask.iter().all(|m|*m==0));assert_eq!(d.decision().unwrap(),last.boundary);
 assert_eq!(d.step(last.boundary.token,RunActionIndex(0)),Err(RunSessionError::Terminal));let rec=d.record().unwrap().unwrap();assert_eq!((rec.terminal,rec.steps.len()),(Some(RunTerminal::Death),146));
 d.reset(None).unwrap();assert_eq!(d.record().unwrap().unwrap().ordinal,1);d.close().unwrap();assert_eq!(d.decision(),Err(RunSessionError::Closed));assert!(d.record().unwrap().unwrap().closed);assert!(d.reproduction().unwrap().closed);assert_eq!(d.reset(None),Err(RunSessionError::Closed));
 // Sidecar and environment metadata.
 let mut t=RunSession::new_m24(config()).unwrap();assert_eq!(t.combat_observation_spec().err(),Some(RunSessionError::UnsupportedBoundary("M24 does not provide the Ironclad combat policy sidecar")));t.reset(Some(7)).unwrap();assert_eq!(t.combat_view().err(),Some(RunSessionError::UnsupportedBoundary("M24 does not provide the Ironclad combat policy sidecar")));
 let env=t.environment_spec().unwrap();assert_eq!(env["profile_id"],"M24WatcherAct3");assert!(env["combat_view_id"].is_null());assert_eq!(env["rules_id"],"spire-gym-native-m24-rules-v1");assert_eq!(env["content_id"],"spire-gym-m24-watcher-a0-acts1-3-captured-v1");
 assert_eq!(env["finite_content"],spec()["semantics"]["finite_content"]);assert!(env["cutoff"]["act4_ready"].as_str().unwrap().contains("before the Act4 door"));
 // UNMET in normal play: no retained transcript reaches the Act3 boss; the healthy Act4Ready publication, step-time cutoff and Act3NoKeys
 // terminal are covered on constructed ending stages in run_session.rs (m24_lifecycle_and_cutoffs_on_constructed_ending) and run_batch.rs.
}

#[test]
fn m24_semantic_replay(){
 // Retained real Act1 transcript (Neow, map, rewards, shop/event, Scry Confirm0 and StancePotion Calm) continuing into the M24 Act2 Map.
 let mut s=RunSession::new_m24(cfg(RunAct1BossTarget::SlimeBoss,RunAct2BossTarget::Collector,RunAct3BossTarget::DonuAndDeca)).unwrap();s.reset(Some(7)).unwrap();let steps=drive(&mut s,&transcript(M22_SEED7_SLIME_BOSS));
 let r=s.record().unwrap().unwrap();assert!(r.steps.iter().any(|x|matches!(x.action,GameAction::Combat(CombatAction::ChooseStance{stance:Stance::Calm}))));assert!(r.steps.iter().any(|x|x.action==GameAction::Combat(CombatAction::ConfirmSelection)));
 let back=RunEpisodeRecord::from_json(&r.to_json().unwrap()).unwrap();assert_eq!(back,r);let report=replay_run(&back,caps()).unwrap();
 assert_eq!(report.final_decision,latest(&s));assert_eq!(report.final_map,s.reproduction().unwrap().latest_map);assert_eq!(report.accepted_count,273);assert_eq!(report.rewards,steps.iter().map(|x|x.reward).collect::<Vec<_>>());assert!(report.terminals.iter().all(|t|t.is_none()));assert!(report.final_decision_error.is_none());
 assert_eq!((report.final_decision.act,report.final_decision.floor),(2,17));
 // Implicit reset ordinal1 record reconstructs directly at ordinal1 (fixed passive public choices, not a policy).
 let mut t=RunSession::new_m24(config()).unwrap();t.reset(Some(11)).unwrap();t.reset(None).unwrap();for _ in 0..40{let d=latest(&t);if d.terminal.is_some(){break}act(&mut t,passive(&d));}
 let r1=t.record().unwrap().unwrap();assert_eq!((r1.seed,r1.ordinal),(11,1));let rep=replay_run(&RunEpisodeRecord::from_json(&r1.to_json().unwrap()).unwrap(),caps()).unwrap();assert_eq!(rep.final_decision,latest(&t));assert_eq!(rep.accepted_count,r1.steps.len());
 // The numeric index/token is never reused: a semantic action that is not currently offered fails at the right boundary.
 let mut wrong=r.clone();wrong.steps[1].action=GameAction::Move(NodeId(999_999));assert_eq!(replay_run(&wrong,caps()).err(),Some(RunSessionError::IncompatibleReplay("Run action not currently offered")));
 // Falling, Tomb, ending, Scry, Meditate and stance replays on constructed M24 stages: run_session.rs m24_semantic_replay_on_constructed_stages.
}

#[test]
fn m24_replay_preflight(){
 let mut s=RunSession::new_m24(config()).unwrap();s.reset(Some(7)).unwrap();drive(&mut s,&transcript(M22_SEED7_GUARDIAN)[..20]);let r=s.record().unwrap().unwrap();replay_run(&r,caps()).unwrap();
 let (g,c,a)=(RunAct1BossTarget::Guardian,RunAct2BossTarget::Champ,RunAct3BossTarget::AwakenedOne);
 let mutate:Vec<Box<dyn Fn(&mut RunEpisodeRecord)>>=vec![Box::new(|x|x.schema_id=RUN_RECORD_SCHEMA_ID_M23.into()),Box::new(|x|x.profile_id="M23WatcherAct2".into()),Box::new(|x|x.rules_id=RUN_RULES_ID_M23.into()),Box::new(|x|x.content_id=RUN_CONTENT_ID_M23.into()),Box::new(|x|x.rng_id="other-rng".into()),Box::new(|x|x.reward_id="other-reward".into()),Box::new(|x|x.seed_adapter_id="other-seed".into()),
  Box::new(|x|x.observation_fingerprint[0]^=1),Box::new(|x|x.observation_fingerprint=m23_run_observation_spec(&caps()).unwrap().fingerprint()),Box::new(|x|x.observation_fingerprint=m19_run_observation_spec(&caps()).unwrap().fingerprint()),
  Box::new(move|x|x.config.boss_history=RunBossHistory::M23WatcherAct2{act1:g,act2:c}),Box::new(move|x|x.config.boss_history=RunBossHistory::M19DefectAct3{act1:g,act2:c,act3:a}),Box::new(|x|x.config.boss_history=RunBossHistory::Act1GuardianUnseen),Box::new(|x|x.config.character=Character::Silent),Box::new(|x|x.config.ascension=1),Box::new(|x|x.config.capacities.actions=1)];
 for (i,m) in mutate.iter().enumerate(){let mut x=r.clone();m(&mut x);assert!(matches!(replay_run(&x,caps()),Err(RunSessionError::IncompatibleReplay(_))),"identity mutation {i}");assert!(x.to_json().is_err(),"export mutation {i}");}
 // Altered producer/capture/Act3 metadata changes the recomputed descriptor fingerprint.
 let spec=m24_run_observation_spec(&caps()).unwrap();for (path,value) in [("/semantics/finite_content/producers/shop_relic_price",json!(1)),("/semantics/finite_content/capture_law/scope/act3",json!("run.m23.relic.capture")),("/semantics/finite_content/status_cards",json!(["dazed","burn","slimed","wound"])),("/semantics/finite_content/relic_taxonomy/18/tier",json!("rare"))]{let mut d=spec.descriptor();*d.pointer_mut(path).unwrap()=value;let mut h=Sha256::new();h.update(b"spire-gym/run-schema/m24-watcher-act3/v1\0");h.update(canonical_json(&d).unwrap());assert_ne!(<[u8;32]>::from(h.finalize()),spec.fingerprint(),"{path}");}
 // Well-formed but altered action/reward/outcome fails against actual execution.
 let mut x=r.clone();x.steps[5].reward+=1;assert_eq!(replay_run(&x,caps()).err(),Some(RunSessionError::IncompatibleReplay("Run reward or terminal mismatch")));
 let mut x=r.clone();x.steps[19].terminal=Some(RunTerminal::Death);x.terminal=Some(RunTerminal::Death);assert!(replay_run(&x,caps()).is_err());
 let mut x=r.clone();x.steps[2].action=GameAction::Combat(CombatAction::PlayCard{instance:999,target:None});assert_eq!(replay_run(&x,caps()).err(),Some(RunSessionError::IncompatibleReplay("Run action not currently offered")));
 let mut alt=r.clone();alt.steps.clear();alt.seed=8;alt.ordinal=3;replay_run(&alt,caps()).unwrap();assert_eq!(RunEpisodeRecord::from_json(&alt.to_json().unwrap()).unwrap(),alt);
}

#[test]
fn m24_delivery_fault_record(){
 // Postaccept publication capacity failure: committed prefix, serial and first structured error retained; no empty success.
 let mut c=config();c.capacities.map_nodes=1;let mut s=RunSession::new_m24(c.clone()).unwrap();let b=s.reset(Some(7)).unwrap();let error=s.step(b.token,RunActionIndex(0)).unwrap_err();assert_eq!(error,RunSessionError::DeliveryFault("post-step Run encoding failed"));
 let r=s.record().unwrap().unwrap();assert_eq!(r.steps.len(),1);let f=r.fault.as_ref().unwrap();assert!(f.committed&&f.gameplay_recorded);assert_eq!(f.error.kind,RunRecordErrorKind::Capacity);assert_eq!(f.error.capacity.as_ref().unwrap().group,"map_nodes");assert_eq!(f.action.as_ref(),Some(&r.steps[0].action));
 let decoded=RunEpisodeRecord::from_json(&r.to_json().unwrap()).unwrap();assert_eq!(decoded,r);let report=replay_run(&decoded,c.capacities.clone()).unwrap();assert_eq!(report.accepted_count,1);assert_eq!(report.reported_fault,r.fault);assert!(report.final_decision_error.is_none());assert_eq!(report.final_decision,latest(&s));
 let before=s.reproduction().unwrap();assert!(matches!(before.fault,Some(RunSessionError::DeliveryFault(_))));assert!(s.reset(None).is_err());assert!(s.step(b.token,RunActionIndex(0)).is_err());assert!(s.decision().is_err());s.fail_delivery("later").unwrap();assert_eq!(s.reproduction().unwrap(),before);s.close().unwrap();assert!(s.record().unwrap().unwrap().closed);assert!(RunEpisodeRecord::from_json(&s.record().unwrap().unwrap().to_json().unwrap()).is_ok());
 // Startup publication failure on reset: no accepted gameplay, first fault retained.
 let mut c2=config();c2.capacities.actions=1;let mut z=RunSession::new_m24(c2.clone()).unwrap();let e=z.reset(Some(7)).unwrap_err();assert_eq!(e,RunSessionError::Capacity{group:"actions",required:2,capacity:1});let zr=z.record().unwrap().unwrap();assert!(zr.steps.is_empty());let zf=zr.fault.as_ref().unwrap();assert!(!zf.committed&&!zf.gameplay_recorded);
 let zb=RunEpisodeRecord::from_json(&zr.to_json().unwrap()).unwrap();assert_eq!(zb,zr);let zrep=replay_run(&zb,c2.capacities.clone()).unwrap();assert_eq!(zrep.accepted_count,0);assert_eq!(zrep.reported_fault,zr.fault);
 // Transport-only loss after accepted actions: reported diagnostic, no fabricated core action.
 let mut t=RunSession::new_m24(config()).unwrap();t.reset(Some(7)).unwrap();drive(&mut t,&transcript(M22_SEED7_SLIME_BOSS)[..4]);t.fail_delivery("python acknowledgement lost").unwrap();let tr=t.record().unwrap().unwrap();let tf=tr.fault.as_ref().unwrap();assert!(tf.committed&&tf.gameplay_recorded);assert_eq!(tf.error.kind,RunRecordErrorKind::DeliveryFault);assert_eq!(tr.steps.len(),4);
 let rep=replay_run(&RunEpisodeRecord::from_json(&tr.to_json().unwrap()).unwrap(),caps()).unwrap();assert_eq!(rep.accepted_count,4);assert_eq!(rep.final_decision,latest(&t));assert_eq!(rep.reported_fault,tr.fault);assert!(t.reset(Some(7)).is_err(),"faulted session cannot reset into a new loss");
 // Core ending/capture faults (run.m24.act3.end) on constructed stages: run_session.rs m24_delivery_fault_record_with_actual_core_faults.
}

fn reproductions(b:&RunBatch,n:usize)->Vec<RunReproduction>{(0..n).map(|i|b.reproduction(i).unwrap()).collect()}
fn indices(singles:&[RunSession],prefixes:&[Vec<GameAction>],i:usize)->Vec<RunActionIndex>{singles.iter().zip(prefixes).map(|(s,p)|index(s,&p[i])).collect()}
fn same_step(a:&RunStepResult,b:&RunStepResult){assert_eq!((&a.boundary.observation,a.boundary.terminal,a.reward,a.terminated,a.truncated),(&b.boundary.observation,b.boundary.terminal,b.reward,b.terminated,b.truncated));}

#[test]
fn m24_batch_order_matches_singles(){
 let configs=vec![cfg(RunAct1BossTarget::SlimeBoss,RunAct2BossTarget::Collector,RunAct3BossTarget::TimeEater),cfg(RunAct1BossTarget::Guardian,RunAct2BossTarget::BronzeAutomaton,RunAct3BossTarget::DonuAndDeca)];let seeds=[Some(7),Some(11)];
 let prefixes=[transcript(M22_SEED7_SLIME_BOSS)[..60].to_vec(),transcript(M22_SEED11_GUARDIAN)[..60].to_vec()];
 for workers in [1,2]{
  let mut b=RunBatch::new_m24(configs.clone(),workers).unwrap();let mut singles=configs.iter().map(|c|RunSession::new_m24(c.clone()).unwrap()).collect::<Vec<_>>();
  let bounds=b.reset(&seeds).unwrap();for (slot,s) in singles.iter_mut().enumerate(){let one=s.reset(seeds[slot]).unwrap();assert_eq!((&bounds[slot].observation,bounds[slot].terminal),(&one.observation,one.terminal),"input-slot order");}
  let mut tokens=bounds.iter().map(|x|x.token).collect::<Vec<_>>();
  for i in 0..60{let idx=indices(&singles,&prefixes,i);let rows=b.step(&tokens,&idx).unwrap();for (slot,s) in singles.iter_mut().enumerate(){let t=s.decision().unwrap().token;let one=s.step(t,idx[slot]).unwrap();same_step(&rows[slot],&one);}tokens=rows.iter().map(|x|x.boundary.token).collect();}
  for (slot,s) in singles.iter().enumerate(){assert_eq!(b.record(slot).unwrap(),s.record().unwrap());assert_eq!(b.reproduction(slot).unwrap(),s.reproduction().unwrap());}
  assert!((0..2).all(|slot|b.reproduction(slot).unwrap().latest_decision.unwrap().unwrap().content_profile==Some(RoomScenario::M24WatcherAct3)));
 }
 // Indexed constructor rejection precedes any reset; no mixed-profile constructor.
 assert!(matches!(RunBatch::new_m24(vec![],1),Err(RunBatchError::Input{slot:None,..})));assert!(matches!(RunBatch::new_m24(vec![config()],0),Err(RunBatchError::Input{slot:None,..})));
 assert!(matches!(RunBatch::new_m24(vec![config(),RunSessionConfig{character:Character::Silent,..config()}],1),Err(RunBatchError::Input{slot:Some(1),..})));
 assert!(matches!(RunBatch::new_m24(vec![m23_cfg(RunAct1BossTarget::Guardian,RunAct2BossTarget::Champ),config()],2),Err(RunBatchError::Input{slot:Some(0),..})));
 assert!(matches!(RunBatch::new_m23(vec![config()],1),Err(RunBatchError::Input{slot:Some(0),..})),"M23 batch rejects the M24 triple");
}

#[test]
fn m24_batch_preflight_full_reset(){
 for workers in [1,2]{
  let configs=vec![cfg(RunAct1BossTarget::SlimeBoss,RunAct2BossTarget::Champ,RunAct3BossTarget::AwakenedOne),cfg(RunAct1BossTarget::Guardian,RunAct2BossTarget::Collector,RunAct3BossTarget::TimeEater)];let prefixes=[transcript(M22_SEED7_SLIME_BOSS),transcript(M22_SEED7_GUARDIAN)];
  let mut b=RunBatch::new_m24(configs.clone(),workers).unwrap();let mut singles=configs.iter().map(|c|RunSession::new_m24(c.clone()).unwrap()).collect::<Vec<_>>();let first=b.reset(&[Some(7),Some(7)]).unwrap();for s in &mut singles{s.reset(Some(7)).unwrap();}
  let mut tokens=first.iter().map(|x|x.token).collect::<Vec<_>>();let before=reproductions(&b,2);
  assert!(matches!(b.step(&tokens[..1],&[RunActionIndex(0)]),Err(RunBatchError::Input{slot:None,..})));assert!(matches!(b.reset(&[Some(7)]),Err(RunBatchError::Input{slot:None,..})));
  let idx=indices(&singles,&prefixes,0);assert_eq!(b.step(&[tokens[0],RunDecisionToken::from([0,0])],&idx),Err(RunBatchError::Input{slot:Some(1),error:RunSessionError::StaleToken}));
  assert!(matches!(b.step(&tokens,&[idx[0],RunActionIndex(999)]),Err(RunBatchError::Input{slot:Some(1),error:RunSessionError::Input(_)})));assert!(matches!(b.step(&tokens,&[idx[0],RunActionIndex(-1)]),Err(RunBatchError::Input{slot:Some(1),..})));
  assert_eq!(reproductions(&b,2),before,"no peer action, RNG, ordinal or serial change");
  // Drive both to the actual normal Death of the last slot; then a healthy peer plus the terminal slot rejects the whole call.
  for i in 0..146{let idx=indices(&singles,&prefixes,i);let rows=b.step(&tokens,&idx).unwrap();for (slot,s) in singles.iter_mut().enumerate(){let t=s.decision().unwrap().token;s.step(t,idx[slot]).unwrap();}tokens=rows.iter().map(|x|x.boundary.token).collect();}
  assert_eq!(b.reproduction(1).unwrap().latest_decision.unwrap().unwrap().terminal,Some(RunTerminal::Death));let before=reproductions(&b,2);
  assert_eq!(b.step(&tokens,&[index(&singles[0],&prefixes[0][146]),RunActionIndex(0)]),Err(RunBatchError::Input{slot:Some(1),error:RunSessionError::Terminal}));assert_eq!(reproductions(&b,2),before,"no terminal auto-skip");
  // Full reset only: per-slot explicit/implicit seeds keep each ordinal law.
  b.reset(&[None,Some(13)]).unwrap();assert_eq!([b.record(0).unwrap().unwrap(),b.record(1).unwrap().unwrap()].map(|r|(r.seed,r.ordinal)),[(7,1),(13,0)]);
  b.reset(&[Some(5),None]).unwrap();assert_eq!([b.record(0).unwrap().unwrap(),b.record(1).unwrap().unwrap()].map(|r|(r.seed,r.ordinal)),[(5,0),(13,1)]);
  let mut alone=RunSession::new_m24(configs[1].clone()).unwrap();alone.reset(Some(13)).unwrap();alone.reset(None).unwrap();assert_eq!(b.reproduction(1).unwrap(),alone.reproduction().unwrap());
 }
}

#[test]
fn m24_batch_postdispatch_faults(){
 for workers in [1,2]{
  // Step: slot1 commits then fails publication; slot0 commits and its row is indexed delivery_lost.
  let mut small=config();small.capacities.map_nodes=1;let mut b=RunBatch::new_m24(vec![config(),small],workers).unwrap();let bounds=b.reset(&[Some(7);2]).unwrap();let tokens=bounds.iter().map(|r|r.token).collect::<Vec<_>>();
  let RunBatchError::Fault{slots}=b.step(&tokens,&[RunActionIndex(0);2]).unwrap_err() else{panic!("expected post-dispatch fault")};assert_eq!(slots.len(),2);
  assert_eq!((slots[0].slot,slots[0].committed,slots[0].delivery_lost),(0,true,true));assert_eq!((slots[1].slot,slots[1].committed,slots[1].delivery_lost),(1,true,false));assert_eq!(slots[1].error,RunSessionError::DeliveryFault("post-step Run encoding failed"));
  let (r0,r1)=(b.record(0).unwrap().unwrap(),b.record(1).unwrap().unwrap());assert_eq!((r0.steps.len(),r1.steps.len()),(1,1));assert_eq!(r1.fault.as_ref().unwrap().error.capacity.as_ref().unwrap().group,"map_nodes");assert_eq!(r0.fault.as_ref().unwrap().error.kind,RunRecordErrorKind::DeliveryFault);
  for r in [&r0,&r1]{let back=RunEpisodeRecord::from_json(&r.to_json().unwrap()).unwrap();assert_eq!(&back,r);assert_eq!(replay_run(&back,r.config.capacities.clone()).unwrap().accepted_count,1);}
  let before=reproductions(&b,2);assert!(b.step(&tokens,&[RunActionIndex(0);2]).is_err());assert!(b.reset(&[Some(7);2]).is_err());assert_eq!(reproductions(&b,2),before,"retry cannot replay either action");b.close().unwrap();assert!(b.record(0).unwrap().unwrap().closed&&b.record(1).unwrap().unwrap().closed);
  // Reset: slot1 startup publication fails; slot0's reset row is delivery_lost without accepted gameplay.
  let mut tiny=config();tiny.capacities.actions=1;let mut b=RunBatch::new_m24(vec![config(),tiny],workers).unwrap();let RunBatchError::Fault{slots}=b.reset(&[Some(7);2]).unwrap_err() else{panic!("expected reset fault")};
  assert_eq!((slots[0].committed,slots[0].delivery_lost,slots[1].committed,slots[1].delivery_lost),(false,true,false,false));assert_eq!(slots[1].error,RunSessionError::Capacity{group:"actions",required:2,capacity:1});
  assert!(b.record(0).unwrap().unwrap().steps.is_empty());assert!(b.record(1).unwrap().unwrap().fault.is_some());assert!(b.reset(&[Some(7);2]).is_err());
 }
 // Actual core faults, worker panics and reservation failures on constructed M24 stages: run_batch.rs m24_actual_batch_tests.
}

fn m15_config()->RunSessionConfig{RunSessionConfig{character:Character::Silent,ascension:0,partition:Partition::Test,boss_history:RunBossHistory::M15SilentHeart{act1:RunAct1BossTarget::Guardian,act2:RunAct2BossTarget::Champ,act3:RunAct3BossTarget::AwakenedOne},capacities:caps(),record_replay:true}}
fn m20_config()->RunSessionConfig{RunSessionConfig{character:Character::Defect,ascension:0,partition:Partition::Test,boss_history:RunBossHistory::M20DefectHeart{act1:RunAct1BossTarget::SlimeBoss,act2:RunAct2BossTarget::Collector,act3:RunAct3BossTarget::DonuAndDeca},capacities:caps(),record_replay:true}}
type Lane=(&'static str,fn(RunSessionConfig)->Result<RunSession,RunSessionError>,fn(Vec<RunSessionConfig>,usize)->Result<RunBatch,RunBatchError>,RunSessionConfig,Vec<GameAction>,[i32;32]);
fn lanes()->Vec<Lane>{vec![
 ("M24",RunSession::new_m24,RunBatch::new_m24,cfg(RunAct1BossTarget::SlimeBoss,RunAct2BossTarget::Collector,RunAct3BossTarget::DonuAndDeca),transcript(M22_SEED7_SLIME_BOSS)[..32].to_vec(),M22_SLIME_PREFIX_REWARDS),
 ("M23",RunSession::new_m23,RunBatch::new_m23,m23_cfg(RunAct1BossTarget::SlimeBoss,RunAct2BossTarget::Collector),transcript(M22_SEED7_SLIME_BOSS)[..32].to_vec(),M22_SLIME_PREFIX_REWARDS),
 ("M15",RunSession::new_m15,RunBatch::new_m15,m15_config(),transcript(M15_SEED7_PREFIX),M15_PREFIX_REWARDS),
 ("M20",RunSession::new_m20,RunBatch::new_m20,m20_config(),transcript(M20_SEED7_PREFIX),M20_PREFIX_REWARDS)]}

#[test]
fn m24_coexistence(){
 // Un-interleaved references from existing retained recorded actions (no chooser, no fresh historical episode).
 let mut reference=Vec::new();
 for (name,new,_,c,actions,rewards) in lanes(){let mut s=new(c).unwrap();s.reset(Some(7)).unwrap();let steps=drive(&mut s,&actions);assert_eq!(steps.iter().map(|x|x.reward).collect::<Vec<_>>(),rewards.to_vec(),"{name} retained rewards");
  assert!(s.record().unwrap().unwrap().steps.iter().any(|x|matches!(x.action,GameAction::Combat(_))),"{name} 32-action prefix crosses an actual combat boundary");assert!(steps.iter().all(|x|!x.terminated));reference.push((steps,s.record().unwrap(),s.reproduction().unwrap()));}
 // Interleaved single sessions (M24, M23, Silent M15, Defect M20), round-robin one accepted action each.
 let mut sessions=lanes().into_iter().map(|(_,new,_,c,actions,_)|{let mut s=new(c).unwrap();s.reset(Some(7)).unwrap();(s,actions)}).collect::<Vec<_>>();let mut got=vec![Vec::new();4];
 for i in 0..32{for (lane,(s,actions)) in sessions.iter_mut().enumerate(){got[lane].push(act(s,actions[i].clone()));}}
 for (lane,(s,_)) in sessions.iter().enumerate(){for (a,b) in got[lane].iter().zip(&reference[lane].0){same_step(a,b);}assert_eq!(s.record().unwrap(),reference[lane].1);assert_eq!(s.reproduction().unwrap(),reference[lane].2);}
 let spec=spec();let stance=col(&spec,"player","stance");assert!(got[0].iter().any(|x|cell(group(&x.boundary.observation,"player"),0,stance).1==1),"M24 Watcher stance column published in combat");
 // M24 and M23 observe the same Act1 game through different schemas; their records differ only in identity and history.
 assert_eq!(reference[0].0.iter().map(|x|x.reward).collect::<Vec<_>>(),reference[1].0.iter().map(|x|x.reward).collect::<Vec<_>>());assert_ne!(reference[0].1.as_ref().unwrap().observation_fingerprint,reference[1].1.as_ref().unwrap().observation_fingerprint);
 // Separate homogeneous batches, workers 1/2, agree with the same singles.
 for workers in [1,2]{for (lane,(_,new_single,new_batch,c,actions,_)) in lanes().into_iter().enumerate(){
  let mut b=new_batch(vec![c.clone(),c.clone()],workers).unwrap();let mut tokens=b.reset(&[Some(7);2]).unwrap().iter().map(|x|x.token).collect::<Vec<_>>();let mut single=new_single(c).unwrap();single.reset(Some(7)).unwrap();
  for (i,a) in actions.iter().enumerate(){let idx=index(&single,a);let rows=b.step(&tokens,&[idx,idx]).unwrap();let t=single.decision().unwrap().token;single.step(t,idx).unwrap();same_step(&rows[0],&reference[lane].0[i]);same_step(&rows[1],&reference[lane].0[i]);tokens=rows.iter().map(|x|x.boundary.token).collect();}
  assert_eq!(b.record(0).unwrap(),reference[lane].1);assert_eq!(b.record(1).unwrap(),reference[lane].1);}}
}
