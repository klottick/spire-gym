use super::super::*;
use crate::content::{CardId,Character,EncounterId};

pub(super) fn entry(encounter:EncounterId)->CombatEntry {
    enemies::setup(encounter,1,0,&mut crate::run::RunRng::new([31;32],rng::Partition::Test,0),enemies::SetupContext {
        player:PlayerState{hp:80,max_hp:80,block:0,energy:3,max_energy:3,draw_per_turn:1,powers:vec![]},
        player_powers:vec![],character:Character::Ironclad,
        cards:vec![permanent_live_card(CardId::Strike,CardInstanceId(1),0)],opening_draw:1,encounter_visit:0,
    }).unwrap()
}
pub(super) fn amount(c:&Combat,slot:usize,id:&str)->i32 {c.state.enemies[slot].full_powers.iter().filter(|p|p.id.0==id).map(|p|p.state.amount).sum()}
fn intent(c:&Combat,slot:usize)->&str{c.state.enemies[slot].intent_id.as_str()}
pub(super) fn hit(c:&mut Combat,slot:usize,n:i32){let owner=c.state.enemies[slot].instance_id.clone();c.enqueue_native_effects(vec![Effect {
    source:Source::Actor(Actor::Player),target:Some(Actor::Enemy(owner)),kind:EffectKind::Damage(DamagePayload::Fixed(n))}],None).unwrap();}

#[test]
fn lagavulin_natural_wake_has_three_no_hit_turns_then_two_attacks_and_siphon() {
    let mut c=Combat::enter(entry(EncounterId::Lagavulin)).unwrap();
    assert_eq!(c.state.enemies[0].block,8);assert_eq!(amount(&c,0,"metallicize"),8);
    for next in ["sleep","sleep","attack"] {c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.state.player.hp,80);assert_eq!(intent(&c,0),next);}
    assert_eq!(amount(&c,0,"metallicize"),0);
    for (hp,next) in [(62,"attack"),(44,"siphon"),(44,"attack")] {c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.state.player.hp,hp);assert_eq!(intent(&c,0),next);}
    let powers=&c.state.player_full_powers;assert!(powers.iter().any(|p|p.id.0=="dexterity"&&p.state.amount== -1));assert!(powers.iter().any(|p|p.id.0=="strength"&&p.state.amount== -1));
}

#[test]
fn lagavulin_only_hp_loss_wakes_and_siphon_artifact_blocks_dexterity_first() {
    let mut c=Combat::enter(entry(EncounterId::Lagavulin)).unwrap();hit(&mut c,0,8);assert_eq!(intent(&c,0),"sleep");
    hit(&mut c,0,1);assert_eq!(intent(&c,0),"stunned");assert_eq!(amount(&c,0,"metallicize"),0);
    c.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::ApplyFullPower{id:FullPowerId("artifact"),amount:1}}],None).unwrap();
    for (hp,next) in [(80,"attack"),(62,"attack"),(44,"siphon"),(44,"attack")] {c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.state.player.hp,hp);assert_eq!(intent(&c,0),next);}
    assert!(!c.state.player_full_powers.iter().any(|p|matches!(p.id.0,"artifact"|"dexterity")));
    assert!(c.state.player_full_powers.iter().any(|p|p.id.0=="strength"&&p.state.amount== -1));
}

#[test]
fn sentries_start_in_order_and_survivors_keep_their_own_alternation() {
    let mut c=Combat::enter(entry(EncounterId::ThreeSentries)).unwrap();
    assert_eq!((intent(&c,0),intent(&c,1),intent(&c,2)),("bolt","beam","bolt"));
    for i in 0..3 {assert_eq!(amount(&c,i,"artifact"),1);assert!((38..=42).contains(&c.state.enemies[i].hp));}
    hit(&mut c,0,42);let id=c.state.enemies[2].instance_id.clone();
    c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.state.player.hp,71);
    assert_eq!((intent(&c,1),intent(&c,2)),("bolt","beam"));assert_eq!(c.state.enemies[2].instance_id,id);
    let dazed=c.state.piles.hand.iter().chain(&c.state.piles.draw).chain(&c.state.piles.discard).filter(|card|card.full().unwrap().id.as_str()=="dazed").count();assert_eq!(dazed,2);
    c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.state.player.hp,62);assert_eq!((intent(&c,1),intent(&c,2)),("beam","bolt"));
}

#[test]
fn three_louse_have_independent_setup_streams_bites_and_curl_up() {
    let e=entry(EncounterId::ThreeLouse);assert_eq!(e.setup.enemies.len(),3);assert_eq!(e.setup.enemy_damage_bases.len(),3);
    for (i,a) in e.setup.enemies.iter().enumerate() {assert!((10..=17).contains(&a.hp));assert!((3..=7).contains(&a.powers[0].state.amount));assert!((5..=7).contains(&e.setup.enemy_damage_bases[&(i as u32,FullIntentId::Bite)]));for b in &e.setup.enemies[..i]{assert_ne!(a.ai_rng.id(),b.ai_rng.id());}}
    let mut c=Combat::enter(e).unwrap();hit(&mut c,1,1);assert!(c.state.enemies[1].block>=3);assert_eq!(c.state.enemies[0].block,0);assert_eq!(c.state.enemies[2].block,0);
    c.step_full(&CombatAction::EndTurn).unwrap();assert!((59..=65).contains(&c.state.player.hp));
}

#[test]
fn lagavulin_awake_event_profile_is_siphon_first_without_intrinsic_block() {
    let mut e=entry(EncounterId::Lagavulin);e.setup.enemies[0].profile=enemies::EnemyProfile::LagavulinAwake;e.setup.enemies[0].intent=FullIntentId::from_name("siphon");
    e.setup.enemies[0].initial_memory=None; // This explicit profile override uses the profile default, not the captured sleeping memory.
    let mut c=Combat::enter(e).unwrap();assert_eq!(c.state.enemies[0].block,0);assert_eq!(amount(&c,0,"metallicize"),0);
    for (hp,next) in [(80,"attack"),(62,"attack"),(44,"siphon")] {c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.state.player.hp,hp);assert_eq!(intent(&c,0),next);}
}

#[test]
fn burning_strength_and_rounded_hp_apply_to_every_member_before_lament() {
    use enemies::{BurningBuff as B,apply_burning_buff};use crate::{content::RelicId,items::CombatItemContext};
    for buff in [B::Strength,B::MaxHp] {
        let mut e=entry(EncounterId::ThreeSentries);
        for (actor,hp) in e.setup.enemies.iter_mut().zip([38,39,42]) {actor.hp=hp;actor.max_hp=hp;}
        apply_burning_buff(&mut e,buff).unwrap();
        let mut items=Some(CombatItemContext::new(vec![RelicId::NeowsLament],vec![]));
        let c=Combat::enter_with_item_context(e,&mut items).unwrap();
        for (i,max_hp) in [48,49,53].into_iter().enumerate(){assert_eq!(c.state.enemies[i].hp,1);assert_eq!(amount(&c,i,"strength"),if buff==B::Strength{2}else{0});assert_eq!(c.state.enemies[i].max_hp,if buff==B::MaxHp{max_hp}else{[38,39,42][i]});}
        assert_eq!(c.item_context().unwrap().counters().get(&RelicId::NeowsLament),Some(&2));
    }
}

#[test]
fn burning_metallicize_stacks_with_lagavulin_and_opening_removes_only_eight() {
    let mut e=entry(EncounterId::Lagavulin);enemies::apply_burning_buff(&mut e,enemies::BurningBuff::Metallicize).unwrap();
    let mut c=Combat::enter(e).unwrap();assert_eq!(amount(&c,0,"metallicize"),12);
    hit(&mut c,0,9);assert_eq!(amount(&c,0,"metallicize"),4);assert_eq!(intent(&c,0),"stunned");
    c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.state.enemies[0].block,4);assert_eq!(amount(&c,0,"metallicize"),4);
}

#[test]
fn burning_monster_regeneration_heals_caps_keeps_its_amount_and_never_revives() {
    let mut e=entry(EncounterId::ThreeSentries);enemies::apply_burning_buff(&mut e,enemies::BurningBuff::Regeneration).unwrap();
    let mut c=Combat::enter(e).unwrap();let max=c.state.enemies[1].max_hp;
    hit(&mut c,0,42);hit(&mut c,1,4);c.step_full(&CombatAction::EndTurn).unwrap();
    assert_eq!(c.state.enemies[0].hp,0);assert_eq!(c.state.enemies[1].hp,max-1);
    c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.state.enemies[1].hp,max);assert_eq!(c.state.enemies[0].hp,0);
    assert_eq!(amount(&c,1,"regeneration"),3);
}

#[test]
fn burning_startup_fault_keeps_lament_buffs_and_first_owned_work_without_retry() {
    use crate::{content::RelicId,items::CombatItemContext};
    let mut e=entry(EncounterId::ThreeSentries);e.cards[0]=permanent_live_card(CardId::Strike,CardInstanceId(u64::MAX),0);
    enemies::apply_burning_buff(&mut e,enemies::BurningBuff::Strength).unwrap();
    let mut items=Some(CombatItemContext::new(vec![RelicId::NeowsLament,RelicId::PureWater],vec![]));
    let Err(CombatEntryFailure::Faulted{mut combat,fault})=Combat::enter_with_item_context(e,&mut items)else{panic!("expected retained native generated-ID fault")};
    assert!(items.is_none());for i in 0..3{assert_eq!(combat.state.enemies[i].hp,1);assert_eq!(amount(&combat,i,"strength"),2);}
    assert_eq!(combat.item_context().unwrap().counters().get(&RelicId::NeowsLament),Some(&2));
    let before=combat.snapshot();assert_eq!(combat.step_full(&CombatAction::EndTurn),Err(crate::run::RunError::Faulted(fault)));assert_eq!(combat.snapshot(),before);
}
