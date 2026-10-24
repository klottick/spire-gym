//! Breaks protected: omitted/guessed resolved HP work, wrong event ownership,
//! cloned-state capture leakage and missing restoration of scoped observation.
use super::super::*;
use super::super::{effects, observe};
use super::support::base_scenario;
use crate::content::{CardId, EncounterId, Character, PotionId, RelicId};

fn combat() -> Combat { Combat::new(&base_scenario()).unwrap() }

#[test]
fn observed_half_dead_retains_native_reviving_lifecycle() {
    let mut c=Combat::enter(super::m8_shapes::entry(EncounterId::ThreeDarklings)).unwrap();
    let actor=Actor::Enemy(c.state.enemies[0].instance_id.clone());let hp=c.state.enemies[0].hp;
    let session=observe::Session::combat(&c);
    c.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(actor.clone()),kind:EffectKind::Damage(DamagePayload::Fixed(hp))}],None).unwrap();
    let observed=session.finish(());
    assert!(c.public_full().unwrap().enemies[0].half_dead);assert!(c.public_full().unwrap().outcome.is_none());
    let lifecycle=observed.effects.iter().find_map(|e|match &e.kind{ResolvedEffectKind::LifeChange{actor:a,after,..}if a==&actor=>Some(format!("{after:?}")),_=>None});
    assert_eq!(lifecycle.as_deref(),Some("HalfDead"));
}
fn work(kind: EffectKind) -> Effect {
    Effect { source: Source::None, target: Some(Actor::Player), kind }
}

#[test]
fn observed_damage_math() {
    for (hp, block, incoming, absorbed, lost, overkill) in [
        (10, 5, 7, 5, 2, 0), (10, 8, 7, 7, 0, 0), (2, 0, 9, 0, 2, 7),
    ] {
        let mut c = combat();
        c.state.player.hp = hp;
        c.state.player.block = block;
        let session = observe::Session::combat(&c);
        effects::apply_effect(&mut c.state, &work(EffectKind::Damage(DamagePayload::Fixed(incoming)))).unwrap();
        let observed = session.finish(());
        let damage = observed.effects.iter().find_map(|e| match e.kind {
            ResolvedEffectKind::Damage { incoming, absorbed_block, hp_lost, overkill, .. } =>
                Some((incoming, absorbed_block, hp_lost, overkill)), _ => None,
        }).expect("actual committed hit must be observed");
        assert_eq!(damage, (incoming, absorbed, lost, Some(overkill)));
        assert_eq!(c.state.player.hp, hp - lost);
    }
}

#[test]
fn observed_heal_order() {
    let mut c = combat();
    c.state.player.hp = 10;
    let session = observe::Session::combat(&c);
    effects::apply_effect(&mut c.state, &work(EffectKind::Damage(DamagePayload::Fixed(7)))).unwrap();
    effects::apply_effect(&mut c.state, &work(EffectKind::Heal(4))).unwrap();
    let observed = session.finish(());
    assert_eq!(c.state.player.hp, 7);
    assert!(matches!(observed.effects[0].kind,
        ResolvedEffectKind::Damage { hp_before:10, hp_after:3, hp_lost:7, .. }));
    assert!(matches!(observed.effects[1].kind,
        ResolvedEffectKind::Heal { hp_before:3, hp_after:7, amount:4 }));
}

#[test]
fn observed_sources_and_moves() {
    let mut c = combat();
    let actor = Actor::Enemy(c.state.enemies[0].instance_id.clone());
    c.step(&Action::EndTurn).unwrap(); // actual Incantation; upcoming Attack
    let observed = c.step_observed(&Action::EndTurn);
    observed.result.unwrap();
    assert!(observed.effects.iter().any(|e| e.source == Source::Actor(actor.clone())
        && matches!(&e.kind, ResolvedEffectKind::EnemyMove { actor:a, move_id }
            if a == &actor && move_id.as_str() == "attack")));
    assert!(observed.effects.iter().any(|e| e.source == Source::Actor(actor.clone())
        && e.target == Some(Actor::Player)
        && matches!(e.kind, ResolvedEffectKind::Damage { incoming:6, hp_lost:6, .. })));
    assert!(observed.effects.windows(2).all(|w| w[0].sequence < w[1].sequence));
}

#[test]
fn observed_scope() {
    let mut c = combat();
    let mut foreign = c.clone();
    let session = observe::Session::combat(&c);
    effects::apply_effect(&mut foreign.state, &work(EffectKind::Heal(1))).unwrap();
    effects::apply_effect(&mut c.state, &work(EffectKind::Heal(2))).unwrap();
    let nested = foreign.step_observed(&Action::EndTurn);
    nested.result.unwrap();
    let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _inner = observe::Session::combat(&foreign);
        panic!("test scope unwind");
    }));
    assert!(panic.is_err());
    effects::apply_effect(&mut c.state, &work(EffectKind::Heal(3))).unwrap();
    let observed = session.finish(());
    assert_eq!(observed.effects.len(), 2, "only this state and outer scope");
    assert!(matches!(observed.effects[0].kind, ResolvedEffectKind::Heal { amount:2, .. }));
    assert!(matches!(observed.effects[1].kind, ResolvedEffectKind::Heal { amount:3, .. }));
    assert!(!observe::enabled(&c.state));
    let before = c.snapshot();
    let mut plain = c.clone();
    let action = Action::EndTurn;
    let observed = c.step_observed(&action);
    assert_eq!(observed.result, plain.step(&action));
    assert_eq!(c.snapshot(), plain.snapshot());
    assert_ne!(c.snapshot(), before);
    assert!(!observe::enabled(&plain.state));
}

#[test]
fn observed_card_block_energy_and_power() {
    let mut c = combat();
    let result = c.step_observed(&Action::PlayCard { card_instance_id:CardHandle::parse("2").unwrap(), target_id:None });
    result.result.unwrap();
    assert!(result.effects.iter().any(|e| matches!(e.kind, ResolvedEffectKind::Energy { before:3, after:2 })));
    assert!(result.effects.iter().any(|e| matches!(e.kind, ResolvedEffectKind::Block { before:0, after:5 })));
    assert!(result.effects.iter().any(|e| matches!(&e.kind, ResolvedEffectKind::CardMove { card,before:PublicZone::Hand,after:PublicZone::InPlay } if card.as_str()=="2")));
    assert!(result.effects.iter().any(|e| matches!(&e.kind, ResolvedEffectKind::CardMove { card,before:PublicZone::InPlay,after:PublicZone::Discard } if card.as_str()=="2")));
    let result = c.step_observed(&Action::PlayCard { card_instance_id:CardHandle::parse("3").unwrap(), target_id:Some(c.state.enemies[0].instance_id.clone()) });
    result.result.unwrap();
    assert!(result.effects.iter().any(|e| matches!(e.kind, ResolvedEffectKind::Power { id:FullPowerId("vulnerable"),before:None,after:Some(2) })));
}

#[test]
fn observed_selection_resume() {
    let mut entry = super::m5_elites::entry(EncounterId::Cultist);
    entry.cards = [CardId::BurningPact,CardId::Strike,CardId::Defend].into_iter().enumerate()
        .map(|(i,id)|permanent_live_card(id,CardInstanceId(i as u64+1),0)).collect();
    entry.opening_draw = 3;
    let mut c = Combat::enter(entry).unwrap();
    let first = c.step_full_observed(&CombatAction::PlayCard { instance:1,target:None });
    first.result.unwrap();
    assert_eq!(first.boundaries[0].parent_card.as_ref().unwrap().as_str(), "1");
    assert_eq!(first.boundaries[0].kind, CombatBoundaryKind::Selection);
    let pick = c.legal_actions_full().unwrap()[0].clone();
    let second = c.step_full_observed(&pick);
    second.result.unwrap();
    assert_eq!(first.effects.iter().chain(&second.effects).filter(|e|
        matches!(&e.kind, ResolvedEffectKind::CardMove { card,after:PublicZone::Discard,.. } if card.as_str()=="1")).count(), 1);
    assert!(second.effects.iter().any(|e| matches!(&e.kind,
        ResolvedEffectKind::CardMove { after:PublicZone::Exhaust, .. }) && e.source==Source::Card(CardHandle::parse("1").unwrap())));
    assert_eq!(c.state.player.energy, 2);
}

#[test]
fn observed_terminal_settlement() {
    use crate::run::*;
    let mut run = Run::from_authored_combat(RunConfig { character:Character::Ironclad,ascension:0 },
        AuthoredRunStart { hp:40,max_hp:80,gold:99,deck:vec![crate::run::CardInstance { instance:1,id:CardId::Strike,upgrades:0 }],
            relics:vec![RelicId::BurningBlood],potions:vec![Some(PotionId::FirePotion);3],relic_counters:Default::default() },
        RunRng::new([7;32],rng::Partition::Test,0)).unwrap();
    run.enter_combat(EncounterId::JawWorm).unwrap();
    let mut plain=run.clone();
    let mut final_observed=None;
    for slot in 0..3 {
        let action=GameAction::Combat(CombatAction::UsePotion { slot,target:Some(TargetId(1)) });
        let observed=run.step_observed(&action);
        assert_eq!(observed.result, plain.step(&action));
        assert_eq!(run,plain);
        if run.decision().unwrap().combat.is_none() { final_observed=Some(observed); break; }
    }
    let observed=final_observed.expect("real killing potion settles");
    let terminal=observed.boundaries.iter().find(|b|b.kind==CombatBoundaryKind::Terminal).expect("terminal roster before settlement");
    let PublicCombatSnapshot::Native(public)=&terminal.public else {panic!("native public")};
    assert_eq!((public.enemies[0].hp,public.player.hp,public.outcome),(0,46,Some(Outcome::Victory)));
    assert!(observed.effects.iter().any(|e| matches!(e.kind,ResolvedEffectKind::Heal { hp_before:40,hp_after:46,amount:6 })));
    assert_eq!(run.decision().unwrap().player.hp,46);
}

#[test]
fn observed_spawn_identity() {
    let mut c=Combat::enter(super::m6_ordinary::entry(EncounterId::GremlinLeader)).unwrap();
    let original=c.public_full().unwrap().enemies;
    let session=observe::Session::combat(&c);
    let owner=c.state.enemies[2].instance_id.clone();
    c.enqueue_native_effects(vec![Effect { source:Source::Actor(Actor::Enemy(owner.clone())),target:None,
        kind:EffectKind::SpawnEnemy(SpawnRequest { summoner:owner,slot:0,species:crate::content::EnemyId::GremlinFat }) }],None).unwrap();
    let summoned=c.public_full().unwrap().enemies[3].clone();
    super::m5_elites::hit(&mut c,2,200);
    let result=session.finish(());
    assert_ne!(summoned.instance_id,original[0].instance_id);
    assert_eq!(summoned.logical_slot,0);
    assert!(result.effects.iter().any(|e|matches!(&e.kind, ResolvedEffectKind::LifeChange { actor:Actor::Enemy(id),before:None,after:PublicLife::Active } if id==&summoned.instance_id)));
    assert!(result.effects.iter().any(|e|matches!(&e.kind, ResolvedEffectKind::LifeChange { actor:Actor::Enemy(id),after:PublicLife::Escaped,.. } if id==&summoned.instance_id)));
}

#[test]
fn observed_rejected_and_faulted() {
    let mut c=combat();
    let before=c.snapshot();
    let result=c.step_observed(&Action::PlayCard { card_instance_id:CardHandle::parse("999").unwrap(),target_id:None });
    assert_eq!(result.result,Err(StepError::InvalidAction));
    assert!(result.effects.is_empty()&&result.boundaries.is_empty());
    assert_eq!(c.snapshot(),before);
    c.state.phase=Phase::EnemyTurn;
    c.state.pending.extend([work(EffectKind::GainBlock(5)),work(EffectKind::Draw(-1))]);
    let session=observe::Session::combat(&c);
    assert_eq!(super::super::engine::resolve(&mut c.state).status,TransitionStatus::Fault);
    let result=session.finish(());
    assert_eq!(result.effects.len(),1,"real committed block only, not the invalid draw");
    assert!(matches!(result.effects[0].kind,ResolvedEffectKind::Block { before:0,after:5 }));
    assert!(result.boundaries.is_empty());
}

#[test]
fn observed_entry_owns_only_real_run_combat() {
    use crate::run::*;
    let mut profile=CreationProfile::experienced();profile.neow_eligible=false;
    let mut run=Run::new_room_scenario(RunConfig {character:Character::Ironclad,ascension:0},profile,
        RunRng::new([7;32],rng::Partition::Test,0),RoomScenario::M4Act1Prefix).unwrap();
    let neow=run.decision().unwrap().room.unwrap().choices.into_iter().find(|c|
        matches!(c.kind,RoomChoiceKind::NeowMaxHp{..})).unwrap().id;
    run.step(&GameAction::Choose(neow)).unwrap();
    let action=GameAction::Move(NodeId(4));
    let mut plain=run.clone();
    let observed=run.step_observed(&action);
    assert_eq!(observed.result,plain.step(&action));assert_eq!(run,plain);
    let entry=observed.boundaries.iter().find(|b|b.kind==CombatBoundaryKind::Entry).expect("actual entry boundary");
    let PublicCombatSnapshot::Native(public)=&entry.public else {panic!("native entry")};
    assert_eq!(public,run.decision().unwrap().combat.as_ref().unwrap());
    assert_eq!(public.hand.len(),5);
}

#[test]
fn observed_replacement_keeps_parent_and_separate_children() {
    let mut c=Combat::enter(super::m5_elites::entry(EncounterId::SlimeBoss)).unwrap();
    let parent=c.state.enemies[0].instance_id.clone();
    super::m5_elites::hit(&mut c,0,71);
    let observed=c.step_full_observed(&CombatAction::EndTurn);observed.result.unwrap();
    assert_eq!(c.public_full().unwrap().enemies.len(),3);
    assert!(observed.effects.iter().any(|e|matches!(&e.kind,
        ResolvedEffectKind::LifeChange {actor:Actor::Enemy(id),after:PublicLife::Replaced,..} if id==&parent)));
    let children=observed.effects.iter().filter_map(|e|match &e.kind {
        ResolvedEffectKind::LifeChange {actor:Actor::Enemy(id),before:None,after:PublicLife::Active}=>Some(id.clone()),_=>None,
    }).collect::<Vec<_>>();
    assert_eq!(children.len(),2);assert_ne!(children[0],children[1]);assert!(!children.contains(&parent));
}

#[test]
fn observed_native_multihit_and_callback_order() {
    let mut entry=super::m6_ordinary::entry(EncounterId::ThreeByrds);
    entry.cards[0]=permanent_live_card(CardId::TwinStrike,CardInstanceId(1),0);
    let mut c=Combat::enter(entry).unwrap();
    let observed=c.step_full_observed(&CombatAction::PlayCard {instance:1,target:Some(TargetId(1))});observed.result.unwrap();
    let hits=observed.effects.iter().filter(|e|matches!(e.kind,ResolvedEffectKind::Damage {incoming:2,hp_lost:2,..})).collect::<Vec<_>>();
    assert_eq!(hits.len(),2);assert!(hits.iter().all(|e|e.source==Source::Card(CardHandle::parse("1").unwrap())));
    let flight=observed.effects.iter().find(|e|matches!(e.kind,ResolvedEffectKind::Power {id:FullPowerId("flight"),before:Some(3),after:Some(2)})).unwrap();
    assert!(hits[0].sequence<hits[1].sequence&&hits[1].sequence<flight.sequence,
        "Flight reductions are actual bottom-queued work, not a guessed immediate decrement");
}

#[test]
fn observed_run_item_heals_commit_separately_not_during_menu_preparation() {
    use crate::run::*;
    let mut profile=CreationProfile::experienced();profile.neow_eligible=false;
    let mut run=Run::new_room_scenario(RunConfig{character:Character::Ironclad,ascension:0},profile,
        RunRng::new([7;32],rng::Partition::Test,0),RoomScenario::M4Act1Prefix).unwrap();
    // Diagnostic valid native owner, not normal-start rollout evidence.
    let choice=run.decision().unwrap().room.unwrap().choices.into_iter().find(|c|c.kind==RoomChoiceKind::NeowLament).unwrap().id;
    run.step(&GameAction::Choose(choice)).unwrap();
    let state=run.state_mut_for_test();state.hp=20;state.relics.push(RelicId::ToyOrnithopter);state.potions[0]=Some(PotionId::BloodPotion);
    let observed=run.step_observed(&GameAction::UsePotion{slot:0,target:None});observed.result.unwrap();
    let heals=observed.effects.iter().filter_map(|e|match e.kind {ResolvedEffectKind::Heal{hp_before,hp_after,amount}=>Some((hp_before,hp_after,amount)),_=>None}).collect::<Vec<_>>();
    assert_eq!(heals,vec![(20,36,16),(36,41,5)]);
    assert_eq!(run.decision().unwrap().player.hp,41);
}
