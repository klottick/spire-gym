// Only the pure policy module is compiled here; no historical example test modules.
#[path="../examples/m22_support/policy.rs"]mod policy;
use policy::*;
use spire_core::{combat::{self,CombatAction,CombatPublic,CharacterCombatView,FullPlayerView,Stance},content::{Character,CardId,RelicId},run::*};
fn decision(actions:Vec<GameAction>)->PublicDecision{PublicDecision{content_profile:Some(RoomScenario::M22WatcherAct1),room:None,reward_total:0,act:1,floor:0,player:PublicPlayer{relic_counters:vec![],character:Character::Watcher,hp:72,max_hp:72,gold:99,relics:vec![RelicId::PureWater],potions:vec![None;3],keys:[false;3]},combat:None,combat_progress:CombatProgress::default(),last_combat:None,objects:vec![],candidates:actions.into_iter().enumerate().map(|(i,action)|Candidate{id:ChoiceId(i as u64),action,label:"fabricated public candidate",neow_offer:None,object:None}).collect(),offers:vec![],selection:None,terminal:None}}
fn room(kind:RoomViewKind,rows:Vec<(RoomChoiceKind,Option<u32>)>)->PublicDecision{let choices=rows.into_iter().enumerate().map(|(i,(kind,price))|RoomChoiceView{id:ChoiceId(i as u64),kind,price,status:RoomChoiceStatus::Offered}).collect::<Vec<_>>();let mut d=decision(choices.iter().map(|c|GameAction::Choose(c.id)).collect());d.room=Some(RoomView{kind,choices});d}
fn combat(actions:Vec<CombatAction>)->PublicDecision{let mut d=decision(actions.into_iter().map(GameAction::Combat).collect());d.combat=Some(CombatPublic{mantra_gained:Some(0),stance_choice:None,played_powers:vec![],playing_cards:vec![],stasis:vec![],content_id:"fabricated public decision".into(),turn:1,player:FullPlayerView{hp:72,max_hp:72,block:0,energy:3,max_energy:3,powers:vec![]},enemies:vec![],character:CharacterCombatView{stance:Stance::Neutral,orbs:vec![],orb_dark_damage:vec![],focus:0},hand:vec![],draw_count:0,discard:vec![],exhaust:vec![],outcome:None,hp_before_healing:None,hp_after_cleanup:None,targets:vec![],selection:None,known_draw:vec![],encounter_facing:None,inventory:None,card_offer:None});d}
#[test]fn m22_caller_neow_card_reward_and_claim_order_are_public(){
    let p=PublicCallerProgress::default();let d=room(RoomViewKind::Neow,vec![(RoomChoiceKind::NeowLament,None),(RoomChoiceKind::NeowMaxHp{amount:7},None)]);assert_eq!(choose(&d,None,&p).unwrap(),GameAction::Choose(ChoiceId(1)));
    let d=room(RoomViewKind::CardReward,vec![(RoomChoiceKind::Skip,None),(RoomChoiceKind::Card{id:CardId::JustLucky,upgrades:0},None),(RoomChoiceKind::Card{id:CardId::Nirvana,upgrades:0},None)]);assert_eq!(choose(&d,None,&p).unwrap(),GameAction::Choose(ChoiceId(1)));
    let d=room(RoomViewKind::Rewards,vec![(RoomChoiceKind::Gold(13),None),(RoomChoiceKind::CardReward,None),(RoomChoiceKind::Proceed,None)]);assert_eq!(choose(&d,None,&p).unwrap(),GameAction::Choose(ChoiceId(0)));
}
#[test]fn m22_caller_shop_buys_one_first_affordable_card_then_leaves(){
    let mut p=PublicCallerProgress::default();let mut d=room(RoomViewKind::Shop,vec![(RoomChoiceKind::Card{id:CardId::MasterReality,upgrades:0},Some(150)),(RoomChoiceKind::Card{id:CardId::LikeWater,upgrades:0},Some(75)),(RoomChoiceKind::Purge,Some(75)),(RoomChoiceKind::Leave,None)]);d.floor=4;assert_eq!(choose(&d,None,&p).unwrap(),GameAction::Choose(ChoiceId(1)));p.purchased_shops.insert((1,4));assert_eq!(choose(&d,None,&p).unwrap(),GameAction::Choose(ChoiceId(3)));p.purchased_shops.clear();d.player.gold=20;assert_eq!(choose(&d,None,&p).unwrap(),GameAction::Choose(ChoiceId(3)));
}
#[test]fn m22_caller_rest_threshold_permanent_and_relic_priorities(){
    let p=PublicCallerProgress::default();let mut d=room(RoomViewKind::Campfire,vec![(RoomChoiceKind::Smith,None),(RoomChoiceKind::Rest{heal:21},None)]);d.player.hp=35;assert_eq!(choose(&d,None,&p).unwrap(),GameAction::Choose(ChoiceId(1)));d.player.hp=36;assert_eq!(choose(&d,None,&p).unwrap(),GameAction::Choose(ChoiceId(0)));
    let d=room(RoomViewKind::BossRelicChoice,vec![(RoomChoiceKind::Relic(RelicId::SlaversCollar),None),(RoomChoiceKind::Relic(RelicId::SacredBark),None),(RoomChoiceKind::Relic(RelicId::VioletLotus),None),(RoomChoiceKind::Relic(RelicId::HolyWater),None),(RoomChoiceKind::Skip,None)]);assert_eq!(choose(&d,None,&p).unwrap(),GameAction::Choose(ChoiceId(3)));
    let d=room(RoomViewKind::Treasure,vec![(RoomChoiceKind::Sapphire,None),(RoomChoiceKind::Relic(RelicId::Anchor),None),(RoomChoiceKind::Proceed,None)]);assert_eq!(choose(&d,None,&p).unwrap(),GameAction::Choose(ChoiceId(1)));
    let d=decision(vec![GameAction::Select(SelectionId(9)),GameAction::Select(SelectionId(4))]);assert_eq!(choose(&d,None,&p).unwrap(),GameAction::Select(SelectionId(9)));
}
#[test]fn m22_caller_combat_preserves_potion_slots_native_card_targets_and_calm(){
    let p=PublicCallerProgress::default();let d=combat(vec![CombatAction::EndTurn,CombatAction::PlayCard{instance:8,target:Some(combat::TargetId(2))},CombatAction::PlayCard{instance:7,target:Some(combat::TargetId(1))}]);assert_eq!(choose(&d,None,&p).unwrap(),d.candidates[1].action);
    let d=combat(vec![CombatAction::UsePotion{slot:2,target:None},CombatAction::PlayCard{instance:8,target:None},CombatAction::UsePotion{slot:0,target:None}]);assert_eq!(choose(&d,None,&p).unwrap(),d.candidates[2].action);
    let mut d=combat(vec![CombatAction::ChooseStance{stance:Stance::Wrath},CombatAction::ChooseStance{stance:Stance::Calm}]);d.combat.as_mut().unwrap().stance_choice=Some(combat::StanceChoiceView{options:vec![Stance::Wrath,Stance::Calm]});assert_eq!(choose(&d,None,&p).unwrap(),d.candidates[1].action);
}
#[test]fn m22_caller_scry_zero_and_meditate_confirm_else_first_select(){
    use combat::{SelectionView,SelectionParent,SelectionOperation,SelectionSourceZone,SelectionDisposition,CardHandle};let p=PublicCallerProgress::default();
    let mut d=combat(vec![CombatAction::DeselectCard{instance:9},CombatAction::SelectCard{instance:8},CombatAction::ConfirmSelection]);d.combat.as_mut().unwrap().selection=Some(SelectionView{parent:SelectionParent::Card(CardHandle::parse("99").unwrap()),operation:SelectionOperation::Scry,candidates:vec![],required:0,selected:vec![],disposition:SelectionDisposition::Discard,source_zone:SelectionSourceZone::Draw,minimum:0,maximum:2});assert_eq!(choose(&d,None,&p).unwrap(),d.candidates[2].action);
    d.combat.as_mut().unwrap().selection.as_mut().unwrap().operation=SelectionOperation::MeditateReturn;assert_eq!(choose(&d,None,&p).unwrap(),d.candidates[2].action);d.candidates.pop();assert_eq!(choose(&d,None,&p).unwrap(),d.candidates[1].action);
}
#[test]fn m22_caller_route_quota_and_node_ties_use_shared_public_routes(){
    use spire_core::run::map::{PublicMap,PublicMapNode};
    let node=|id,kind,edges:Vec<u64>|PublicMapNode{id:NodeId(id),floor:id as u32,x:0,y:0,kind,resolved_room:None,edges:edges.into_iter().map(NodeId).collect(),visited:false,revealed:true,burning:None};
    let map=PublicMap{boss:Some(spire_core::content::EncounterId::Guardian),act:1,current:None,winged_moves:0,nodes:vec![node(1,MapNodeKind::Monster,vec![3]),node(2,MapNodeKind::Monster,vec![3]),node(3,MapNodeKind::Elite,vec![4]),node(4,MapNodeKind::Boss,vec![])]};
    let mut d=decision(vec![GameAction::Move(NodeId(2)),GameAction::Move(NodeId(1))]);d.room=Some(RoomView{kind:RoomViewKind::Map,choices:vec![]});
    assert_eq!(choose(&d,Some(&map),&PublicCallerProgress::default()).unwrap(),GameAction::Move(NodeId(1)));
    assert!(choose(&d,None,&PublicCallerProgress::default()).is_err());
}
#[path="../examples/m22_support/inputs.rs"]mod inputs;
#[test]fn m22_caller_input_contract_rejects_invalid_values_without_creation(){
    let args=|v:&[&str]|v.iter().map(|s|s.to_string()).collect::<Vec<_>>();
    for value in ["0","1","32768"]{assert_eq!(inputs::parse_args(args(&["--budget",value])).unwrap().budget,value.parse::<usize>().unwrap());}
    for v in [vec!["--budget","32769"],vec!["--budget","-1"],vec!["--budget","bad"],vec!["--scenario","unknown"],vec!["--seed","bad"],vec!["--unknown","7"]]{assert!(inputs::parse_args(args(&v)).is_err());}
    let d=inputs::parse_args(Vec::<String>::new()).unwrap();assert_eq!((d.seed,d.budget,d.scenarios.len(),d.nonattacking),(7,32768,3,false));let d=inputs::parse_args(args(&["--scenario","death"])).unwrap();assert!(d.nonattacking);assert_eq!(d.scenarios,vec![inputs::Scenario::Guardian]);
}
#[test]fn m22_caller_exact_six_initial_histories_have_canonical_resources(){
    for seed in [7,11]{for scenario in inputs::SCENARIOS{let r=inputs::make_run(seed,scenario).unwrap();let d=r.decision().unwrap();assert_eq!(d.player.character,Character::Watcher);assert_eq!((d.player.hp,d.player.max_hp,d.player.gold),(72,72,99));assert_eq!(d.objects.len(),10);assert_eq!(d.player.relics,vec![RelicId::PureWater]);assert_eq!(d.player.potions,vec![None;3]);assert!(r.public_map().is_none());assert_eq!(d.candidates.len(),2);}}
}
