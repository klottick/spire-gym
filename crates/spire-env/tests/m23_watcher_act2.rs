//! M23 finite Watcher A0 Acts1–2 native profile: descriptor, encoder, session, replay, batch and coexistence.
//! Constructed views are labelled; real-play checks use the canonical M23 initializer and fixed public choices only.
use spire_env::*;
use spire_core::{combat::{rng::Partition,CombatAction,Stance,TargetId},content::*,run::{*,diagnostics::{m23_diagnostic,m23_played_fixture,M23Diagnostic,M23DiagnosticBoss,M23PlayedFixture,M22DiagnosticBranch}}};
use serde_json::{json,Value};
use sha2::{Digest,Sha256};

// Retained pre-change bytes (evidence h1-v1/preservation/baseline-fingerprints-v1.txt).
const FROZEN:&[(&str,&str)]=&[
 ("m9","0893bce86e4b3abf11aa384c501503a91ae37f688ec605ab40b814ad365cb999"),("m10","7664874a7a483d1f2e0439ea7b319d1a4be4b9d276d0cb0f908654feda9393db"),
 ("m12","366c572af8141146995e849acc737a33484623790e2b3d661e290fadfb6c9fac"),("m13","3f1e21d50106b8ca71d7cd67a2de7bfa4074455dbbb459629d8e2ac24cf39cf8"),
 ("m14","50a685721e5db56ed7a5332160f3747087f96ce738da020415b1427159377aee"),("m15","254fb75b404fc558070468c6397324e68f962f23b33aca819b54fecf142ea11a"),
 ("m17","2176f835eca1384b2070f3d7d001158ef1fead23be5d6d7eea138b57f6bff999"),("m18","e097a8202f7cf7433f0ce57a39807bb74d370fba75a9aa84af65d726269564bb"),
 ("m19","149092e11432e182920b5f07e0728dac45e880296815d9b84fba9b6180c2817b"),("m20","8c03211b4a3c589cfb55ba1078ddd447a444f3f53d8e9bfdab9ab85ab80e512d"),
 ("m22","5d1c77658141b2dfefbe95e17b85899ad6b77fb370e402bcbbe771146821e087"),("policy","30905db1b7a520482d3ea8aa2e63ddb7f78b7c081de89d87268151f29cf4c7f8"),
 ("combat_v1","02009686282c592f971eece3d5853eb54f2a482dde9dae1d0cf380ecdaf8fd78"),("combat_v2","02433813e1297244060d9852f342c6eb4c9c4d62f3dde9763ff82191867eceec"),
 ("m21","e7e49e88d50c947fde064a764153a5846ba3dc6c8298116f03e8126383b51e34"),("m16","2b53f59fac1899e231300914b0b88aabc4e26add3492584de479b60e5fafce0a"),
];
// Design-table oracles (docs/superpowers/specs/2026-10-28-m23-watcher-act2-design.md), not the implementation.
const ACT2_INTENTS:&[&str]=&["sphere_activate","sphere_debuff","sphere_slam","sphere_harden","byrd_caw","byrd_peck","byrd_swoop","byrd_headbutt","byrd_stunned","byrd_go_airborne","centurion_slash","centurion_protect","centurion_fury","mystic_attack","mystic_buff","mystic_heal","chosen_poke","chosen_hex","chosen_debilitate","chosen_drain","chosen_zap","snecko_glare","snecko_bite","snecko_tail_whip","leader_rally","leader_encourage","leader_stab","gremlin_scratch","gremlin_puncture","gremlin_smash","gremlin_protect","gremlin_bash","gremlin_charge","gremlin_blast","slaver_stab","slaver_rake","slaver_scrape","slaver_entangle","taskmaster_whip","book_multi","book_big","champ_anger","champ_execute","champ_taunt","champ_stance","champ_gloat","champ_slap","champ_slash","automaton_spawn","automaton_flail","automaton_boost","automaton_beam","automaton_stun","orb_stasis","orb_support","orb_beam","collector_spawn","collector_revive","collector_rake","collector_buff","collector_ultimate","torch_attack"];
const ACT2_POWERS:&[&str]=&["artifact","barricade","flight","hex","confused","minion","angry","entangled","painful_stabs","stasis"];
const ACT2_CAPTURES:&[EncounterId]=&[EncounterId::SphericGuardian,EncounterId::ThreeByrds,EncounterId::CenturionAndHealer,EncounterId::Snecko,EncounterId::ChosenAndByrds,EncounterId::ThreeCultists,EncounterId::GremlinLeader,EncounterId::Slavers,EncounterId::BookOfStabbing,EncounterId::Champ,EncounterId::Collector,EncounterId::BronzeAutomaton];

fn hex(b:[u8;32])->String{b.iter().map(|b|format!("{b:02x}")).collect()}
fn caps()->RunCapacities{RunCapacities::m10_default()}
fn cfg(act1:RunAct1BossTarget,act2:RunAct2BossTarget)->RunSessionConfig{RunSessionConfig{character:Character::Watcher,ascension:0,partition:Partition::Test,boss_history:RunBossHistory::M23WatcherAct2{act1,act2},capacities:caps(),record_replay:true}}
fn config()->RunSessionConfig{cfg(RunAct1BossTarget::Guardian,RunAct2BossTarget::Champ)}
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
/// Actual M21-owned Watcher combat setup for an authored encounter. Relabelled only to drive the pure M23 encoder.
fn fixture(encounter:EncounterId,start:M21WatcherCombatStart)->Run{let mut root=[0;32];root[..8].copy_from_slice(&7u64.to_le_bytes());let mut r=Run::prepare_m21_watcher_combat(start,encounter,RunRng::new(root,Partition::Test,0)).unwrap();r.enter_training_combat(encounter).unwrap();r}
fn supplied(cards:&[CardId],potions:Vec<Option<PotionId>>)->M21WatcherCombatStart{M21WatcherCombatStart::Supplied(AuthoredRunStart{hp:72,max_hp:72,gold:99,deck:cards.iter().enumerate().map(|(i,id)|CardInstance{instance:31+i as u64,id:*id,upgrades:0,misc:0}).collect(),relics:vec![RelicId::PureWater],potions,relic_counters:Default::default()})}
/// Constructed (labelled) placement of an actual standalone combat view at an Act2 floor; gameplay values untouched.
fn relabel(r:&Run)->PublicDecision{let mut d=r.decision().unwrap();d.content_profile=Some(RoomScenario::M23WatcherAct2);d.act=2;d.floor=20;d}
fn play(r:&mut Run,kind:CardId)->bool{let d=r.decision().unwrap();let c=d.combat.as_ref().unwrap();let Some(a)=d.candidates.iter().find(|row|matches!(row.action,GameAction::Combat(CombatAction::PlayCard{instance,..}) if c.hand.iter().any(|card|card.instance.0==instance&&card.id==kind))).map(|c|c.action.clone()) else{return false};r.step(&a).unwrap();true}
fn end_turn(r:&mut Run)->bool{let d=r.decision().unwrap();if d.terminal.is_some()||!d.candidates.iter().any(|c|c.action==GameAction::Combat(CombatAction::EndTurn)){return false}r.step(&GameAction::Combat(CombatAction::EndTurn)).unwrap();true}
fn stance_code(s:Stance)->i64{match s{Stance::Neutral=>0,Stance::Wrath=>1,Stance::Calm=>2,Stance::Divinity=>3}}
fn m23_combat(seed:u64)->RunSession{let mut s=RunSession::new_m23(config()).unwrap();s.reset(Some(seed)).unwrap();let d=latest(&s);act(&mut s,d.candidates[0].action.clone());let d=latest(&s);act(&mut s,first_move(&d));assert!(latest(&s).combat.is_some(),"first Act1 node is an actual combat");s}

#[test]
fn m23_descriptor_covers_exact_reachable_union_and_old_profiles(){
 let spec=m23_run_observation_spec(&caps()).unwrap();let d=spec.descriptor();
 let mut h=Sha256::new();h.update(b"spire-gym/run-schema/m23-watcher-act2/v1\0");h.update(canonical_json(&d).unwrap());assert_eq!(spec.fingerprint(),<[u8;32]>::from(h.finalize()));
 assert_eq!((d["schema_id"].as_str(),d["profile_id"].as_str()),(Some(RUN_OBSERVATION_M23_SCHEMA_ID),Some("M23WatcherAct2")));assert_eq!(RUN_OBSERVATION_M23_SCHEMA_ID,"spire-gym-m23-watcher-act2-observation-v1");
 let names=|ids:&[CardId]|ids.iter().map(|id|json!(id.as_str())).collect::<Vec<_>>();
 let f=&d["semantics"]["finite_content"];
 assert_eq!(f["permanent_cards"].as_array().unwrap(),&names(M23_WATCHER_ACT2_CARDS));assert_eq!(M23_WATCHER_ACT2_CARDS.len(),28);
 assert_eq!(f["generated_cards"],json!(["miracle","smite","safety"]));assert_eq!(f["status_cards"],json!(["dazed","burn","slimed","wound"]));
 let mut physical=vec![json!("padding")];physical.extend(names(M23_WATCHER_ACT2_CARDS));physical.extend(names(m23_content::GENERATED_ONLY));physical.extend(names(m23_content::STATUSES));
 assert_eq!(d["vocabularies"]["card"].as_array().unwrap(),&physical);assert_eq!(physical.len(),1+35);
 assert_eq!(f["relics"].as_array().unwrap().len(),18);assert_eq!(f["potions"].as_array().unwrap().len(),7);assert_eq!(d["vocabularies"]["relic"].as_array().unwrap().len(),19);assert_eq!(d["vocabularies"]["potion"].as_array().unwrap().len(),8);
 assert_eq!(f["playable_encounters"].as_array().unwrap().len(),24);assert_eq!(f["playable_enemies"].as_array().unwrap().len(),37);assert_eq!(f["map_only_boss_icons"],json!(["awakened_one","time_eater","donu_and_deca"]));
 for e in ACT2_CAPTURES{assert!(f["playable_encounters"].as_array().unwrap().contains(&json!(e.as_str())),"{e:?}");vocab_id(&d,"encounter",e.as_str());}
 for name in ["awakened_one","time_eater","donu_and_deca"]{vocab_id(&d,"encounter",name);assert!(!f["playable_encounters"].as_array().unwrap().contains(&json!(name)));}
 for i in ACT2_INTENTS{vocab_id(&d,"intent",i);}for p in ACT2_POWERS.iter().chain(&["mantra","devotion","foresight","nirvana","establishment","master_reality","battle_hymn","mental_fortress","rushdown","like_water"]){vocab_id(&d,"power",p);}
 for ev in ["neow","upgrade_shrine","purifier","ancient_writing","joust"]{vocab_id(&d,"event",ev);}vocab_id(&d,"room","act3_ready");vocab_id(&d,"choice","ancient_simplicity");vocab_id(&d,"choice","joust_owner");
 assert!(!d["vocabularies"]["card"].as_array().unwrap().contains(&json!("parasite")),"no Parasite source in captured rooms");
 // Producer/capture metadata is a projection of the single content owners.
 let m22=m22_run_observation_spec(&caps()).unwrap().descriptor();assert_eq!(f["producers"],m22["semantics"]["finite_content"]["producers"]);assert_eq!(f["producers"]["owner"],"spire_core::run::m22_content");
 assert_eq!(f["capture_law"]["scope"],json!({"act1":m22_content::RELIC_CAPTURE_SCOPE_NAME,"act2":m23_content::ACT2_RELIC_CAPTURE_SCOPE_NAME}));assert_eq!(f["capture_law"]["scope"]["act2"],"run.m23.relic.capture");assert_eq!(f["capture_law"]["key_fields"],m22["semantics"]["finite_content"]["capture_law"]["key_fields"]);
 assert!(d["semantics"]["cutoff"].as_str().unwrap().contains("Act3Ready"));
 // Every captured Act2 setup is representable from the actual setup/AI owners: initial, two later turns, summons/held/status rows.
 let mut stasis=0;let mut wound=0;let mut seen_enemies=std::collections::BTreeSet::new();
 for &e in ACT2_CAPTURES{let mut r=fixture(e,M21WatcherCombatStart::Canonical);for _turn in 0..4{let dd=relabel(&r);let o=encode_run_m23(&dd,None,&caps()).unwrap_or_else(|err|panic!("{e:?}: {err:?}"));let Some(c)=dd.combat.as_ref() else{break};
  for (row,en) in c.enemies.iter().enumerate(){seen_enemies.insert(en.enemy_id.as_str());assert_eq!(cell(group(&o,"enemies"),row,0).0,vocab_id(&d,"enemy",en.enemy_id.as_str()));}
  stasis+=c.stasis.len();wound+=c.hand.iter().chain(&c.discard).filter(|x|x.id==CardId::Wound).count();if !end_turn(&mut r){break}}}
 for actor in ["torch_head","bronze_orb","gremlin_leader","taskmaster","mystic","byrd","chosen","snecko"]{assert!(seen_enemies.contains(actor),"{actor} actually produced");}
 assert!(stasis>0,"actual Bronze Orb Stasis custody encoded");assert!(wound>0,"actual Act2 Wound encoded");
 // Foreign authored setups outside the captured union fail closed when they name an unknown actor.
 let mut foreign=0;for (id,_,_) in spire_core::combat::enemies::A0_ENCOUNTER_SETUPS.iter(){if f["playable_encounters"].as_array().unwrap().contains(&json!(id.as_str())){continue}let r=fixture(*id,M21WatcherCombatStart::Canonical);let dd=relabel(&r);if dd.combat.as_ref().unwrap().enemies.iter().any(|en|!f["playable_enemies"].as_array().unwrap().contains(&json!(en.enemy_id.as_str()))){assert!(encode_run_m23(&dd,None,&caps()).is_err(),"{id:?}");foreign+=1;}}
 assert!(foreign>0);
 // Old descriptors stay byte-identical.
 let c=caps();let p=combat_policy_capacities();
 let now=[("m9",run_observation_spec(&RunCapacities::m9_default()).unwrap().fingerprint()),("m10",m10_run_observation_spec(&c).unwrap().fingerprint()),("m12",m12_run_observation_spec(&c).unwrap().fingerprint()),("m13",m13_run_observation_spec(&c).unwrap().fingerprint()),("m14",m14_run_observation_spec(&c).unwrap().fingerprint()),("m15",m15_run_observation_spec(&c).unwrap().fingerprint()),("m17",m17_run_observation_spec(&c).unwrap().fingerprint()),("m18",m18_run_observation_spec(&c).unwrap().fingerprint()),("m19",m19_run_observation_spec(&c).unwrap().fingerprint()),("m20",m20_run_observation_spec(&c).unwrap().fingerprint()),("m22",m22_run_observation_spec(&c).unwrap().fingerprint()),("policy",policy_run_observation_spec(&c).unwrap().fingerprint()),("combat_v1",combat_policy_observation_spec().unwrap().fingerprint()),("combat_v2",combat_policy_observation_spec_v2().unwrap().fingerprint()),("m21",watcher_combat_observation_spec(&p).unwrap().fingerprint()),("m16",defect_combat_observation_spec(&p).unwrap().fingerprint())];
 for ((a,fp),(b,want)) in now.iter().zip(FROZEN){assert_eq!(a,b);assert_eq!(hex(*fp),*want,"{a} descriptor changed");}
 assert_ne!(hex(spec.fingerprint()),FROZEN[10].1);
}

#[test]
fn m23_encoding_uses_current_watcher_and_stasis_state(){
 let spec=m23_run_observation_spec(&caps()).unwrap().descriptor();let (stance,mantra)=(col(&spec,"player","stance"),col(&spec,"player","mantra_gained"));
 // Real M23 normal play: canonical Eruption/Vigilance change the current stance.
 let mut s=m23_combat(7);let mut changed=0;
 for _ in 0..6{let d=latest(&s);let c=d.combat.as_ref().unwrap();let o=s.decision().unwrap().observation;assert_eq!(cell(group(&o,"player"),0,stance),(stance_code(c.character.stance),1));
  assert_eq!(o.action_mask.iter().filter(|m|**m==1).count(),d.candidates.len());assert!(o.action_mask[..d.candidates.len()].iter().all(|m|*m==1));
  let card=c.hand.iter().find(|x|matches!(x.id,CardId::Eruption|CardId::Vigilance)&&d.candidates.iter().any(|row|matches!(row.action,GameAction::Combat(CombatAction::PlayCard{instance,..}) if instance==x.instance.0)));
  if let Some(card)=card{let a=d.candidates.iter().find(|row|matches!(row.action,GameAction::Combat(CombatAction::PlayCard{instance,..}) if instance==card.instance.0)).unwrap().action.clone();let want=if card.id==CardId::Eruption{1}else{2};act(&mut s,a);if latest(&s).combat.is_none(){break}let o=s.decision().unwrap().observation;assert_eq!(cell(group(&o,"player"),0,stance).0,want);changed+=1;}
  else{act(&mut s,GameAction::Combat(CombatAction::EndTurn));if latest(&s).combat.is_none(){break}}}
 assert!(changed>0,"a canonical stance card was played in real M23 combat");
 // Mantra/Divinity: current stance and Mantra power vs gained history (actual M21-owned Watcher combat, relabelled).
 let mut r=fixture(EncounterId::JawWorm,supplied(&[CardId::Worship;10],vec![None;3]));assert!(play(&mut r,CardId::Worship));let d=relabel(&r);let o=encode_run_m23(&d,None,&caps()).unwrap();let c=d.combat.as_ref().unwrap();
 assert_eq!(cell(group(&o,"player"),0,mantra).0,i64::from(c.mantra_gained.unwrap()));let pk=vocab_id(&spec,"power","mantra");let p=group(&o,"powers");
 let mantra_now=c.player.powers.iter().find(|x|x.id.0=="mantra").map(|x|x.amount);let rows=(0..p.shape[0]).filter(|i|p.occupied.as_ref().unwrap()[*i]==1&&cell(p,*i,2).0==pk).collect::<Vec<_>>();
 assert_eq!(rows.iter().map(|i|cell(p,*i,3).0).collect::<Vec<_>>(),mantra_now.map(i64::from).into_iter().collect::<Vec<_>>());
 assert!(end_turn(&mut r));assert!(play(&mut r,CardId::Worship));let d=relabel(&r);let o=encode_run_m23(&d,None,&caps()).unwrap();let c=d.combat.as_ref().unwrap();assert_eq!(c.character.stance,Stance::Divinity);
 assert_eq!(cell(group(&o,"player"),0,stance).0,3);assert_eq!(cell(group(&o,"player"),0,mantra).0,i64::from(c.mantra_gained.unwrap()));assert!(c.mantra_gained.unwrap()>=10);
 // Current printed vs combat/turn prices and Retain are the public card values.
 let cc=group(&o,"combat_cards");let (eff,cost,turn,ret)=(col(&spec,"combat_cards","effective_cost"),col(&spec,"combat_cards","combat_cost"),col(&spec,"combat_cards","turn_cost"),col(&spec,"combat_cards","retain"));
 for (row,card) in c.hand.iter().enumerate(){assert_eq!(cell(cc,row,eff),card.effective_cost.map_or((0,0),|v|(i64::from(v),1)));let w=card.watcher.as_ref().unwrap();assert_eq!(cell(cc,row,cost),w.combat_cost.map_or((0,0),|v|(i64::from(v),1)));assert_eq!(cell(cc,row,turn),w.turn_cost.map_or((0,0),|v|(i64::from(v),1)));assert_eq!(cell(cc,row,ret).0,i64::from(w.retain));}
 // Scry (actual M23 diagnostic stage, M23 identity, no relabel): reveal position and selected ordinal B,A after deselect/reselect; Confirm0.
 let mut r=m23_diagnostic(M23Diagnostic::ScryChoice).unwrap();assert_eq!(r.decision().unwrap().content_profile,Some(RoomScenario::M23WatcherAct2));let sel=r.decision().unwrap().combat.unwrap().selection.unwrap();let (a,b)=(sel.candidates[0].instance.0,sel.candidates[1].instance.0);
 let zero=r.clone();for x in [CombatAction::SelectCard{instance:a},CombatAction::SelectCard{instance:b},CombatAction::DeselectCard{instance:a},CombatAction::SelectCard{instance:a}]{r.step(&GameAction::Combat(x)).unwrap();}
 let d=r.decision().unwrap();let o=encode_run_m23(&d,r.public_map().as_ref(),&caps()).unwrap();let g=group(&o,"selection");let (rp,so)=(col(&spec,"selection","reveal_position"),col(&spec,"selection","selected_ordinal"));
 assert_eq!((cell(g,0,rp),cell(g,1,rp)),((0,1),(1,1)));assert_eq!((cell(g,0,so).0,cell(g,1,so).0),(2,1));
 for mut run in [r,zero]{run.step(&GameAction::Combat(CombatAction::ConfirmSelection)).unwrap();encode_run_m23(&run.decision().unwrap(),run.public_map().as_ref(),&caps()).unwrap();}
 // Meditate return (actual M23 stage) uses the Watcher operation row.
 let r=m23_diagnostic(M23Diagnostic::MeditateChoice).unwrap();let d=r.decision().unwrap();assert!(d.combat.as_ref().unwrap().selection.is_some());encode_run_m23(&d,r.public_map().as_ref(),&caps()).unwrap();
 // StancePotion (actual M23 stage): distinct nonphysical Wrath/Calm rows referenced by actions.
 let r=m23_diagnostic(M23Diagnostic::StanceChoice).unwrap();let d=r.decision().unwrap();let o=encode_run_m23(&d,r.public_map().as_ref(),&caps()).unwrap();
 assert_eq!(group(&o,"stance_choices").values,vec![1,2]);let refc=col(&spec,"actions","stance_choice_ref");let refs=(0..d.candidates.len()).map(|i|cell(group(&o,"actions"),i,refc).0).filter(|v|*v>=0).collect::<Vec<_>>();assert_eq!(refs,vec![0,1]);
 // Bronze Orb Stasis in the actual M23 Act2 boss room (supplied M23 entry, labelled): one stasis-zone row pointing at its orb; no extra identities.
 let mut r=m23_played_fixture(M23PlayedFixture::StasisBronzeAutomaton).unwrap();let mv=first_move(&r.decision().unwrap());r.step(&mv).unwrap();let mut held=false;
 for _ in 0..8{let d=r.decision().unwrap();let Some(c)=d.combat.as_ref() else{break};if !c.stasis.is_empty(){let o=encode_run_m23(&d,r.public_map().as_ref(),&caps()).unwrap();let cc=group(&o,"combat_cards");let zone=vocab_id(&spec,"zone","stasis");let sz=col(&spec,"combat_cards","stasis_enemy_row");
  let rows=(0..cc.shape[0]).filter(|i|cc.occupied.as_ref().unwrap()[*i]==1).collect::<Vec<_>>();let stasis_rows=rows.iter().filter(|i|cell(cc,**i,0).0==zone).collect::<Vec<_>>();assert_eq!(stasis_rows.len(),c.stasis.len());
  for (row,h) in stasis_rows.iter().zip(&c.stasis){let orb=c.enemies.iter().position(|e|e.instance_id==h.orb).unwrap();assert_eq!(cell(cc,**row,sz),(orb as i64,1));assert_eq!(cell(cc,**row,2).0,vocab_id(&spec,"card",h.card.id.as_str()));}
  let mut ids=c.hand.iter().chain(&c.discard).chain(&c.exhaust).chain(&c.played_powers).chain(&c.playing_cards).chain(c.stasis.iter().map(|x|&x.card)).chain(c.known_draw.iter().map(|x|&x.card)).map(|x|x.instance).collect::<Vec<_>>();ids.sort();ids.dedup();assert_eq!(rows.len(),ids.len());held=true;break}
  if !end_turn(&mut r){break}}
 assert!(held,"actual Stasis reached in the M23 BronzeAutomaton room");
}

#[test]
fn m23_numeric_privacy_and_capacity_survive_continuation(){
 // Same public start under different seeds/RNG: identical numeric result; physical handles never become features.
 let (mut a,mut b)=(RunSession::new_m23(config()).unwrap(),RunSession::new_m23(config()).unwrap());assert_eq!(a.reset(Some(7)).unwrap().observation,b.reset(Some(11)).unwrap().observation);
 let mut d=latest(&a);let original=encode_run_m23(&d,None,&caps()).unwrap();for c in &mut d.objects{c.instance+=1000;}assert_eq!(encode_run_m23(&d,None,&caps()).unwrap(),original);d.objects[0].id=CardId::Strike;assert!(encode_run_m23(&d,None,&caps()).is_err(),"foreign permanent fails closed");
 let neow=latest(&a);act(&mut a,neow.candidates[0].action.clone());let map_view=latest(&a);let map=a.reproduction().unwrap().latest_map;
 // Constructed public boundaries (labelled): healthy Act3Ready is zero-action; false or downstream boundaries reject.
 let ready=|act:u8,floor:u32,kind:RoomViewKind,keep:bool|{let mut d=map_view.clone();d.act=act;d.floor=floor;d.room.as_mut().unwrap().kind=kind;d.room.as_mut().unwrap().choices.clear();if !keep{d.candidates.clear();}d};
 let o=encode_run_m23(&ready(3,34,RoomViewKind::Act3Ready,false),None,&caps()).unwrap();assert_eq!(o.action_mask,vec![0;caps().actions]);assert!(group(&o,"actions").occupied.as_ref().unwrap().iter().all(|x|*x==0));
 let spec=m23_run_observation_spec(&caps()).unwrap().descriptor();assert_eq!(cell(group(&o,"context"),0,col(&spec,"context","room_kind")).0,vocab_id(&spec,"room","act3_ready"));assert_eq!(cell(group(&o,"context"),0,col(&spec,"context","ascension")),(0,1));
 for bad in [ready(3,34,RoomViewKind::Act3Ready,true),ready(3,33,RoomViewKind::Act3Ready,false),ready(2,34,RoomViewKind::Act3Ready,false),ready(2,17,RoomViewKind::Act2Ready,false),ready(3,34,RoomViewKind::Map,true),ready(2,35,RoomViewKind::Map,true)]{assert!(matches!(encode_run_m23(&bad,None,&caps()),Err(RunSessionError::UnsupportedBoundary(_))),"{:?}/{}/{:?}",bad.act,bad.floor,bad.room.as_ref().unwrap().kind);}
 let mut over=map_view.clone();over.terminal=Some(RunTerminal::Act3NoKeys);over.candidates.clear();assert!(encode_run_m23(&over,None,&caps()).is_err());
 let mut cont=map_view.clone();cont.act=2;cont.floor=17;encode_run_m23(&cont,map.as_ref(),&caps()).expect("committed (1,2,17) continues to the Act2 Map");
 // Capacity faults name group/required/capacity; never truncate, filter or page legal content.
 let s=m23_combat(7);let d=latest(&s);let map=s.reproduction().unwrap().latest_map.unwrap();let c=d.combat.as_ref().unwrap();
 let cards=c.hand.len()+c.discard.len()+c.exhaust.len();let potions=c.inventory.as_ref().unwrap().potions.len();
 let mut checks:Vec<(&str,usize,Box<dyn Fn(&mut RunCapacities,usize)>)>=vec![("map_nodes",map.nodes.len(),Box::new(|c,n|c.map_nodes=n)),("permanent_cards",d.objects.len(),Box::new(|c,n|c.permanent_cards=n)),("combat_cards",cards,Box::new(|c,n|c.combat_cards=n)),("potions",potions,Box::new(|c,n|c.potions=n)),("actions",d.candidates.len(),Box::new(|c,n|c.actions=n))];
 if c.enemies.len()>1{checks.push(("enemies",c.enemies.len(),Box::new(|c,n|c.enemies=n)));}
 for (g,required,set) in &checks{let mut small=caps();set(&mut small,required-1);assert_eq!(encode_run_m23(&d,Some(&map),&small),Err(RunSessionError::Capacity{group:g,required:*required,capacity:required-1}),"{g}");}
 let mut r=fixture(EncounterId::JawWorm,supplied(&[CardId::ThirdEye;10],vec![None;3]));assert!(play(&mut r,CardId::ThirdEye));let sd=relabel(&r);let n=sd.combat.as_ref().unwrap().selection.as_ref().unwrap().candidates.len();let mut small=caps();small.selection=n-1;assert_eq!(encode_run_m23(&sd,None,&small),Err(RunSessionError::Capacity{group:"selection",required:n,capacity:n-1}));
 // Owned observations remain stable after later core mutations.
 let mut s=m23_combat(7);let before=s.decision().unwrap();let snapshot=before.clone();act(&mut s,GameAction::Combat(CombatAction::EndTurn));assert_eq!(before,snapshot);assert_ne!(s.decision().unwrap().token,before.token);
}

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

fn decode(t:&str)->GameAction{
 let (head,rest)=t.split_at(1);let mut it=rest.split('@');let n=it.next().filter(|s|!s.is_empty()).map(|s|s.parse::<u64>().unwrap());let target=it.next().map(|s|TargetId(s.parse().unwrap()));
 match head{"C"=>GameAction::Choose(ChoiceId(n.unwrap())),"M"=>GameAction::Move(NodeId(n.unwrap())),"S"=>GameAction::Select(SelectionId(n.unwrap())),"K"=>GameAction::Confirm,"X"=>GameAction::Cancel,
  "u"=>GameAction::UsePotion{slot:n.unwrap() as u32,target},"d"=>GameAction::DiscardPotion{slot:n.unwrap() as u32},
  "P"=>GameAction::Combat(CombatAction::PlayCard{instance:n.unwrap(),target}),"L"=>GameAction::Combat(CombatAction::SelectCard{instance:n.unwrap()}),"D"=>GameAction::Combat(CombatAction::DeselectCard{instance:n.unwrap()}),
  "F"=>GameAction::Combat(CombatAction::ConfirmSelection),"E"=>GameAction::Combat(CombatAction::EndTurn),"U"=>GameAction::Combat(CombatAction::UsePotion{slot:n.unwrap() as usize,target}),
  "W"=>GameAction::Combat(CombatAction::ChooseStance{stance:Stance::Wrath}),"A"=>GameAction::Combat(CombatAction::ChooseStance{stance:Stance::Calm}),_=>panic!("token {t}")}
}
fn transcript(s:&str)->Vec<GameAction>{s.split(' ').map(decode).collect()}
fn drive(s:&mut RunSession,actions:&[GameAction])->Vec<RunStepResult>{actions.iter().enumerate().map(|(i,a)|{let d=latest(s);assert!(d.candidates.iter().any(|c|&c.action==a),"retained action {i} {a:?} not offered");act(s,a.clone())}).collect()}
fn boss(a1:RunAct1BossTarget)->EncounterId{match a1{RunAct1BossTarget::Guardian=>EncounterId::Guardian,RunAct1BossTarget::Hexaghost=>EncounterId::Hexaghost,RunAct1BossTarget::SlimeBoss=>EncounterId::SlimeBoss}}
fn boss2(a2:RunAct2BossTarget)->EncounterId{match a2{RunAct2BossTarget::Champ=>EncounterId::Champ,RunAct2BossTarget::BronzeAutomaton=>EncounterId::BronzeAutomaton,RunAct2BossTarget::Collector=>EncounterId::Collector}}
const A1:[RunAct1BossTarget;3]=[RunAct1BossTarget::Guardian,RunAct1BossTarget::Hexaghost,RunAct1BossTarget::SlimeBoss];
const A2:[RunAct2BossTarget;3]=[RunAct2BossTarget::Champ,RunAct2BossTarget::BronzeAutomaton,RunAct2BossTarget::Collector];
fn wire(r:&RunEpisodeRecord)->Value{serde_json::from_slice(&r.to_json().unwrap()).unwrap()}
fn import(v:&Value)->Result<RunEpisodeRecord,RunSessionError>{RunEpisodeRecord::from_json(&serde_json::to_vec(v).unwrap())}

#[test]
fn m23_config_history_validation_precedes_initialization(){
 for a1 in A1{for a2 in A2{let mut s=RunSession::new_m23(cfg(a1,a2)).unwrap();s.reset(Some(7)).unwrap();let neow=latest(&s);act(&mut s,neow.candidates[0].action.clone());assert_eq!(s.reproduction().unwrap().latest_map.unwrap().boss,Some(boss(a1)),"{a1:?}/{a2:?} Act1 exclusion");}}
 // Exact M22 Act1 prefix (retained normal transcript), then the M23 Act2 Map whose boss is the configured unseen target.
 let act1=transcript(M22_SEED7_SLIME_BOSS);let mut m22=RunSession::new_m22(RunSessionConfig{boss_history:RunBossHistory::Act1SlimeBossUnseen,..config()}).unwrap();m22.reset(Some(7)).unwrap();let m22_steps=drive(&mut m22,&act1);
 let ready=latest(&m22);assert_eq!(ready.room.as_ref().unwrap().kind,RoomViewKind::Act2Ready);assert!(ready.candidates.is_empty());
 for a2 in A2{let mut s=RunSession::new_m23(cfg(RunAct1BossTarget::SlimeBoss,a2)).unwrap();s.reset(Some(7)).unwrap();let mut m=RunSession::new_m22(RunSessionConfig{boss_history:RunBossHistory::Act1SlimeBossUnseen,..config()}).unwrap();m.reset(Some(7)).unwrap();
  for (i,a) in act1.iter().enumerate(){let (mut x,mut y)=(latest(&s),latest(&m));x.content_profile=None;y.content_profile=None;assert_eq!(x,y,"Act1 prefix diverged at {i}");let (p,q)=(act(&mut s,a.clone()),act(&mut m,a.clone()));assert_eq!((p.reward,p.terminated),(q.reward,q.terminated));assert_eq!(p.reward,m22_steps[i].reward);}
  let d=latest(&s);assert_eq!((d.act,d.floor,d.room.as_ref().unwrap().kind),(2,17,RoomViewKind::Map));assert!(d.terminal.is_none()&&d.candidates.iter().any(|c|matches!(c.action,GameAction::Move(_))));
  let map=s.reproduction().unwrap().latest_map.unwrap();assert_eq!((map.act,map.boss),(2,Some(boss2(a2))));assert!(s.decision().unwrap().observation.action_mask.iter().any(|m|*m==1));}
 // Rejected before any Run/resource/RNG startup (initializer count is asserted in the owning module).
 let other=[RunBossHistory::Act1GuardianUnseen,RunBossHistory::AwakenedUnseen,RunBossHistory::M17DefectAct1{act1:RunAct1BossTarget::Guardian},RunBossHistory::M13SilentAct2{act1:RunAct1BossTarget::Guardian,act2:RunAct2BossTarget::Champ},RunBossHistory::M18DefectAct2{act1:RunAct1BossTarget::Guardian,act2:RunAct2BossTarget::Champ},RunBossHistory::M14SilentAct3{act1:RunAct1BossTarget::Guardian,act2:RunAct2BossTarget::Champ,act3:RunAct3BossTarget::AwakenedOne},RunBossHistory::M15SilentHeart{act1:RunAct1BossTarget::Guardian,act2:RunAct2BossTarget::Champ,act3:RunAct3BossTarget::AwakenedOne},RunBossHistory::M19DefectAct3{act1:RunAct1BossTarget::Guardian,act2:RunAct2BossTarget::Champ,act3:RunAct3BossTarget::AwakenedOne},RunBossHistory::M20DefectHeart{act1:RunAct1BossTarget::Guardian,act2:RunAct2BossTarget::Champ,act3:RunAct3BossTarget::AwakenedOne}];
 for h in other{assert!(matches!(RunSession::new_m23(RunSessionConfig{boss_history:h,..config()}),Err(RunSessionError::Input(_))),"{h:?}");}
 for ch in [Character::Ironclad,Character::Silent,Character::Defect]{assert_eq!(RunSession::new_m23(RunSessionConfig{character:ch,..config()}).err(),Some(RunSessionError::Input("M23 requires Watcher A0")));}
 assert_eq!(RunSession::new_m23(RunSessionConfig{ascension:1,..config()}).err(),Some(RunSessionError::Input("M23 requires Watcher A0")));
 let mut bad=config();bad.capacities.actions=0;assert!(RunSession::new_m23(bad).is_err());
 assert!(RunSession::new_m22(config()).is_err(),"M22 rejects the composite M23 history");assert!(RunSession::new_m18(RunSessionConfig{character:Character::Defect,..config()}).is_err(),"no normalization into an old profile");
 // Old alias, strings and default constructor stay valid.
 let old:M9BossHistory=M9BossHistory::AwakenedUnseen;RunSession::new(RunSessionConfig{character:Character::Ironclad,ascension:0,partition:Partition::Test,boss_history:old,capacities:RunCapacities::m9_default(),record_replay:true}).unwrap();
}

#[test]
fn m23_raw_json_history_and_action_shapes_are_strict(){
 let mut s=RunSession::new_m23(cfg(RunAct1BossTarget::SlimeBoss,RunAct2BossTarget::BronzeAutomaton)).unwrap();s.reset(Some(7)).unwrap();drive(&mut s,&transcript(M22_SEED7_SLIME_BOSS)[..3]);s.fail_delivery("transport acknowledgement lost").unwrap();
 let r=s.record().unwrap().unwrap();let bytes=r.to_json().unwrap();let text=String::from_utf8(bytes.clone()).unwrap();
 assert!(text.contains(r#""boss_history":{"m23-watcher-act2":{"act1":"slime_boss","act2":"bronze_automaton"}}"#),"{text}");assert_eq!(RunEpisodeRecord::from_json(&bytes).unwrap(),r);
 assert_eq!((r.schema_id.as_str(),r.profile_id.as_str(),r.rules_id.as_str(),r.content_id.as_str()),("spire-gym-m23-watcher-act2-replay-v1","M23WatcherAct2","spire-gym-native-m23-rules-v1","spire-gym-m23-watcher-a0-acts1-2-captured-v1"));
 assert_eq!((RUN_RECORD_SCHEMA_ID_M23,RUN_RULES_ID_M23,RUN_CONTENT_ID_M23),(r.schema_id.as_str(),r.rules_id.as_str(),r.content_id.as_str()));
 // Byte-level duplicates at every nesting reject before Value conversion.
 for (from,to) in [(r#""closed":false"#,r#""closed":false,"closed":false"#),(r#""character":"watcher""#,r#""character":"watcher","character":"watcher""#),(r#"{"act1":"slime_boss""#,r#"{"act1":"slime_boss","act1":"slime_boss""#),(r#""act2":"bronze_automaton""#,r#""act2":"bronze_automaton","act2":"collector""#),
  (r#"{"m23-watcher-act2":{"act1":"slime_boss","act2":"bronze_automaton"}}"#,r#"{"m23-watcher-act2":{"act1":"slime_boss","act2":"bronze_automaton"},"m23-watcher-act2":{"act1":"slime_boss","act2":"bronze_automaton"}}"#),(r#""choice":1"#,r#""choice":1,"choice":1"#),(r#""kind":"delivery_fault""#,r#""kind":"delivery_fault","kind":"delivery_fault""#),(r#""actions":2048"#,r#""actions":2048,"actions":2048"#)]{
  let dup=text.replacen(from,to,1);assert_ne!(dup,text,"{from}");assert!(RunEpisodeRecord::from_json(dup.as_bytes()).is_err(),"duplicate {from}");}
 // Unknown/extra keys and mixed/partial/wrong-type histories reject.
 let v=wire(&r);for pointer in ["","/config","/config/capacities","/config/boss_history","/config/boss_history/m23-watcher-act2","/steps/0","/steps/0/action","/fault","/fault/error"]{let mut w=v.clone();w.pointer_mut(pointer).unwrap().as_object_mut().unwrap().insert("extra".into(),json!(1));assert!(import(&w).is_err(),"extra at {pointer}");}
 for h in [json!("act1-slime-boss-unseen"),json!({"m18-defect-act2":{"act1":"slime_boss","act2":"bronze_automaton"}}),json!({"m13-silent-act2":{"act1":"slime_boss","act2":"bronze_automaton"}}),json!({"m23-watcher-act2":{"act1":"slime_boss"}}),json!({"m23-watcher-act2":{"act2":"bronze_automaton"}}),json!({"m23-watcher-act2":{"act1":"slime_boss","act2":"automaton"}}),json!({"m23-watcher-act2":{"act1":"slime_boss","act2":"Automaton"}}),json!({"m23-watcher-act2":{"act1":1,"act2":"champ"}}),json!({"m23-watcher-act2":{"act1":"slime_boss","act2":"bronze_automaton","act3":"awakened-one"}}),json!({"m23-watcher-act2":"slime_boss"}),json!({"m23-watcher-act2":{"act1":"slime_boss","act2":"bronze_automaton"},"m22":1})]{let mut w=v.clone();w["config"]["boss_history"]=h.clone();assert!(import(&w).is_err(),"{h}");}
 // New-profile data is never normalized into an old profile, and old profiles keep their spellings.
 let mut w=v.clone();w["schema_id"]=json!(RUN_RECORD_SCHEMA_ID_M22);w["profile_id"]=json!("M22WatcherAct1");w["rules_id"]=json!(RUN_RULES_ID_M22);w["content_id"]=json!(RUN_CONTENT_ID_M22);assert!(import(&w).is_err());
 let mut w=v.clone();w["schema_id"]=json!(RUN_RECORD_SCHEMA_ID_M18);w["profile_id"]=json!("M18DefectAct2");w["rules_id"]=json!(RUN_RULES_ID_M18);w["content_id"]=json!(RUN_CONTENT_ID_M18);w["config"]["boss_history"]=json!({"m18-defect-act2":{"act1":"slime_boss","act2":"bronze_automaton"}});assert!(import(&w).is_err(),"Watcher config is not an M18 record");
 let mut m22=RunSession::new_m22(RunSessionConfig{boss_history:RunBossHistory::Act1SlimeBossUnseen,..config()}).unwrap();m22.reset(Some(7)).unwrap();let mv=wire(&m22.record().unwrap().unwrap());assert_eq!(mv["config"]["boss_history"],json!("act1-slime-boss-unseen"));let mut w=mv.clone();w["config"]["boss_history"]=v["config"]["boss_history"].clone();assert!(import(&w).is_err());
 // Watcher semantic verbs are admitted by the M23 codec; old schemas reject them.
 let base={let mut t=RunSession::new_m23(config()).unwrap();t.reset(Some(7)).unwrap();t.record().unwrap().unwrap()};
 for a in [CombatAction::DeselectCard{instance:4},CombatAction::ChooseStance{stance:Stance::Wrath},CombatAction::ChooseStance{stance:Stance::Calm},CombatAction::ConfirmSelection,CombatAction::SelectCard{instance:4}]{let mut x=base.clone();x.steps.push(RunRecordedStep{action:GameAction::Combat(a),reward:0,terminal:None});let wv=x.to_json().unwrap();assert_eq!(RunEpisodeRecord::from_json(&wv).unwrap(),x);let mut j:Value=serde_json::from_slice(&wv).unwrap();j["steps"][0]["action"]["hidden"]=json!(1);assert!(import(&j).is_err());}
 let mut x=base.clone();x.steps.push(RunRecordedStep{action:GameAction::Combat(CombatAction::ChooseStance{stance:Stance::Divinity}),reward:0,terminal:None});assert!(x.to_json().is_err(),"Divinity is never a choice verb");
 let mut c=config();c.character=Character::Defect;c.boss_history=RunBossHistory::M18DefectAct2{act1:RunAct1BossTarget::Guardian,act2:RunAct2BossTarget::Champ};let mut old=RunSession::new_m18(c).unwrap();old.reset(Some(7)).unwrap();let mut y=old.record().unwrap().unwrap();y.steps.push(RunRecordedStep{action:GameAction::Combat(CombatAction::ChooseStance{stance:Stance::Calm}),reward:0,terminal:None});assert!(y.to_json().is_err());
 // Typed export and raw import share capacity/structured-error validation.
 let mut t=base.clone();t.fault=Some(RunRecordedFault{action:None,committed:false,gameplay_recorded:false,error:RunRecordError{kind:RunRecordErrorKind::Capacity,message:"Run capacity exceeded".into(),capacity:None}});assert!(t.to_json().is_err());
 let mut ok=base.clone();ok.fault=Some(RunRecordedFault{action:None,committed:false,gameplay_recorded:false,error:RunRecordError{kind:RunRecordErrorKind::Capacity,message:"Run capacity exceeded".into(),capacity:Some(RunCapacityFailure{group:"map_nodes".into(),required:3,capacity:2})}});let okv=wire(&ok);assert_eq!(import(&okv).unwrap(),ok);
 for (field,value) in [("group",json!("hidden_rng")),("required",json!(2)),("capacity",json!(-1))]{let mut w=okv.clone();w["fault"]["error"]["capacity"][field]=value;assert!(import(&w).is_err(),"{field}");let mut typed=ok.clone();if field=="group"{typed.fault.as_mut().unwrap().error.capacity.as_mut().unwrap().group="hidden_rng".into();assert!(typed.to_json().is_err());}}
}

#[test]
fn m23_lifecycle_has_direct_implicit_ordinal_and_healthy_cutoff(){
 let mut s=RunSession::new_m23(config()).unwrap();assert_eq!(s.decision(),Err(RunSessionError::NotReset));assert_eq!(s.step(RunDecisionToken::from([0,0]),RunActionIndex(0)),Err(RunSessionError::NotReset));assert!(s.record().unwrap().is_none());
 let first=s.reset(Some(7)).unwrap();let r0=s.record().unwrap().unwrap();assert_eq!((r0.seed,r0.ordinal),(7,0));
 let mut again=RunSession::new_m23(config()).unwrap();assert_eq!(again.reset(Some(7)).unwrap().observation,first.observation,"ordinal0 reproducible");
 let second=s.reset(None).unwrap();let r1=s.record().unwrap().unwrap();assert_eq!((r1.seed,r1.ordinal),(7,1));assert_ne!(first.token,second.token);assert_eq!(s.effective_seed(),Some(7));
 let mut peer=RunSession::new_m23(config()).unwrap();let pb=peer.reset(Some(7)).unwrap();let n=latest(&s).candidates.len() as i64;let before=s.reproduction().unwrap();
 for (token,i,want) in [(first.token,0,Some(RunSessionError::StaleToken)),(pb.token,0,Some(RunSessionError::StaleToken)),(second.token,-1,None),(second.token,n,None),(second.token,i64::MAX,None)]{let e=s.step(token,RunActionIndex(i)).unwrap_err();if let Some(want)=want{assert_eq!(e,want);}else{assert!(matches!(e,RunSessionError::Input(_)));}assert_eq!(s.reproduction().unwrap(),before);}
 // Actual Death from the retained M22-normal Act1 transcript replayed in M23 at ordinal0.
 let mut d=RunSession::new_m23(config()).unwrap();d.reset(Some(7)).unwrap();let steps=drive(&mut d,&transcript(M22_SEED7_GUARDIAN));let last=steps.last().unwrap();
 assert!(steps[..steps.len()-1].iter().all(|x|!x.terminated));assert!(last.terminated&&!last.truncated);assert_eq!(last.boundary.terminal,Some(RunTerminal::Death));assert!(last.boundary.observation.action_mask.iter().all(|m|*m==0));assert_eq!(d.decision().unwrap(),last.boundary);
 assert_eq!(d.step(last.boundary.token,RunActionIndex(0)),Err(RunSessionError::Terminal));let rec=d.record().unwrap().unwrap();assert_eq!((rec.terminal,rec.steps.len()),(Some(RunTerminal::Death),146));
 let rep=d.reproduction().unwrap();assert_eq!(rep.latest_decision.unwrap().unwrap().terminal,Some(RunTerminal::Death));d.reset(None).unwrap();assert_eq!(d.record().unwrap().unwrap().ordinal,1);d.close().unwrap();assert_eq!(d.decision(),Err(RunSessionError::Closed));assert!(d.record().unwrap().unwrap().closed);assert!(d.reproduction().unwrap().closed);assert_eq!(d.reset(None),Err(RunSessionError::Closed));
 // UNMET in normal play: no retained transcript reaches the Act2 boss, so healthy Act3Ready is covered only by the labelled owning-module forced cutoff.
}

#[test]
fn m23_semantic_replay_uses_recorded_actions_at_recorded_ordinal(){
 // Retained real Act1 transcript (Neow, map, rewards, shop/event, Scry Confirm0 and StancePotion Calm) continuing into the M23 Act2 Map.
 let mut s=RunSession::new_m23(cfg(RunAct1BossTarget::SlimeBoss,RunAct2BossTarget::Collector)).unwrap();s.reset(Some(7)).unwrap();let steps=drive(&mut s,&transcript(M22_SEED7_SLIME_BOSS));
 let r=s.record().unwrap().unwrap();assert!(r.steps.iter().any(|x|matches!(x.action,GameAction::Combat(CombatAction::ChooseStance{stance:Stance::Calm}))));assert!(r.steps.iter().any(|x|x.action==GameAction::Combat(CombatAction::ConfirmSelection)));
 let back=RunEpisodeRecord::from_json(&r.to_json().unwrap()).unwrap();assert_eq!(back,r);let report=replay_run(&back,caps()).unwrap();
 assert_eq!(report.final_decision,latest(&s));assert_eq!(report.final_map,s.reproduction().unwrap().latest_map);assert_eq!(report.accepted_count,273);assert_eq!(report.rewards,steps.iter().map(|x|x.reward).collect::<Vec<_>>());assert!(report.terminals.iter().all(|t|t.is_none()));assert!(report.final_decision_error.is_none());
 assert_eq!((report.final_decision.act,report.final_decision.floor),(2,17));
 // Implicit reset ordinal1 record reconstructs directly at ordinal1 (fixed passive public choices, not a policy).
 let mut t=RunSession::new_m23(config()).unwrap();t.reset(Some(11)).unwrap();t.reset(None).unwrap();for _ in 0..40{let d=latest(&t);if d.terminal.is_some(){break}act(&mut t,passive(&d));}
 let r1=t.record().unwrap().unwrap();assert_eq!((r1.seed,r1.ordinal),(11,1));let rep=replay_run(&RunEpisodeRecord::from_json(&r1.to_json().unwrap()).unwrap(),caps()).unwrap();assert_eq!(rep.final_decision,latest(&t));assert_eq!(rep.accepted_count,r1.steps.len());
 // The numeric index/token is never reused: a semantic action that is not currently offered fails at the right boundary.
 let mut wrong=r.clone();wrong.steps[1].action=GameAction::Move(NodeId(999_999));assert_eq!(replay_run(&wrong,caps()).err(),Some(RunSessionError::IncompatibleReplay("Run action not currently offered")));
}

#[test]
fn m23_replay_preflights_identity_before_startup_and_checks_results(){
 let mut s=RunSession::new_m23(config()).unwrap();s.reset(Some(7)).unwrap();drive(&mut s,&transcript(M22_SEED7_GUARDIAN)[..20]);let r=s.record().unwrap().unwrap();replay_run(&r,caps()).unwrap();
 let mutate:Vec<Box<dyn Fn(&mut RunEpisodeRecord)>>=vec![Box::new(|x|x.schema_id=RUN_RECORD_SCHEMA_ID_M22.into()),Box::new(|x|x.profile_id="M22WatcherAct1".into()),Box::new(|x|x.rules_id=RUN_RULES_ID_M22.into()),Box::new(|x|x.content_id=RUN_CONTENT_ID_M22.into()),Box::new(|x|x.rng_id="other-rng".into()),Box::new(|x|x.reward_id="other-reward".into()),Box::new(|x|x.seed_adapter_id="other-seed".into()),
  Box::new(|x|x.observation_fingerprint[0]^=1),Box::new(|x|x.observation_fingerprint=m22_run_observation_spec(&caps()).unwrap().fingerprint()),Box::new(|x|x.config.boss_history=RunBossHistory::Act1GuardianUnseen),Box::new(|x|x.config.boss_history=RunBossHistory::M18DefectAct2{act1:RunAct1BossTarget::Guardian,act2:RunAct2BossTarget::Champ}),Box::new(|x|x.config.character=Character::Silent),Box::new(|x|x.config.ascension=1),Box::new(|x|x.config.capacities.actions=1)];
 for (i,m) in mutate.iter().enumerate(){let mut x=r.clone();m(&mut x);assert!(matches!(replay_run(&x,caps()),Err(RunSessionError::IncompatibleReplay(_))),"identity mutation {i}");assert!(x.to_json().is_err(),"export mutation {i}");}
 // Altered producer/capture metadata changes the recomputed descriptor fingerprint.
 let spec=m23_run_observation_spec(&caps()).unwrap();for (path,value) in [("/semantics/finite_content/producers/shop_relic_price",json!(1)),("/semantics/finite_content/capture_law/scope/act2",json!("run.m22.relic.capture")),("/semantics/finite_content/status_cards",json!(["dazed","burn","slimed"]))]{let mut d=spec.descriptor();*d.pointer_mut(path).unwrap()=value;let mut h=Sha256::new();h.update(b"spire-gym/run-schema/m23-watcher-act2/v1\0");h.update(canonical_json(&d).unwrap());assert_ne!(<[u8;32]>::from(h.finalize()),spec.fingerprint(),"{path}");}
 // Well-formed but altered action/reward/outcome fails against actual execution.
 let mut x=r.clone();x.steps[5].reward+=1;assert_eq!(replay_run(&x,caps()).err(),Some(RunSessionError::IncompatibleReplay("Run reward or terminal mismatch")));
 let mut x=r.clone();x.steps[19].terminal=Some(RunTerminal::Death);x.terminal=Some(RunTerminal::Death);assert!(replay_run(&x,caps()).is_err());
 let mut x=r.clone();x.steps[2].action=GameAction::Combat(CombatAction::PlayCard{instance:999,target:None});assert_eq!(replay_run(&x,caps()).err(),Some(RunSessionError::IncompatibleReplay("Run action not currently offered")));
 // A valid alternate seed/ordinal is another valid record, not detectable authentication tampering.
 let mut alt=r.clone();alt.steps.clear();alt.seed=8;alt.ordinal=3;replay_run(&alt,caps()).unwrap();assert_eq!(RunEpisodeRecord::from_json(&alt.to_json().unwrap()).unwrap(),alt);
}

#[test]
fn m23_delivery_fault_record_preserves_committed_semantics(){
 // Postaccept publication capacity failure: committed prefix, serial and first structured error retained; no empty success.
 let mut c=config();c.capacities.map_nodes=1;let mut s=RunSession::new_m23(c.clone()).unwrap();let b=s.reset(Some(7)).unwrap();let error=s.step(b.token,RunActionIndex(0)).unwrap_err();assert_eq!(error,RunSessionError::DeliveryFault("post-step Run encoding failed"));
 let r=s.record().unwrap().unwrap();assert_eq!(r.steps.len(),1);let f=r.fault.as_ref().unwrap();assert!(f.committed&&f.gameplay_recorded);assert_eq!(f.error.kind,RunRecordErrorKind::Capacity);assert_eq!(f.error.capacity.as_ref().unwrap().group,"map_nodes");assert_eq!(f.action.as_ref(),Some(&r.steps[0].action));
 let decoded=RunEpisodeRecord::from_json(&r.to_json().unwrap()).unwrap();assert_eq!(decoded,r);let report=replay_run(&decoded,c.capacities.clone()).unwrap();assert_eq!(report.accepted_count,1);assert_eq!(report.reported_fault,r.fault);assert!(report.final_decision_error.is_none());assert_eq!(report.final_decision,latest(&s));
 let before=s.reproduction().unwrap();assert!(matches!(before.fault,Some(RunSessionError::DeliveryFault(_))));assert!(s.reset(None).is_err());assert!(s.step(b.token,RunActionIndex(0)).is_err());assert!(s.decision().is_err());s.fail_delivery("later").unwrap();assert_eq!(s.reproduction().unwrap(),before);s.close().unwrap();assert!(s.record().unwrap().unwrap().closed);assert!(RunEpisodeRecord::from_json(&s.record().unwrap().unwrap().to_json().unwrap()).is_ok());
 // Startup publication failure on reset: no accepted gameplay, first fault retained.
 let mut c2=config();c2.capacities.actions=1;let mut z=RunSession::new_m23(c2.clone()).unwrap();let e=z.reset(Some(7)).unwrap_err();assert_eq!(e,RunSessionError::Capacity{group:"actions",required:2,capacity:1});let zr=z.record().unwrap().unwrap();assert!(zr.steps.is_empty());let zf=zr.fault.as_ref().unwrap();assert!(!zf.committed&&!zf.gameplay_recorded);
 let zb=RunEpisodeRecord::from_json(&zr.to_json().unwrap()).unwrap();assert_eq!(zb,zr);let zrep=replay_run(&zb,c2.capacities.clone()).unwrap();assert_eq!(zrep.accepted_count,0);assert_eq!(zrep.reported_fault,zr.fault);
 // Transport-only loss after an accepted combat action: reported diagnostic, no fabricated core action.
 let mut t=RunSession::new_m23(config()).unwrap();t.reset(Some(7)).unwrap();drive(&mut t,&transcript(M22_SEED7_SLIME_BOSS)[..4]);t.fail_delivery("python acknowledgement lost").unwrap();let tr=t.record().unwrap().unwrap();let tf=tr.fault.as_ref().unwrap();assert!(tf.committed&&tf.gameplay_recorded);assert_eq!(tf.error.kind,RunRecordErrorKind::DeliveryFault);assert_eq!(tr.steps.len(),4);
 let rep=replay_run(&RunEpisodeRecord::from_json(&tr.to_json().unwrap()).unwrap(),caps()).unwrap();assert_eq!(rep.accepted_count,4);assert_eq!(rep.final_decision,latest(&t));assert_eq!(rep.reported_fault,tr.fault);assert!(t.reset(Some(7)).is_err(),"faulted session cannot reset into a new loss");
}
// seed-11-guardian-normal-budget-32768.attempt.json (M22 original normal, Death after 114) sha256 9549c61f0bacaa8bf3d5c5972545bdfcdf8f162f7191a1b45e54b950dd8f43bd, 114 actions.
const M22_SEED11_GUARDIAN:&str="C1 M3 P11 P5 P2@1 P10 E P4@1 P8 P1@1 E P4@1 P5 P2@1 E P7 P8 P6 E P8 P7 P4@1 E P10 P5 E P5 P3@1 P1@1 E P7 P4@1 C2 C3 C6 C4 C5 M11 C10 S12 M19 C34 S36 M27 U0 P13 P4@1 P12@1 P2@1 P7 E P3@1 P6 P5 E P8 P4@1 P2@1 E P10 P5 E P3@1 P2@1 C58 C59 C62 C60 C61 M34 U0 P14 P5 P10 P3@1 P9@1 P7 E P2@1 P8 P13@2 C66 C67 C70 C68 C69 M41 C75 S77 M48 U0 P15 P3@1 P5 P2@1 P6 E P8 P13@1 F P4@1 E P7 P12@1 P13@1 F E P9@1 P14 F E P2@1 P10 E";
// Retained per-step transition rewards for the first 32 actions (source receipts above).
const M15_PREFIX_REWARDS:[i32;32]=[0,1,0,0,0,0,0,0,1,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0];
const M20_PREFIX_REWARDS:[i32;32]=[0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0];
const M22_SLIME_PREFIX_REWARDS:[i32;32]=[0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0];

fn reproductions(b:&RunBatch,n:usize)->Vec<RunReproduction>{(0..n).map(|i|b.reproduction(i).unwrap()).collect()}
fn indices(singles:&[RunSession],prefixes:&[Vec<GameAction>],i:usize)->Vec<RunActionIndex>{singles.iter().zip(prefixes).map(|(s,p)|index(s,&p[i])).collect()}
fn same_step(a:&RunStepResult,b:&RunStepResult){assert_eq!((&a.boundary.observation,a.boundary.terminal,a.reward,a.terminated,a.truncated),(&b.boundary.observation,b.boundary.terminal,b.reward,b.terminated,b.truncated));}

#[test]
fn m23_batch_order_matches_singles_for_workers_one_two(){
 let configs=vec![cfg(RunAct1BossTarget::SlimeBoss,RunAct2BossTarget::Collector),cfg(RunAct1BossTarget::Guardian,RunAct2BossTarget::BronzeAutomaton)];let seeds=[Some(7),Some(11)];
 let prefixes=[transcript(M22_SEED7_SLIME_BOSS)[..60].to_vec(),transcript(M22_SEED11_GUARDIAN)[..60].to_vec()];
 for workers in [1,2]{
  let mut b=RunBatch::new_m23(configs.clone(),workers).unwrap();let mut singles=configs.iter().map(|c|RunSession::new_m23(c.clone()).unwrap()).collect::<Vec<_>>();
  let bounds=b.reset(&seeds).unwrap();for (slot,s) in singles.iter_mut().enumerate(){let one=s.reset(seeds[slot]).unwrap();assert_eq!((&bounds[slot].observation,bounds[slot].terminal),(&one.observation,one.terminal),"input-slot order");}
  let mut tokens=bounds.iter().map(|x|x.token).collect::<Vec<_>>();
  for i in 0..60{let idx=indices(&singles,&prefixes,i);let rows=b.step(&tokens,&idx).unwrap();for (slot,s) in singles.iter_mut().enumerate(){let t=s.decision().unwrap().token;let one=s.step(t,idx[slot]).unwrap();same_step(&rows[slot],&one);}tokens=rows.iter().map(|x|x.boundary.token).collect();}
  for (slot,s) in singles.iter().enumerate(){assert_eq!(b.record(slot).unwrap(),s.record().unwrap());assert_eq!(b.reproduction(slot).unwrap(),s.reproduction().unwrap());}
  assert!((0..2).all(|slot|b.reproduction(slot).unwrap().latest_decision.unwrap().unwrap().content_profile==Some(RoomScenario::M23WatcherAct2)));
 }
 // Indexed constructor rejection precedes any reset.
 assert!(matches!(RunBatch::new_m23(vec![],1),Err(RunBatchError::Input{slot:None,..})));assert!(matches!(RunBatch::new_m23(vec![config()],0),Err(RunBatchError::Input{slot:None,..})));
 assert!(matches!(RunBatch::new_m23(vec![config(),RunSessionConfig{character:Character::Silent,..config()}],1),Err(RunBatchError::Input{slot:Some(1),..})));
 assert!(matches!(RunBatch::new_m23(vec![RunSessionConfig{boss_history:RunBossHistory::Act1GuardianUnseen,..config()},config()],2),Err(RunBatchError::Input{slot:Some(0),..})));
 assert!(matches!(RunBatch::new_m22(vec![config()],1),Err(RunBatchError::Input{slot:Some(0),..})),"no mixed-profile constructor");
}

#[test]
fn m23_batch_preflight_and_full_reset_are_atomic(){
 for workers in [1,2]{
  let configs=vec![cfg(RunAct1BossTarget::SlimeBoss,RunAct2BossTarget::Champ),cfg(RunAct1BossTarget::Guardian,RunAct2BossTarget::Collector)];let prefixes=[transcript(M22_SEED7_SLIME_BOSS),transcript(M22_SEED7_GUARDIAN)];
  let mut b=RunBatch::new_m23(configs.clone(),workers).unwrap();let mut singles=configs.iter().map(|c|RunSession::new_m23(c.clone()).unwrap()).collect::<Vec<_>>();let first=b.reset(&[Some(7),Some(7)]).unwrap();for s in &mut singles{s.reset(Some(7)).unwrap();}
  let mut tokens=first.iter().map(|x|x.token).collect::<Vec<_>>();let before=reproductions(&b,2);
  assert!(matches!(b.step(&tokens[..1],&[RunActionIndex(0)]),Err(RunBatchError::Input{slot:None,..})));assert!(matches!(b.reset(&[Some(7)]),Err(RunBatchError::Input{slot:None,..})));
  let idx=indices(&singles,&prefixes,0);assert_eq!(b.step(&[tokens[0],RunDecisionToken::from([0,0])],&idx),Err(RunBatchError::Input{slot:Some(1),error:RunSessionError::StaleToken}));
  assert!(matches!(b.step(&tokens,&[idx[0],RunActionIndex(999)]),Err(RunBatchError::Input{slot:Some(1),error:RunSessionError::Input(_)})));assert!(matches!(b.step(&tokens,&[idx[0],RunActionIndex(-1)]),Err(RunBatchError::Input{slot:Some(1),..})));
  assert_eq!(reproductions(&b,2),before,"no peer action, RNG, ordinal or serial change");
  // Drive both to the actual Death of the last slot; then a healthy peer plus terminal slot rejects the whole call.
  for i in 0..146{let idx=indices(&singles,&prefixes,i);let rows=b.step(&tokens,&idx).unwrap();for (slot,s) in singles.iter_mut().enumerate(){let t=s.decision().unwrap().token;s.step(t,idx[slot]).unwrap();}tokens=rows.iter().map(|x|x.boundary.token).collect();}
  assert_eq!(b.reproduction(1).unwrap().latest_decision.unwrap().unwrap().terminal,Some(RunTerminal::Death));let before=reproductions(&b,2);
  assert_eq!(b.step(&tokens,&[index(&singles[0],&prefixes[0][146]),RunActionIndex(0)]),Err(RunBatchError::Input{slot:Some(1),error:RunSessionError::Terminal}));assert_eq!(reproductions(&b,2),before,"no terminal auto-skip");
  // Full reset only: per-slot explicit/implicit seeds keep each ordinal law.
  b.reset(&[None,Some(13)]).unwrap();assert_eq!([b.record(0).unwrap().unwrap(),b.record(1).unwrap().unwrap()].map(|r|(r.seed,r.ordinal)),[(7,1),(13,0)]);
  b.reset(&[Some(5),None]).unwrap();assert_eq!([b.record(0).unwrap().unwrap(),b.record(1).unwrap().unwrap()].map(|r|(r.seed,r.ordinal)),[(5,0),(13,1)]);
  let mut alone=RunSession::new_m23(configs[1].clone()).unwrap();alone.reset(Some(13)).unwrap();alone.reset(None).unwrap();assert_eq!(b.reproduction(1).unwrap(),alone.reproduction().unwrap());
 }
}

#[test]
fn m23_batch_postdispatch_faults_keep_indexed_peer_delivery(){
 for workers in [1,2]{
  // Step: slot1 commits then fails publication; slot0 commits and its row is indexed delivery_lost.
  let mut small=config();small.capacities.map_nodes=1;let mut b=RunBatch::new_m23(vec![config(),small],workers).unwrap();let bounds=b.reset(&[Some(7);2]).unwrap();let tokens=bounds.iter().map(|r|r.token).collect::<Vec<_>>();
  let RunBatchError::Fault{slots}=b.step(&tokens,&[RunActionIndex(0);2]).unwrap_err() else{panic!("expected post-dispatch fault")};assert_eq!(slots.len(),2);
  assert_eq!((slots[0].slot,slots[0].committed,slots[0].delivery_lost),(0,true,true));assert_eq!((slots[1].slot,slots[1].committed,slots[1].delivery_lost),(1,true,false));assert_eq!(slots[1].error,RunSessionError::DeliveryFault("post-step Run encoding failed"));
  let (r0,r1)=(b.record(0).unwrap().unwrap(),b.record(1).unwrap().unwrap());assert_eq!((r0.steps.len(),r1.steps.len()),(1,1));assert_eq!(r1.fault.as_ref().unwrap().error.capacity.as_ref().unwrap().group,"map_nodes");assert_eq!(r0.fault.as_ref().unwrap().error.kind,RunRecordErrorKind::DeliveryFault);
  for r in [&r0,&r1]{let back=RunEpisodeRecord::from_json(&r.to_json().unwrap()).unwrap();assert_eq!(&back,r);assert_eq!(replay_run(&back,r.config.capacities.clone()).unwrap().accepted_count,1);}
  let before=reproductions(&b,2);assert!(b.step(&tokens,&[RunActionIndex(0);2]).is_err());assert!(b.reset(&[Some(7);2]).is_err());assert_eq!(reproductions(&b,2),before,"retry cannot replay either action");b.close().unwrap();assert!(b.record(0).unwrap().unwrap().closed&&b.record(1).unwrap().unwrap().closed);
  // Reset: slot1 startup publication fails; slot0's reset row is delivery_lost without accepted gameplay.
  let mut tiny=config();tiny.capacities.actions=1;let mut b=RunBatch::new_m23(vec![config(),tiny],workers).unwrap();let RunBatchError::Fault{slots}=b.reset(&[Some(7);2]).unwrap_err() else{panic!("expected reset fault")};
  assert_eq!((slots[0].committed,slots[0].delivery_lost,slots[1].committed,slots[1].delivery_lost),(false,true,false,false));assert_eq!(slots[1].error,RunSessionError::Capacity{group:"actions",required:2,capacity:1});
  assert!(b.record(0).unwrap().unwrap().steps.is_empty());assert!(b.record(1).unwrap().unwrap().fault.is_some());assert!(b.reset(&[Some(7);2]).is_err());
 }
}

fn m15_config()->RunSessionConfig{RunSessionConfig{character:Character::Silent,ascension:0,partition:Partition::Test,boss_history:RunBossHistory::M15SilentHeart{act1:RunAct1BossTarget::Guardian,act2:RunAct2BossTarget::Champ,act3:RunAct3BossTarget::AwakenedOne},capacities:caps(),record_replay:true}}
fn m20_config()->RunSessionConfig{RunSessionConfig{character:Character::Defect,ascension:0,partition:Partition::Test,boss_history:RunBossHistory::M20DefectHeart{act1:RunAct1BossTarget::SlimeBoss,act2:RunAct2BossTarget::Collector,act3:RunAct3BossTarget::DonuAndDeca},capacities:caps(),record_replay:true}}
fn m22_config()->RunSessionConfig{RunSessionConfig{boss_history:RunBossHistory::Act1SlimeBossUnseen,..config()}}
fn lanes()->Vec<(&'static str,fn(RunSessionConfig)->Result<RunSession,RunSessionError>,RunSessionConfig,Vec<GameAction>,[i32;32])>{vec![
 ("M23",RunSession::new_m23,cfg(RunAct1BossTarget::SlimeBoss,RunAct2BossTarget::Collector),transcript(M22_SEED7_SLIME_BOSS)[..32].to_vec(),M22_SLIME_PREFIX_REWARDS),
 ("M22",RunSession::new_m22,m22_config(),transcript(M22_SEED7_SLIME_BOSS)[..32].to_vec(),M22_SLIME_PREFIX_REWARDS),
 ("M15",RunSession::new_m15,m15_config(),transcript(M15_SEED7_PREFIX),M15_PREFIX_REWARDS),
 ("M20",RunSession::new_m20,m20_config(),transcript(M20_SEED7_PREFIX),M20_PREFIX_REWARDS)]}

#[test]
fn m23_coexistence_preserves_watcher_and_other_character_state(){
 // Un-interleaved references from existing retained recorded actions (no chooser, no fresh historical episode).
 let mut reference=Vec::new();
 for (name,new,c,actions,rewards) in lanes(){let mut s=new(c).unwrap();s.reset(Some(7)).unwrap();let steps=drive(&mut s,&actions);assert_eq!(steps.iter().map(|x|x.reward).collect::<Vec<_>>(),rewards.to_vec(),"{name} retained rewards");
  assert!(s.record().unwrap().unwrap().steps.iter().any(|x|matches!(x.action,GameAction::Combat(_))),"{name} 32-action prefix crosses an actual combat boundary");assert!(steps.iter().all(|x|!x.terminated));reference.push((steps,s.record().unwrap(),s.reproduction().unwrap()));}
 // Interleaved single sessions, round-robin one accepted action each.
 let mut sessions=lanes().into_iter().map(|(_,new,c,actions,_)|{let mut s=new(c).unwrap();s.reset(Some(7)).unwrap();(s,actions)}).collect::<Vec<_>>();let mut got=vec![Vec::new();4];
 for i in 0..32{for (lane,(s,actions)) in sessions.iter_mut().enumerate(){got[lane].push(act(s,actions[i].clone()));}}
 for (lane,(s,_)) in sessions.iter().enumerate(){for (a,b) in got[lane].iter().zip(&reference[lane].0){same_step(a,b);}assert_eq!(s.record().unwrap(),reference[lane].1);assert_eq!(s.reproduction().unwrap(),reference[lane].2);}
 let spec=m23_run_observation_spec(&caps()).unwrap().descriptor();let stance=col(&spec,"player","stance");assert!(got[0].iter().any(|x|cell(group(&x.boundary.observation,"player"),0,stance).1==1),"M23 Watcher stance column published in combat");
 // Separate homogeneous batches, workers 1/2, agree with the same singles.
 for workers in [1,2]{for (lane,(_,new_batch,c,actions)) in [("M23",RunBatch::new_m23 as fn(Vec<RunSessionConfig>,usize)->Result<RunBatch,RunBatchError>,cfg(RunAct1BossTarget::SlimeBoss,RunAct2BossTarget::Collector),&sessions[0].1),("M22",RunBatch::new_m22,m22_config(),&sessions[1].1),("M15",RunBatch::new_m15,m15_config(),&sessions[2].1),("M20",RunBatch::new_m20,m20_config(),&sessions[3].1)].into_iter().enumerate(){
  let mut b=new_batch(vec![c.clone(),c],workers).unwrap();let mut tokens=b.reset(&[Some(7);2]).unwrap().iter().map(|x|x.token).collect::<Vec<_>>();let (_,new_single,single_config,_,_)=lanes().swap_remove(lane);let mut single=new_single(single_config).unwrap();single.reset(Some(7)).unwrap();
  for (i,a) in actions.iter().enumerate(){let idx=index(&single,a);let rows=b.step(&tokens,&[idx,idx]).unwrap();let t=single.decision().unwrap().token;single.step(t,idx).unwrap();same_step(&rows[0],&reference[lane].0[i]);same_step(&rows[1],&reference[lane].0[i]);tokens=rows.iter().map(|x|x.boundary.token).collect();}
  assert_eq!(b.record(0).unwrap(),reference[lane].1);assert_eq!(b.record(1).unwrap(),reference[lane].1);}}
}

#[test]
fn m23_encoding_actual_act2_played_rooms_use_m23_identity(){
 // Supplied M23 entries one edge before the actual captured Act2 room (labelled constructed start); the room, setup, AI and statuses are real.
 let spec=m23_run_observation_spec(&caps()).unwrap().descriptor();let (eff,cost)=(col(&spec,"combat_cards","effective_cost"),col(&spec,"combat_cards","combat_cost"));
 for f in M23PlayedFixture::ALL{
  let mut r=m23_played_fixture(f).unwrap();let mv=first_move(&r.decision().unwrap());r.step(&mv).unwrap();assert_eq!(r.decision().unwrap().content_profile,Some(RoomScenario::M23WatcherAct2));
  let (mut wound,mut flight,mut priced,mut stasis)=(0,0,0,0);
  for _ in 0..6{let d=r.decision().unwrap();let Some(c)=d.combat.as_ref() else{break};let o=encode_run_m23(&d,r.public_map().as_ref(),&caps()).unwrap_or_else(|e|panic!("{}: {e:?}",f.name()));
   for (row,en) in c.enemies.iter().enumerate(){assert_eq!(cell(group(&o,"enemies"),row,0).0,vocab_id(&spec,"enemy",en.enemy_id.as_str()));}
   let cc=group(&o,"combat_cards");for (row,card) in c.hand.iter().enumerate(){assert_eq!(cell(cc,row,eff),card.effective_cost.map_or((0,0),|v|(i64::from(v),1)));let w=card.watcher.as_ref().unwrap();assert_eq!(cell(cc,row,cost),w.combat_cost.map_or((0,0),|v|(i64::from(v),1)));priced+=usize::from(w.combat_cost.is_some());}
   wound+=c.hand.iter().chain(&c.discard).filter(|x|x.id==CardId::Wound).count();stasis+=c.stasis.len();flight+=c.enemies.iter().flat_map(|e|&e.powers).filter(|p|p.id.0=="flight").count();
   if !end_turn(&mut r){break}}
  match f{M23PlayedFixture::FlightThreeByrds=>assert!(flight>0),M23PlayedFixture::PricesSnecko=>assert!(priced>0,"Confused current combat prices encoded"),M23PlayedFixture::WoundsSlavers|M23PlayedFixture::WoundsBookOfStabbing=>assert!(wound>0,"{}",f.name()),M23PlayedFixture::StasisBronzeAutomaton=>assert!(stasis>0),M23PlayedFixture::PowerScryChosenAndByrds=>{}}
 }
}

#[test]
fn m23_numeric_actual_act3ready_and_boss_choices_encode_with_map_only_icons(){
 // Constructed boss source (labelled) then actual owner stages; the encoder sees the real Act3 public map.
 let spec=m23_run_observation_spec(&caps()).unwrap().descriptor();let map_boss=col(&spec,"context","map_boss");let room=col(&spec,"context","room_kind");
 for boss in M23DiagnosticBoss::ALL{
  for stage in [M23Diagnostic::BossRewards{boss},M23Diagnostic::BossRelicMenu{boss}]{let r=m23_diagnostic(stage).unwrap();let o=encode_run_m23(&r.decision().unwrap(),r.public_map().as_ref(),&caps()).unwrap();assert!(o.action_mask.iter().any(|m|*m==1),"{stage:?}");}
  for branch in [M22DiagnosticBranch::HolyWater,M22DiagnosticBranch::VioletLotus,M22DiagnosticBranch::SacredBark,M22DiagnosticBranch::EmptyCage,M22DiagnosticBranch::SlaversCollar,M22DiagnosticBranch::Skip]{
   let child=m23_diagnostic(M23Diagnostic::BossChild{boss,branch}).unwrap();encode_run_m23(&child.decision().unwrap(),child.public_map().as_ref(),&caps()).unwrap();
   let r=m23_diagnostic(M23Diagnostic::Act3Ready{boss,branch}).unwrap();let d=r.decision().unwrap();let map=r.public_map().unwrap();let icon=map.boss.unwrap();
   assert_eq!((d.act,d.floor,d.room.as_ref().unwrap().kind),(3,34,RoomViewKind::Act3Ready));assert!(d.candidates.is_empty()&&d.terminal.is_none());assert!(matches!(icon,EncounterId::AwakenedOne|EncounterId::TimeEater|EncounterId::DonuAndDeca));
   let o=encode_run_m23(&d,Some(&map),&caps()).unwrap();assert_eq!(o.action_mask,vec![0;caps().actions]);assert_eq!(cell(group(&o,"context"),0,map_boss),(vocab_id(&spec,"encounter",icon.as_str()),1));assert_eq!(cell(group(&o,"context"),0,room).0,vocab_id(&spec,"room","act3_ready"));
   assert!(group(&o,"map_nodes").occupied.as_ref().unwrap().iter().any(|x|*x==1),"Act3 map published, map-only");
   let mut small=caps();small.map_nodes=map.nodes.len()-1;assert_eq!(encode_run_m23(&d,Some(&map),&small),Err(RunSessionError::Capacity{group:"map_nodes",required:map.nodes.len(),capacity:map.nodes.len()-1}),"no truncation of the Act3 map");
  }
 }
}
