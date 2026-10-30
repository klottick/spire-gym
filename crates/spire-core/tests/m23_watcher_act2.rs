// Pure M23 caller and input contract. Only the policy/input modules are compiled;
// no rollout, no outcome run and no historical example test module.
#[path="../examples/m23_support/policy.rs"]mod policy;
#[path="../examples/m22_support/policy.rs"]mod m22_policy;
#[path="../examples/m23_support/inputs.rs"]mod inputs;
use policy::*;
use spire_core::{combat::{self,CombatAction,CombatPublic,CharacterCombatView,FullPlayerView,Stance},content::{Character,CardId,EncounterId,EventId,RelicId},run::*};
fn decision(actions:Vec<GameAction>)->PublicDecision{PublicDecision{content_profile:Some(RoomScenario::M23WatcherAct2),room:None,reward_total:0,act:1,floor:0,player:PublicPlayer{relic_counters:vec![],character:Character::Watcher,hp:72,max_hp:72,gold:99,relics:vec![RelicId::PureWater],potions:vec![None;3],keys:[false;3]},combat:None,combat_progress:CombatProgress::default(),last_combat:None,objects:vec![],candidates:actions.into_iter().enumerate().map(|(i,action)|Candidate{id:ChoiceId(i as u64),action,label:"fabricated public candidate",neow_offer:None,object:None}).collect(),offers:vec![],selection:None,terminal:None}}
fn room(kind:RoomViewKind,rows:Vec<(RoomChoiceKind,Option<u32>)>)->PublicDecision{let choices=rows.into_iter().enumerate().map(|(i,(kind,price))|RoomChoiceView{id:ChoiceId(i as u64),kind,price,status:RoomChoiceStatus::Offered}).collect::<Vec<_>>();let mut d=decision(choices.iter().map(|c|GameAction::Choose(c.id)).collect());d.room=Some(RoomView{kind,choices});d}
fn combat(actions:Vec<CombatAction>)->PublicDecision{let mut d=decision(actions.into_iter().map(GameAction::Combat).collect());d.combat=Some(CombatPublic{mantra_gained:Some(0),stance_choice:None,played_powers:vec![],playing_cards:vec![],stasis:vec![],content_id:"fabricated public decision".into(),turn:1,player:FullPlayerView{hp:72,max_hp:72,block:0,energy:3,max_energy:3,powers:vec![]},enemies:vec![],character:CharacterCombatView{stance:Stance::Neutral,orbs:vec![],orb_dark_damage:vec![],focus:0},hand:vec![],draw_count:0,discard:vec![],exhaust:vec![],outcome:None,hp_before_healing:None,hp_after_cleanup:None,targets:vec![],selection:None,known_draw:vec![],encounter_facing:None,inventory:None,card_offer:None});d}
fn m22p(p:&PublicCallerProgress)->m22_policy::PublicCallerProgress{m22_policy::PublicCallerProgress{ordinary_reached:p.ordinary_reached,elite_reached:p.elite_reached,purchased_shops:p.purchased_shops.clone()}}

#[test]fn m23_caller_keeps_m22_prefix_and_fixed_act2_choices(){
    // Every non-Act2-specific menu returns the accepted M22 verb.
    let p=PublicCallerProgress::default();
    let mut cases=vec![
        room(RoomViewKind::Neow,vec![(RoomChoiceKind::NeowLament,None),(RoomChoiceKind::NeowMaxHp{amount:7},None)]),
        room(RoomViewKind::CardReward,vec![(RoomChoiceKind::Skip,None),(RoomChoiceKind::Card{id:CardId::JustLucky,upgrades:0},None)]),
        room(RoomViewKind::Rewards,vec![(RoomChoiceKind::Gold(13),None),(RoomChoiceKind::CardReward,None),(RoomChoiceKind::Proceed,None)]),
        room(RoomViewKind::Shop,vec![(RoomChoiceKind::Card{id:CardId::MasterReality,upgrades:0},Some(150)),(RoomChoiceKind::Card{id:CardId::LikeWater,upgrades:0},Some(75)),(RoomChoiceKind::Leave,None)]),
        room(RoomViewKind::Campfire,vec![(RoomChoiceKind::Smith,None),(RoomChoiceKind::Rest{heal:21},None)]),
        room(RoomViewKind::BossRelicChoice,vec![(RoomChoiceKind::Relic(RelicId::SlaversCollar),None),(RoomChoiceKind::Relic(RelicId::HolyWater),None),(RoomChoiceKind::Skip,None)]),
        room(RoomViewKind::Treasure,vec![(RoomChoiceKind::Sapphire,None),(RoomChoiceKind::Relic(RelicId::Anchor),None),(RoomChoiceKind::Proceed,None)]),
        room(RoomViewKind::ProceedWithoutRelic,vec![(RoomChoiceKind::ConfirmSkip,None),(RoomChoiceKind::CancelSkip,None)]),
        room(RoomViewKind::Event(EventId::Purifier),vec![(RoomChoiceKind::ShrineRemove,None),(RoomChoiceKind::Leave,None)]),
        decision(vec![GameAction::Select(SelectionId(9)),GameAction::Select(SelectionId(4))]),
        combat(vec![CombatAction::EndTurn,CombatAction::PlayCard{instance:8,target:Some(combat::TargetId(2))}]),
        combat(vec![CombatAction::UsePotion{slot:2,target:None},CombatAction::UsePotion{slot:0,target:None}]),
    ];
    let mut act2=cases.clone();for d in &mut act2{d.act=2;d.floor=20;}cases.extend(act2);
    for d in &cases{assert_eq!(choose(&d,None,&p),m22_policy::choose(d,None,&m22p(&p)),"{:?}",d.room.as_ref().map(|r|r.kind));}
    // Joust: Murderer, then the actual Continue/Result acknowledgements, then Leave.
    for rows in [vec![(RoomChoiceKind::JoustMurderer,Some(50)),(RoomChoiceKind::JoustOwner,Some(50))],vec![(RoomChoiceKind::JoustContinue,None)],vec![(RoomChoiceKind::JoustResult{owner_wins:true},None)],vec![(RoomChoiceKind::Leave,None)]]{
        let mut d=room(RoomViewKind::Event(EventId::Joust),rows);d.act=2;assert_eq!(choose(&d,None,&p).unwrap(),GameAction::Choose(ChoiceId(0)));}
    // AncientWriting takes Simplicity when offered, Leave afterwards.
    let mut d=room(RoomViewKind::Event(EventId::AncientWriting),vec![(RoomChoiceKind::AncientElegance,None),(RoomChoiceKind::AncientSimplicity,None)]);d.act=2;assert_eq!(choose(&d,None,&p).unwrap(),GameAction::Choose(ChoiceId(1)));
    let mut d=room(RoomViewKind::Event(EventId::AncientWriting),vec![(RoomChoiceKind::AncientElegance,None)]);d.act=2;assert_eq!(choose(&d,None,&p).unwrap(),GameAction::Choose(ChoiceId(0)),"unavailable Simplicity falls back to the shared order");
    let mut d=room(RoomViewKind::Event(EventId::AncientWriting),vec![(RoomChoiceKind::Leave,None)]);d.act=2;assert_eq!(choose(&d,None,&p).unwrap(),GameAction::Choose(ChoiceId(0)));
}

#[test]fn m23_caller_route_witness_is_per_act(){
    let entered=|encounter|RunTransition{reward:0,terminal:None,committed_events:vec![CommittedEvent::CombatEntered{visit:0,encounter}]};
    let mut d1=decision(vec![]);d1.act=1;let mut d2=decision(vec![]);d2.act=2;let mv=GameAction::Move(NodeId(1));
    let steps=vec![(d1.clone(),mv.clone(),entered(EncounterId::Cultist)),(d1.clone(),mv.clone(),entered(EncounterId::GremlinNob)),(d2.clone(),mv.clone(),entered(EncounterId::Snecko))];
    let view=|act|act_progress(steps.iter().map(|(d,a,t)|(d,a,t)),act);
    assert_eq!((view(1).ordinary_reached,view(1).elite_reached),(true,true));
    assert_eq!((view(2).ordinary_reached,view(2).elite_reached),(true,false),"Act1 elite does not satisfy the Act2 witness");
    assert_eq!((view(3).ordinary_reached,view(3).elite_reached),(false,false));
    // Shop purchase history keyed by (act,floor).
    let mut shop=room(RoomViewKind::Shop,vec![(RoomChoiceKind::Card{id:CardId::LikeWater,upgrades:0},Some(75)),(RoomChoiceKind::Leave,None)]);shop.act=2;shop.floor=21;
    let bought=vec![(shop.clone(),GameAction::Choose(ChoiceId(0)),RunTransition{reward:0,terminal:None,committed_events:vec![]})];
    let p=act_progress(bought.iter().map(|(d,a,t)|(d,a,t)),2);assert!(p.purchased_shops.contains(&(2,21)));assert_eq!(choose(&shop,None,&p).unwrap(),GameAction::Choose(ChoiceId(1)));
}

#[test]fn m23_caller_input_contract_rejects_invalid_values_without_creation(){
    let args=|v:&[&str]|v.iter().map(|s|s.to_string()).collect::<Vec<_>>();
    for value in ["0","1","32768"]{assert_eq!(inputs::parse_args(args(&["--budget",value])).unwrap().budget,value.parse::<usize>().unwrap());}
    for v in [vec!["--budget","32769"],vec!["--budget","-1"],vec!["--budget","bad"],vec!["--budget","1e3"],vec!["--scenario","unknown"],vec!["--seed","bad"],vec!["--seed","13"],vec!["--unknown","7"],vec!["--seed","11","--scenario","death"],vec!["--budget"]]{assert!(inputs::parse_args(args(&v)).is_err(),"{v:?}");}
    let d=inputs::parse_args(Vec::<String>::new()).unwrap();assert_eq!((d.seed,d.budget,d.scenarios.clone(),d.nonattacking),(7,32768,inputs::SCENARIOS.to_vec(),false));
    let d=inputs::parse_args(args(&["--scenario","death"])).unwrap();assert!(d.nonattacking);assert_eq!(d.scenarios,vec![inputs::Scenario::Champ]);
    use inputs::Scenario::*;use EncounterId as E;
    for (seed,s,a1) in [(7,Champ,E::Guardian),(7,BronzeAutomaton,E::Hexaghost),(7,Collector,E::SlimeBoss),(11,Champ,E::SlimeBoss),(11,BronzeAutomaton,E::Guardian),(11,Collector,E::Hexaghost)]{assert_eq!(inputs::act1(seed,s,false),Ok(a1));}
    assert!(inputs::make_run(13,Champ,false).is_err());assert!(inputs::creation(13,Champ,false).is_err());
}

#[test]fn m23_caller_exact_six_initial_histories_have_canonical_resources(){
    for seed in inputs::SEEDS{for s in inputs::SCENARIOS{
        let p=inputs::creation(seed,s,false).unwrap();let unseen1=["Guardian","Hexaghost","SlimeBoss"].into_iter().filter(|n|!p.seen_bosses[0].contains(n)).collect::<Vec<_>>();
        let unseen2=["Champ","Automaton","Collector"].into_iter().filter(|n|!p.seen_bosses[1].contains(n)).collect::<Vec<_>>();assert_eq!((unseen1.len(),unseen2.len()),(1,1));
        assert_eq!(p.seen_bosses[2],CreationProfile::experienced().seen_bosses[2]);
        let r=inputs::make_run(seed,s,false).unwrap();let d=r.decision().unwrap();assert_eq!(d.content_profile,Some(RoomScenario::M23WatcherAct2));
        assert_eq!((d.player.hp,d.player.max_hp,d.player.gold,d.objects.len()),(72,72,99,10));assert_eq!(d.player.relics,vec![RelicId::PureWater]);assert_eq!(d.player.potions,vec![None;3]);assert!(r.public_map().is_none());
        assert!(d.candidates.len()==2,"short Neow only");
    }}
    let death=inputs::creation(7,inputs::Scenario::Champ,true).unwrap();assert_eq!(death.seen_bosses,inputs::creation(7,inputs::Scenario::Champ,false).unwrap().seen_bosses);
}

#[path="../examples/m23_support/fixtures.rs"]mod fixtures;
fn card(instance:u64,id:CardId)->combat::LiveCardView{combat::LiveCardView{instance:combat::CardInstanceId(instance),id,origin:combat::CombatCardRef::Permanent(combat::CardInstanceId(instance)),upgrades:0,misc:0,watcher:None,effective_cost:Some(1),values:None}}
fn hand(actions:Vec<CombatAction>,cards:Vec<combat::LiveCardView>)->PublicDecision{let mut d=combat(actions);d.combat.as_mut().unwrap().hand=cards;d}
#[test]fn m23_fixture_scripts_follow_plan_table_without_search(){
    use fixtures::{Script,ScriptState,script_action};use CombatAction as A;
    let none=ScriptState::default();
    assert_eq!(script_action(Script::Flight,&[],&decision(vec![]),&none),Err("fixture combat settled".into()));
    // Stance modal: first offered option; Flight uses potions by ascending slot before cards.
    let mut d=combat(vec![A::ChooseStance{stance:Stance::Calm},A::ChooseStance{stance:Stance::Wrath}]);d.combat.as_mut().unwrap().stance_choice=Some(combat::StanceChoiceView{options:vec![Stance::Wrath,Stance::Calm]});
    assert_eq!(script_action(Script::Flight,&[],&d,&none).unwrap(),GameAction::Combat(A::ChooseStance{stance:Stance::Wrath}));
    let d=hand(vec![A::PlayCard{instance:1,target:None},A::UsePotion{slot:1,target:None},A::UsePotion{slot:0,target:None},A::EndTurn],vec![card(1,CardId::Eruption)]);
    assert_eq!(script_action(Script::Flight,&[CardId::Eruption],&d,&none).unwrap(),GameAction::Combat(A::UsePotion{slot:0,target:None}));
    // Listed card order with first legal native target.
    let d=hand(vec![A::EndTurn,A::PlayCard{instance:2,target:Some(combat::TargetId(3))},A::PlayCard{instance:2,target:Some(combat::TargetId(1))},A::PlayCard{instance:1,target:None}],vec![card(1,CardId::Vigilance),card(2,CardId::Eruption)]);
    assert_eq!(script_action(Script::Wounds,&[CardId::Eruption,CardId::Vigilance],&d,&none).unwrap(),GameAction::Combat(A::PlayCard{instance:2,target:Some(combat::TargetId(3))}));
    // Prices: only Establishment/Vigilance until Confused is publicly observed.
    let d=hand(vec![A::PlayCard{instance:1,target:None},A::PlayCard{instance:2,target:None},A::EndTurn],vec![card(1,CardId::SandsOfTime),card(2,CardId::Establishment)]);
    let order=[CardId::SandsOfTime,CardId::Establishment];
    assert_eq!(script_action(Script::Prices,&order,&d,&none).unwrap(),GameAction::Combat(A::PlayCard{instance:2,target:None}));
    assert_eq!(script_action(Script::Prices,&order,&d,&ScriptState{confused_seen:true,..Default::default()}).unwrap(),GameAction::Combat(A::PlayCard{instance:1,target:None}));
    // Stasis: powers only until a real Stasis capture is observed.
    let d=hand(vec![A::PlayCard{instance:1,target:Some(combat::TargetId(1))},A::EndTurn],vec![card(1,CardId::WindmillStrike)]);
    assert_eq!(script_action(Script::Stasis,&[CardId::WindmillStrike],&d,&none).unwrap(),GameAction::Combat(A::EndTurn));
    assert_eq!(script_action(Script::Stasis,&[CardId::WindmillStrike],&d,&ScriptState{stasis_seen:true,..Default::default()}).unwrap(),d.candidates[0].action);
    // PowerScry: attacks never played; Scry select A,B, deselect A, reselect A, confirm.
    let d=hand(vec![A::PlayCard{instance:1,target:Some(combat::TargetId(1))},A::EndTurn],vec![card(1,CardId::Weave)]);
    assert_eq!(script_action(Script::PowerScry,&[CardId::Weave],&d,&none).unwrap(),GameAction::Combat(A::EndTurn));
    use combat::{SelectionView,SelectionParent,SelectionOperation,SelectionSourceZone,SelectionDisposition,CardHandle};
    let scry=|actions:Vec<A>,cands:Vec<u64>|{let mut d=combat(actions);d.combat.as_mut().unwrap().selection=Some(SelectionView{parent:SelectionParent::Card(CardHandle::parse("99").unwrap()),operation:SelectionOperation::Scry,candidates:cands.into_iter().map(|i|card(i,CardId::Dazed)).collect(),required:0,selected:vec![],disposition:SelectionDisposition::Discard,source_zone:SelectionSourceZone::Draw,minimum:0,maximum:3});d};
    let all=vec![A::SelectCard{instance:5},A::SelectCard{instance:6},A::DeselectCard{instance:5},A::ConfirmSelection];
    let expect=[A::SelectCard{instance:5},A::SelectCard{instance:6},A::DeselectCard{instance:5},A::SelectCard{instance:5},A::ConfirmSelection];
    for (k,want) in expect.iter().enumerate(){let s=ScriptState{scry_opening:Some(vec![5,6]),scry_steps:k,..Default::default()};assert_eq!(script_action(Script::PowerScry,&[],&scry(all.clone(),vec![5,6]),&s).unwrap(),GameAction::Combat(want.clone()),"step {k}");}
    let one=[A::SelectCard{instance:5},A::DeselectCard{instance:5},A::SelectCard{instance:5},A::ConfirmSelection];
    for (k,want) in one.iter().enumerate(){let s=ScriptState{scry_opening:Some(vec![5]),scry_steps:k,..Default::default()};assert_eq!(script_action(Script::PowerScry,&[],&scry(all.clone(),vec![5]),&s).unwrap(),GameAction::Combat(want.clone()));}
    assert_eq!(script_action(Script::PowerScry,&[],&scry(vec![A::ConfirmSelection],vec![]),&none).unwrap(),GameAction::Combat(A::ConfirmSelection),"confirm0");
    // Meditate: first legal pick, then confirm.
    let mut d=scry(vec![A::ConfirmSelection,A::SelectCard{instance:7}],vec![7]);d.combat.as_mut().unwrap().selection.as_mut().unwrap().operation=SelectionOperation::MeditateReturn;
    assert_eq!(script_action(Script::Wounds,&[],&d,&none).unwrap(),GameAction::Combat(A::SelectCard{instance:7}));
    d.combat.as_mut().unwrap().selection.as_mut().unwrap().selected=vec![combat::CardInstanceId(7)];assert_eq!(script_action(Script::Wounds,&[],&d,&none).unwrap(),GameAction::Combat(A::ConfirmSelection));
}

#[test]fn m23_fixture_settled_victory_is_success_not_error(){
    assert!(fixtures::settled_victory(fixtures::SETTLED,true),"a played Victory that settles the fixture is a successful completion");
    assert!(!fixtures::settled_victory(fixtures::SETTLED,false),"settled without a Victory record stays an error");
    assert!(!fixtures::settled_victory("missing public EndTurn",true),"other script errors stay errors");
    assert_eq!(fixtures::script_action(fixtures::Script::Flight,&[],&decision(vec![]),&fixtures::ScriptState::default()),Err(fixtures::SETTLED.to_string()));
}
