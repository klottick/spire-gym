//! Supplied authored-run census diagnostic; not normal-profile acquisition proof.
use crate::{combat::{Actor,CombatAction,DamagePayload,Effect,EffectKind,Source},content::{CardId,Character,EncounterId},run::*};

#[test]
fn m10_batch_exhaustion_preserves_permanent_deck_identities_through_real_settlement(){
    let start=AuthoredRunStart{hp:80,max_hp:80,gold:99,
        deck:(1..=3).map(|instance|CardInstance{ misc:0,instance,id:CardId::Strike,upgrades:0}).collect(),
        relics:vec![],potions:vec![None;3],relic_counters:Default::default()};
    let mut run=Run::from_authored_combat(RunConfig{character:Character::Ironclad,ascension:0},start,
        RunRng::new([37;32],crate::combat::rng::Partition::Test,0)).unwrap();
    let deck=vec![CardInstance{ misc:0,instance:1,id:CardId::SecondWind,upgrades:0},
        CardInstance{ misc:0,instance:2,id:CardId::Defend,upgrades:0},CardInstance{ misc:0,instance:3,id:CardId::Inflame,upgrades:0}];
    run.state.deck=deck.clone();
    run.enter_combat(EncounterId::JawWorm).unwrap();
    let initial=run.decision().unwrap().combat.unwrap();
    assert_eq!(initial.hand.len(),3);
    run.choose_combat(CombatAction::PlayCard{instance:1,target:None}).unwrap();
    let after_batch=run.decision().unwrap().combat.unwrap();
    for instance in [2_u64,3]{
        assert_eq!(after_batch.exhaust.iter().filter(|card|card.instance.0==instance).count(),1);
    }
    let combat=run.state_mut_for_test().combat.as_mut().unwrap();
    let target=Actor::Enemy(combat.public_full().unwrap().enemies[0].instance_id.clone());
    combat.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(target),kind:EffectKind::Damage(DamagePayload::Fixed(2000))}],None).unwrap();
    assert_eq!(combat.public_full().unwrap().outcome,Some(crate::combat::Outcome::Victory));
    run.settle_combat().unwrap();
    assert_eq!(run.state.deck,deck,"all three permanent source/hand identities remain once in the Run master deck despite batch movement");
    assert_eq!(run.state.combat_progress,CombatProgress{entered:1,settled:1});
    assert_eq!(run.state.last_combat.as_ref().unwrap().outcome,crate::combat::Outcome::Victory);
    assert!(run.state.combat.is_none());
}
