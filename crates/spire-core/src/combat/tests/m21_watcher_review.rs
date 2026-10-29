//! Bounded supplied integration obligations from the single independent review.
//! These are shared-owner unit fixtures, never replacement policy episodes.
use super::super::*;
use crate::{content::{CardId,Character,EncounterId,RelicId},items::{CombatItemContext,CombatStartupProfile}};
use cards::watcher::WatcherEffect;
fn handle(id:u64)->CardHandle{card_state::handle(CardInstanceId(id))}
fn power(id:&'static str,amount:i32)->FullPowerEntry{FullPowerEntry{id:FullPowerId(id),state:PowerState{amount,skip_first:false,monster_source_grace:false}}}
fn fixture(encounter:EncounterId,parent:CardId)->Combat{
    let act=match encounter{EncounterId::JawWorm=>1,EncounterId::ShieldAndSpear=>4,_=>3};
    let mut e=enemies::setup(encounter,act,0,&mut crate::run::RunRng::new([41;32],rng::Partition::Test,0),enemies::SetupContext{
        player:PlayerState{hp:300,max_hp:300,block:0,energy:3,max_energy:3,draw_per_turn:1,powers:vec![]},
        player_powers:vec![],character:Character::Watcher,cards:vec![m21_watcher_live_card(parent,CardInstanceId(1),0).unwrap()],opening_draw:1,encounter_visit:0,
    }).unwrap();
    e.cards=vec![m21_watcher_live_card(parent,CardInstanceId(1),0).unwrap()];e.opening_draw=1;
    let mut items=Some(CombatItemContext::new(vec![],vec![None;3]).with_startup_profile(CombatStartupProfile::M21Watcher));
    Combat::enter_with_item_context(e,&mut items).unwrap()
}
fn add(c:&mut Combat,id:u64,kind:CardId,zone:SelectionSourceZone){
    let card=m21_watcher_live_card(kind,CardInstanceId(id),0).unwrap();assert!(c.state.card_ids_issued.insert(handle(id)));
    let pile=match zone{SelectionSourceZone::Hand=>&mut c.state.piles.hand,SelectionSourceZone::Draw=>&mut c.state.piles.draw,SelectionSourceZone::Discard=>&mut c.state.piles.discard,_=>unreachable!()};pile.push(model::OwnedCard::Full(card));
}
fn returns(c:&Combat)->Vec<u64>{c.state.pending.iter().filter_map(|e|match &e.kind{EffectKind::Watcher(WatcherEffect::ReturnDiscard{card})=>Some(card.as_str().parse().unwrap()),_=>None}).collect()}
// Pause only the real accepted modal continuation, as existing shared-owner
// cursor tests do. No production injection surface or alternative resolver.
fn stage_confirm(c:&mut Combat){
    let selection=c.state.pending_selection.take().unwrap();c.state.fault_selection=Some(selection.clone());
    c.state.phase=c.state.watcher.resume_phase.take().unwrap();
    let source=match &selection.parent{SelectionParent::Card(id)=>Source::Card(id.clone()),SelectionParent::Power{..}=>Source::Actor(Actor::Player),_=>unreachable!()};
    c.state.pending.push_front(Effect{source,target:Some(Actor::Player),kind:EffectKind::Watcher(WatcherEffect::CommitSelection{selection,next:0})});
}
fn commit_only(c:&mut Combat){
    for _ in 0..4{if !matches!(c.state.pending.front().map(|e|&e.kind),Some(EffectKind::Watcher(WatcherEffect::CommitSelection{..}))){return}engine::dispatch_one(&mut c.state);assert!(c.fault_record().is_none(),"{:?}",c.fault_record());}panic!("finite selected cursor failed to finish")
}
fn scry_full(parent:CardId)->Combat{
    let mut c=fixture(EncounterId::JawWorm,parent);
    for id in 2..=10{add(&mut c,id,CardId::DefendPurple,SelectionSourceZone::Hand)}
    for id in [20,21]{add(&mut c,id,CardId::Weave,SelectionSourceZone::Discard)}
    for id in [22,23]{add(&mut c,id,CardId::Weave,SelectionSourceZone::Draw)}
    c.step_full(&CombatAction::PlayCard{instance:1,target:(parent==CardId::CutThroughFate).then_some(TargetId(1))}).unwrap();c
}
#[test]
fn m21_review_i3_genuine_opening_and_postselection_requests_survive_fullhand_in_order(){
    for selected in [false,true]{
        let mut c=scry_full(CardId::ThirdEye);assert_eq!(c.state.piles.hand.len(),9);
        assert_eq!(returns(&c),vec![20,21],"genuine opening O exists before any selected card moves");
        if selected{
            for id in [23,22]{c.step_full(&CombatAction::SelectCard{instance:id}).unwrap();}
            c.step_full(&CombatAction::DeselectCard{instance:23}).unwrap();c.step_full(&CombatAction::SelectCard{instance:23}).unwrap();
        }
        stage_confirm(&mut c);commit_only(&mut c);
        assert_eq!(returns(&c),if selected{vec![20,21,20,21,22,23]}else{vec![20,21,20,21]},"P preserves physical duplicates including zero confirm");
        let expected=returns(&c);let mut attempted=vec![];
        while !c.state.pending.is_empty(){
            if let Some(EffectKind::Watcher(WatcherEffect::ReturnDiscard{card}))=c.state.pending.front().map(|e|&e.kind){attempted.push(card.clone());}
            engine::dispatch_one(&mut c.state);assert!(c.fault_record().is_none(),"{:?}",c.fault_record());
        }
        c.state.fault_selection=None; // normal commit_full_selection success cleanup
        assert_eq!(attempted,expected.into_iter().map(handle).collect::<Vec<_>>());
        assert_eq!(c.state.piles.hand.len(),10);assert!(c.state.piles.hand.iter().any(|c|c.handle()==handle(20)));
        assert!(c.state.piles.discard.iter().any(|c|c.handle()==handle(21)));assert!(engine::validate_runtime(&c.state).is_ok(),"{:?}",engine::validate_runtime(&c.state));
    }
}
#[test]
fn m21_review_i3_later_draw_and_creation_stay_ahead_of_bottom_weave_attempts(){
    let mut c=scry_full(CardId::CutThroughFate);assert_eq!(returns(&c),vec![20,21]);
    // A later bottom producer is placed after the already constructed O requests.
    let kind=cards::watcher::create_action(&c.state,CardId::Safety,1).unwrap();
    enemies::act2::bottom(&mut c.state,Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind});
    stage_confirm(&mut c);commit_only(&mut c);assert_eq!(returns(&c),vec![20,21,20,21]);
    let work=c.state.pending.iter().map(|e|&e.kind).collect::<Vec<_>>();
    assert!(work.iter().position(|k|matches!(k,EffectKind::Draw(1))).unwrap()<work.iter().position(|k|matches!(k,EffectKind::Watcher(WatcherEffect::ReturnDiscard{..}))).unwrap());
    assert_ne!(engine::resolve(&mut c.state).status,TransitionStatus::Fault);
    assert!(c.state.piles.hand.iter().any(|c|c.handle()==handle(23)),"real source draw consumes final hand slot before O");
    assert!(c.state.piles.discard.iter().any(|c|c.full().is_some_and(|c|c.id==CardId::Safety)),"later creation overflows rather than moving ahead of draw");
    for id in [20,21]{assert!(c.state.piles.discard.iter().any(|c|c.handle()==handle(id)));}
}

// Same prepare/detach/effects/finish/decision ownership as step_full, paused at
// the actual queue boundary for physical flag/cursor assertions.
fn stage_play(c:&mut Combat,instance:u64,target:Option<TargetId>){
    let prepared=cards::prepare(c,&CombatAction::PlayCard{instance,target}).unwrap();
    let source=Source::Card(handle(instance));let mut work=prepared.effects;
    c.state.in_play=Some(c.state.piles.hand.remove(prepared.hand_index));c.state.player.energy=prepared.remaining_energy;c.state.phase=Phase::PlayerAction;
    cards::capture_silent_targets(&mut c.state,&mut work).unwrap();
    c.state.pending.push_back(Effect{source:source.clone(),target:None,kind:EffectKind::Hook(Hook::CardPlayed)});
    c.state.pending.extend(work);c.state.pending.push_back(Effect{source:source.clone(),target:None,kind:EffectKind::Hook(Hook::CardAfterUse)});
    c.state.pending.push_back(Effect{source,target:None,kind:EffectKind::FinishCard});c.state.pending.push_back(Effect{source:Source::None,target:None,kind:EffectKind::Decision});
}
fn full_card(c:&Combat,id:u64)->&LiveCard{piles::owned_full_cards(&c.state).find(|c|c.instance.0==id).unwrap()}
#[test]
fn m21_review_i3_meditate_hand9_auto_marks_all_but_selected_marks_only_moved(){
    for auto in [true,false]{for reverse in [false,true]{
        let mut c=fixture(EncounterId::JawWorm,CardId::Meditate);
        *c.state.piles.hand[0].full_mut().unwrap()=m21_watcher_live_card(CardId::Meditate,CardInstanceId(1),1).unwrap();
        for id in 2..=10{add(&mut c,id,CardId::DefendPurple,SelectionSourceZone::Hand)}
        for id in 20..=if auto{21}else{22}{add(&mut c,id,CardId::DefendPurple,SelectionSourceZone::Discard)}
        let census=c.state.card_ids_issued.clone();stage_play(&mut c,1,None);
        engine::dispatch_one(&mut c.state);engine::dispatch_one(&mut c.state);assert!(c.fault_record().is_none());
        if auto{
            assert!(c.state.pending_selection.is_none());assert!(full_card(&c,20).local.retain&&full_card(&c,21).local.retain);
            assert!(c.state.piles.hand.iter().any(|c|c.handle()==handle(20)));assert!(c.state.piles.discard.iter().any(|c|c.handle()==handle(21)));
        }else{
            assert!(c.state.pending_selection.is_some());let order=if reverse{[21,20]}else{[20,21]};
            for id in order{c.step_full(&CombatAction::SelectCard{instance:id}).unwrap();}
            stage_confirm(&mut c);commit_only(&mut c);
            assert!(c.state.piles.hand.iter().any(|c|c.handle()==handle(order[0])));assert!(full_card(&c,order[0]).local.retain);
            for id in [order[1],22]{assert!(c.state.piles.discard.iter().any(|c|c.handle()==handle(id)));assert!(!full_card(&c,id).local.retain);}
        }
        assert_eq!(c.state.piles.hand.len(),10);assert_eq!(piles::owned_full_cards(&c.state).map(|c|handle(c.instance.0)).collect::<std::collections::BTreeSet<_>>(),census);
        assert_ne!(engine::resolve(&mut c.state).status,TransitionStatus::Fault,"{:?}",c.fault_record());c.state.fault_selection=None;
        assert_eq!(c.state.turn,2);assert!(engine::validate_runtime(&c.state).is_ok(),"{:?}",engine::validate_runtime(&c.state));
    }}
}
#[test]
fn m21_review_i3_all_fixed_children_capture_early_and_hand_late_without_visual_ids(){
    for kind in [CardId::Miracle,CardId::Smite,CardId::Safety]{for early in [false,true]{for late in [false,true]{
        let mut c=fixture(EncounterId::JawWorm,CardId::DefendPurple);
        for id in 2..=9{add(&mut c,id,CardId::DefendPurple,SelectionSourceZone::Hand)}
        if early{c.state.player_full_powers.push(power("master_reality",-1))}
        let captured=cards::watcher::create_action(&c.state,kind,3).unwrap();c.state.player_full_powers.retain(|p|p.id.0!="master_reality");
        if late{c.state.player_full_powers.push(power("master_reality",-1))}
        let issued=c.state.card_ids_issued.clone();let events=c.state.hook_events.len();
        c.state.pending.push_back(Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:captured});
        c.state.pending.push_back(Effect{source:Source::None,target:None,kind:EffectKind::Decision});
        assert_ne!(engine::resolve(&mut c.state).status,TransitionStatus::Fault,"{:?}",c.fault_record());
        let hand=c.state.piles.hand.last().unwrap().full().unwrap();assert_eq!((hand.id,hand.instance.0,hand.upgrades),(kind,10,u32::from(early||late)));
        assert_eq!(c.state.piles.discard.iter().map(|c|{let c=c.full().unwrap();(c.id,c.instance.0,c.upgrades)}).collect::<Vec<_>>(),vec![(kind,11,u32::from(early)),(kind,12,u32::from(early))]);
        assert_eq!(c.state.card_ids_issued.difference(&issued).cloned().collect::<Vec<_>>(),[10,11,12].map(handle));
        assert!(c.state.hook_events[events..].iter().all(|e|!matches!(e.hook,Hook::CardDiscarded|Hook::CardManuallyDiscarded)),"overflow inserts captured argument without discard copied hooks");
        assert!(engine::validate_runtime(&c.state).is_ok());
    }}}
}
#[test]
fn m21_review_i3_merely_queued_master_reality_does_not_activate_early_template(){
    for kind in [CardId::Miracle,CardId::Smite,CardId::Safety]{
        let mut c=fixture(EncounterId::JawWorm,CardId::DefendPurple);for id in 2..=10{add(&mut c,id,CardId::DefendPurple,SelectionSourceZone::Hand)}
        c.state.pending.push_back(Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::ApplyFullPower{id:FullPowerId("master_reality"),amount:-1}});
        let captured=cards::watcher::create_action(&c.state,kind,2).unwrap();
        c.state.pending.push_back(Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:captured});c.state.pending.push_back(Effect{source:Source::None,target:None,kind:EffectKind::Decision});
        assert_ne!(engine::resolve(&mut c.state).status,TransitionStatus::Fault);assert!(c.state.player_full_powers.iter().any(|p|p.id.0=="master_reality"));
        assert_eq!(c.state.piles.discard.iter().map(|c|c.full().unwrap().upgrades).collect::<Vec<_>>(),vec![0,0],"queued installation was absent at construction, overflow display upgrade cannot mutate argument");
    }
}

#[test]
fn m21_review_i3_surrounded_top_cleanup_precedes_real_op_returns_and_derived_damage(){
    let mut c=fixture(EncounterId::ShieldAndSpear,CardId::CutThroughFate);
    let intent=ActorIntentId::Full(FullIntentId::from_name("shield_bash"));
    c.state.enemies[0].intent_id=intent.clone();c.state.enemies[0].selected_move=intent.clone();c.state.enemies[0].move_history=vec![intent];effects::refresh_enemy_values(&mut c.state).unwrap();
    add(&mut c,20,CardId::Weave,SelectionSourceZone::Discard);add(&mut c,21,CardId::DefendPurple,SelectionSourceZone::Draw);
    c.step_full(&CombatAction::PlayCard{instance:1,target:Some(TargetId(2))}).unwrap();
    assert_eq!(c.state.facing,Some(PlayerFacing::Right));
    assert_eq!(c.state.player_full_powers.iter().find(|p|p.id.0=="surrounded").unwrap().state.amount,1,"accepted native marker, never the original -1 sentinel");
    assert_eq!(returns(&c),vec![20]);stage_confirm(&mut c);commit_only(&mut c);assert_eq!(returns(&c),vec![20,20]);
    // O has not executed at the modal. Its successful return owns the real
    // source refresh; inspect that boundary before Decision can refresh again.
    for _ in 0..10{if matches!(c.state.pending.front().map(|e|&e.kind),Some(EffectKind::Watcher(WatcherEffect::ReturnDiscard{..}))){break}engine::dispatch_one(&mut c.state);assert!(c.fault_record().is_none());}
    engine::dispatch_one(&mut c.state);assert!(c.state.piles.hand.iter().any(|c|c.handle()==handle(20)));
    assert_eq!((c.state.enemies[0].intent_damage,c.state.enemies[1].intent_damage),(Some(18),Some(5)),"successful O refresh normalizes actual native facing/slot-derived damage");
    assert_eq!(returns(&c),vec![20]);
    let spear=Actor::Enemy(c.state.enemies[1].instance_id.clone());
    c.state.pending.push_front(Effect{source:Source::Actor(Actor::Player),target:Some(spear),kind:EffectKind::Damage(DamagePayload::Fixed(999))});
    let mut cleanup=vec![];
    for _ in 0..20{
        if matches!(c.state.pending.front().map(|e|&e.kind),Some(EffectKind::Watcher(WatcherEffect::ReturnDiscard{..}))){break}
        if let Some(EffectKind::RemoveFullPower(id))=c.state.pending.front().map(|e|&e.kind){cleanup.push(id.0);}
        engine::dispatch_one(&mut c.state);assert!(c.fault_record().is_none(),"{:?}",c.fault_record());
    }
    assert_eq!(cleanup,vec!["surrounded","back_attack"]);assert_eq!(returns(&c),vec![20]);
    assert_eq!(c.state.facing,Some(PlayerFacing::Left));assert_eq!(c.state.enemies[0].intent_damage,Some(12));
    assert!(view::encounter_facing(&c.state).unwrap().unwrap().enemies.iter().all(|e|!e.back_attack));
    engine::dispatch_one(&mut c.state);assert!(c.state.piles.hand.iter().any(|c|c.handle()==handle(20)));
    assert_ne!(engine::resolve(&mut c.state).status,TransitionStatus::Fault);c.state.fault_selection=None;assert!(c.public_full().is_ok());
}
#[test]
fn m21_review_i3_terminal_and_playerdeath_cancel_op_returns_preserving_parent_for_exit(){
    for death in [false,true]{
        let mut c=scry_full(CardId::ThirdEye);let ids=c.state.card_ids_issued.clone();stage_confirm(&mut c);commit_only(&mut c);assert_eq!(returns(&c),vec![20,21,20,21]);
        let target=if death{Actor::Player}else{Actor::Enemy(c.state.enemies[0].instance_id.clone())};
        c.state.pending.push_front(Effect{source:Source::Actor(Actor::Player),target:Some(target),kind:EffectKind::Damage(DamagePayload::HpLoss(999))});
        assert_eq!(engine::resolve(&mut c.state).status,TransitionStatus::Terminal);c.state.fault_selection=None;
        assert_eq!(c.state.outcome,Some(if death{Outcome::Death}else{Outcome::Victory}));assert!(returns(&c).is_empty());assert!(c.state.pending.is_empty());
        assert_eq!(c.state.in_play.as_ref().unwrap().handle(),handle(1));for id in [20,21]{assert!(c.state.piles.discard.iter().any(|c|c.handle()==handle(id)));}
        assert_eq!(piles::owned_full_cards(&c.state).map(|c|handle(c.instance.0)).collect::<std::collections::BTreeSet<_>>(),ids);
        assert!(engine::validate_runtime(&c.state).is_ok());assert!(c.exit().is_ok());let frozen=c.clone();assert!(c.step_full(&CombatAction::ConfirmSelection).is_err());assert_eq!(c,frozen);
    }
}
#[test]
fn m21_review_i3_halfdead_keeps_live_return_continuation_and_rebirth_owner(){
    let mut c=fixture(EncounterId::AwakenedOne,CardId::ThirdEye);
    for index in [0,1]{let target=Actor::Enemy(c.state.enemies[index].instance_id.clone());c.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(target),kind:EffectKind::Damage(DamagePayload::Fixed(999))}],None).unwrap();}
    add(&mut c,20,CardId::Weave,SelectionSourceZone::Discard);add(&mut c,21,CardId::DefendPurple,SelectionSourceZone::Draw);
    c.step_full(&CombatAction::PlayCard{instance:1,target:None}).unwrap();stage_confirm(&mut c);commit_only(&mut c);
    let target=Actor::Enemy(c.state.enemies[2].instance_id.clone());c.state.pending.push_front(Effect{source:Source::Actor(Actor::Player),target:Some(target),kind:EffectKind::Damage(DamagePayload::Fixed(999))});
    assert_ne!(engine::resolve(&mut c.state).status,TransitionStatus::Fault);c.state.fault_selection=None;
    assert_eq!(c.state.outcome,None);assert_eq!(c.state.enemies[2].life,ActorLife::HalfDead);assert!(c.state.in_play.is_none());
    assert!(c.public_full().unwrap().hand.iter().any(|c|c.instance.0==20));assert!(engine::validate_runtime(&c.state).is_ok());
    c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.state.enemies[2].life,ActorLife::Active);assert_eq!(c.state.enemies[2].hp,300);
}
#[test]
fn m21_review_i3_first_native_return_fault_retains_op_prefix_and_cannot_retry(){
    let mut c=scry_full(CardId::ThirdEye);stage_confirm(&mut c);commit_only(&mut c);
    // Reach actual bottom O work only after the physical parent has finished.
    for _ in 0..8{if matches!(c.state.pending.front().map(|e|&e.kind),Some(EffectKind::Watcher(WatcherEffect::ReturnDiscard{..}))){break}engine::dispatch_one(&mut c.state);}
    assert!(c.state.in_play.is_none());assert_eq!(returns(&c),vec![20,21,20,21]);
    c.state.resolved_effects=99998;engine::dispatch_one(&mut c.state);engine::dispatch_one(&mut c.state);
    assert_eq!(c.fault_record().unwrap().kind,FaultKind::EffectGuard);
    assert!(c.state.piles.hand.iter().any(|c|c.handle()==handle(20)));assert!(c.state.piles.discard.iter().any(|c|c.handle()==handle(21)));
    assert!(matches!(c.state.current.as_ref().map(|e|&e.kind),Some(EffectKind::Watcher(WatcherEffect::ReturnDiscard{card})) if *card==handle(21)));
    assert_eq!(returns(&c),vec![20,21]);let frozen=c.clone();assert!(c.step_full(&CombatAction::ConfirmSelection).is_err());engine::dispatch_one(&mut c.state);assert_eq!(c,frozen);
}
#[test]
fn m21_review_i3_startup_child_commits_before_real_opening_owner_fault(){
    // A supplied native dispatch-budget checkpoint, not an original-frame cap
    // equivalence or a published-state capacity failure. Run the actual pre-draw
    // hook/creator/OpeningHand owners, pausing only to place the native guard.
    let mut c=fixture(EncounterId::JawWorm,CardId::DefendPurple);
    c.state.piles.draw.append(&mut c.state.piles.hand);let original=c.state.piles.draw.clone();let issued=c.state.card_ids_issued.clone();
    c.state.item_context=Some(CombatItemContext::new(vec![RelicId::PureWater],vec![None;3]).with_startup_profile(CombatStartupProfile::M21Watcher));
    c.state.phase=Phase::PlayerStart;
    c.state.pending.push_back(Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::Hook(Hook::CombatStartedPreDraw)});
    c.state.pending.push_back(Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::OpeningHand{count:1}});
    c.state.pending.push_back(Effect{source:Source::None,target:None,kind:EffectKind::Decision});
    for _ in 0..5{if matches!(c.state.pending.front().map(|e|&e.kind),Some(EffectKind::OpeningHand{..})){break}engine::dispatch_one(&mut c.state);assert!(c.fault_record().is_none());}
    assert_eq!(c.state.piles.hand.len(),1);let child=c.state.piles.hand[0].full().unwrap();assert_eq!((child.id,child.instance.0),(CardId::Miracle,2));
    assert_eq!(c.state.card_ids_issued.difference(&issued).cloned().collect::<Vec<_>>(),vec![handle(2)]);assert_eq!(c.state.piles.draw,original);
    c.state.resolved_effects=99999;engine::dispatch_one(&mut c.state);
    assert_eq!(c.fault_record().unwrap().kind,FaultKind::EffectGuard);assert!(matches!(c.state.current.as_ref().map(|e|&e.kind),Some(EffectKind::OpeningHand{count:1})));
    assert_eq!(c.state.piles.draw,original);assert_eq!(c.state.piles.hand.len(),1);assert_eq!(c.state.pending.back().unwrap().kind,EffectKind::Decision);
    assert!(engine::validate_runtime(&c.state).is_ok());let frozen=c.clone();assert!(c.public_full().is_err());assert!(c.step_full(&CombatAction::EndTurn).is_err());engine::dispatch_one(&mut c.state);assert_eq!(c,frozen);
}
