//! Constructed actual Run startup prefixes, with typed shared-owner inspection.
use super::super::*;
use crate::{run::{diagnostics::{m22_diagnostic,M22Diagnostic},GameAction,CombatProgress,RunError},content::{CardId,RelicId}};
#[test]
fn m22_anchor_correction_preserves_shared_startup_profiles_and_strict_decision_fault(){
    use crate::{content::{Character,EncounterId},items::{CombatItemContext,CombatStartupProfile,NativeBattleContext},run::RunRng};
    for lane in 0..4{
        let character=match lane{0|1=>Character::Watcher,2=>Character::Defect,_=>Character::Ironclad};
        let card=match character{Character::Watcher=>CardId::DefendPurple,Character::Defect=>CardId::DefendBlue,_=>CardId::Defend};
        let e=enemies::setup(EncounterId::JawWorm,1,0,&mut RunRng::new([7;32],rng::Partition::Test,0),enemies::SetupContext{player:PlayerState{hp:48,max_hp:72,block:0,energy:3,max_energy:3,draw_per_turn:5,powers:vec![]},player_powers:vec![],character,cards:(1..=10).map(|id|permanent_live_card(card,CardInstanceId(id),0)).collect(),opening_draw:5,encounter_visit:0}).unwrap();
        let relics=if lane==1{vec![RelicId::PureWater,RelicId::Vajra]}else{vec![RelicId::Anchor,RelicId::Vajra,RelicId::Lantern,RelicId::BloodVial]};
        let context=CombatItemContext::new(relics.clone(),vec![None;3]);
        let context=match lane{0=>context.with_m22_profile(),1=>context.with_startup_profile(CombatStartupProfile::M21Watcher),2=>context.with_battle_context(NativeBattleContext{m20:true,..Default::default()}),_=>context};
        let mut owner=Some(context);let mut c=Combat::enter_with_item_context(e.clone(),&mut owner).unwrap();assert!(owner.is_none());
        let v=c.public_full().unwrap();assert_eq!((v.player.hp,v.player.block,v.player.energy),if lane==1{(48,0,3)}else{(50,10,4)});assert_eq!(v.hand.len(),if lane==1{6}else{5});
        assert_eq!(v.player.powers.iter().find(|p|p.id.0=="strength").unwrap().amount,1);assert_eq!(c.item_context().unwrap().relics(),relics);
        assert_eq!(c.state.phase,Phase::Decision);assert!(c.state.pending.is_empty()&&c.state.current.is_none());
        if lane==1{
            let invalid=CombatItemContext::new(vec![RelicId::PureWater,RelicId::Anchor],vec![None;3]).with_startup_profile(CombatStartupProfile::M21Watcher);let mut invalid=Some(invalid);let before=invalid.clone();assert!(matches!(Combat::enter_with_item_context(e,&mut invalid),Err(CombatEntryFailure::Rejected(_))));assert_eq!(invalid,before,"M21 inventory admission remains finite");
        }
        // Deliberately malformed queue checks the unchanged strict marker and first-fault owner.
        let block=c.state.player.block;c.state.phase=Phase::PlayerStart;
        c.state.pending.extend([Effect{source:Source::None,target:None,kind:EffectKind::Decision},Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::GainBlock(10)}]);
        assert_eq!(engine::resolve(&mut c.state).status,TransitionStatus::Fault);assert_eq!(c.state.player.block,block);assert_eq!(c.state.current.as_ref().unwrap().kind,EffectKind::Decision);assert_eq!(c.state.pending.len(),1);
        assert_eq!(c.state.fault.as_ref().unwrap().message,"decision before quiescence");let before=c.clone();engine::resolve(&mut c.state);assert_eq!(c,before);
    }
}
#[test]
fn m22_holy_water_actual_run_first_child_identity_fault_preserves_detached_census(){
    let mut r=m22_diagnostic(M22Diagnostic::BeforeStartupFault).unwrap();let action=r.decision().unwrap().candidates.iter().find(|c|matches!(c.action,GameAction::Move(_))).unwrap().action.clone();
    let deck=r.state().deck.clone();assert!(matches!(r.step(&action),Err(RunError::Faulted(_))));
    let s=r.state();assert!(s.relics.is_empty()&&s.potions.is_empty()&&s.relic_counters.is_empty());assert_eq!(s.deck,deck);assert_eq!(s.combat_progress,CombatProgress{entered:1,settled:0});
    let c=s.combat.as_ref().unwrap();assert_eq!(c.state.piles.hand.len(),1);assert_eq!(c.state.piles.draw.len(),10);assert!(c.state.piles.discard.is_empty());
    let child=c.state.piles.hand[0].full().unwrap();assert_eq!((child.id,child.instance),(CardId::Miracle,CardInstanceId(u64::MAX)));assert!(matches!(child.origin,CombatCardRef::CombatLocal(_)));
    assert!(matches!(c.state.current.as_ref().map(|e|&e.kind),Some(EffectKind::Watcher(cards::watcher::WatcherEffect::Create{count:3,next:1,..}))));
    assert!(c.state.pending.iter().any(|e|matches!(e.kind,EffectKind::OpeningHand{count:5})));
    let items=c.item_context().unwrap();assert_eq!(items.relics(),&[RelicId::HolyWater]);assert_eq!(items.potions(),&[None;3]);
    let mut originals=c.state.piles.draw.iter().filter_map(|c|c.full()).filter_map(|c|match c.origin{CombatCardRef::Permanent(id)=>Some(id.0),_=>None}).collect::<Vec<_>>();originals.sort_unstable();let mut expected=deck.iter().map(|c|c.instance).collect::<Vec<_>>();expected.sort_unstable();assert_eq!(originals,expected);
    let before=r.clone();assert!(matches!(r.step(&action),Err(RunError::Faulted(_))));assert!(r==before);
}
#[test]
fn m22_boss_bark_held_miracle_potency_reinitializes_once_and_overflow_keeps_generated_custody(){
    use crate::run::{diagnostics::{M22DiagnosticBoss as B,M22DiagnosticBranch as R},RunRng};use crate::content::{Character,EncounterId,PotionId};
    let mut r=m22_diagnostic(M22Diagnostic::BossChild{boss:B::Guardian,branch:R::SacredBark}).unwrap();let original=r.state().deck.clone();assert_eq!(r.state().potions[0],Some(PotionId::BottledMiracle));
    // Supplied next-combat startup, not playable post-cutoff progression.
    let mut e=enemies::setup(EncounterId::JawWorm,1,0,&mut RunRng::new([7;32],rng::Partition::Test,0),enemies::SetupContext{player:PlayerState{hp:31,max_hp:79,block:0,energy:3,max_energy:3,draw_per_turn:5,powers:vec![]},player_powers:vec![],character:Character::Watcher,cards:original.iter().map(|c|permanent_live_card(c.id,CardInstanceId(c.instance),c.upgrades)).collect(),opening_draw:10,encounter_visit:1}).unwrap();e.opening_draw=10;
    let items=crate::items::CombatItemContext::take_from_run(r.state_mut_for_test()).with_m22_profile();assert_eq!(items.potion_potency(0),Some(4));let mut items=Some(items);let mut c=Combat::enter_with_item_context(e,&mut items).unwrap();assert_eq!(c.state.piles.hand.len(),10);assert_eq!(c.item_context().unwrap().potion_potency(0),Some(4));c.step_full(&CombatAction::UsePotion{slot:0,target:None}).unwrap();
    assert_eq!(c.state.piles.hand.len(),10);assert_eq!(c.state.piles.discard.iter().filter_map(|c|c.full()).filter(|c|c.id==CardId::Miracle).count(),4);assert_eq!(c.item_context().unwrap().potions()[0],None);
    let cards=piles::owned_full_cards(&c.state).collect::<Vec<_>>();assert_eq!(cards.iter().filter(|c|c.id==CardId::Miracle).count(),5);assert!(cards.iter().filter(|c|c.id==CardId::Miracle).all(|c|c.upgrades==0&&matches!(c.origin,CombatCardRef::CombatLocal(_))));let ids=cards.iter().filter_map(|c|if let CombatCardRef::Permanent(id)=c.origin{Some(id.0)}else{None}).collect::<std::collections::BTreeSet<_>>();assert_eq!(ids,original.iter().map(|c|c.instance).collect());assert_eq!(r.state().deck,original);
}
