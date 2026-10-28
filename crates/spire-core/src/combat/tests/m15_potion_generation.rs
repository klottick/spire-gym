use super::super::*;
use crate::{content::{Character, PotionId, RelicId}, items::CombatItemContext};

fn brew_combat(slot_count: usize, sozu: bool) -> Combat {
    let mut potions = vec![None; slot_count];
    potions[0] = Some(PotionId::EntropicBrew);
    let mut relics = Vec::new();
    if slot_count == 5 { relics.push(RelicId::PotionBelt); }
    if sozu { relics.push(RelicId::Sozu); }
    let mut entry=super::m8_shapes::entry(crate::content::EncounterId::ThreeShapes);
    entry.setup.character=Character::Silent;
    Combat::enter_with_item_context(entry,&mut Some(CombatItemContext::new(relics,potions).with_m15_profile())).unwrap()
}

#[test]
fn m15_combat_brew_captures_five_slots_samples_under_sozu_and_blocks_each_obtain() {
    let mut combat = brew_combat(5, true);
    let card_rng_before = combat.state.card_rng.clone().unwrap();

    combat.step_full(&CombatAction::UsePotion{slot:0,target:None}).unwrap();

    let items = combat.state.item_context.as_ref().unwrap();
    assert_eq!(items.potions(), &[None, None, None, None, None]);
    assert!(combat.state.card_rng.as_ref().unwrap().word_index() > card_rng_before.word_index());
    assert_eq!(combat.state.resolved_effects, 6, "one Brew action plus five separately dispatched obtains");
    assert!(combat.state.pending.is_empty());
    assert!(combat.state.current.is_none());
}

#[test]
fn m15_combat_brew_fills_a_vacated_three_slot_belt_without_sozu() {
    let mut combat = brew_combat(3, false);
    combat.step_full(&CombatAction::UsePotion{slot:0,target:None}).unwrap();

    let potions = combat.state.item_context.as_ref().unwrap().potions();
    assert_eq!(potions.len(), 3);
    assert!(potions.iter().all(Option::is_some), "Brew's used slot is the first obtain destination");
    assert_eq!(potions.iter().filter(|potion| **potion == Some(PotionId::EntropicBrew)).count(), 0);
    assert_eq!(combat.state.resolved_effects, 4, "one Brew action plus three separately dispatched obtains");
}

#[test]
fn m15_accepted_bad_obtain_retains_current_source_and_successful_prefix_as_first_fault() {
    let mut combat = brew_combat(3, false);
    combat.state.item_context = Some(CombatItemContext::new(vec![], vec![None, None, None]).with_m15_profile());
    let source = Source::Potion { slot: 0, id: PotionId::EntropicBrew };
    let accepted = Effect { source: source.clone(), target: None,
        kind: EffectKind::ObtainPotion { id: PotionId::StrengthPotion } };
    let malformed = Effect { source, target: None,
        kind: EffectKind::ObtainPotion { id: PotionId::Unknown("not_registered") } };

    assert_eq!(super::super::engine::validate_effect(&malformed, &combat.state), Ok(()));

    let result = combat.enqueue_native_effects(vec![accepted, malformed.clone()], None);
    let Err(crate::run::RunError::Faulted(crate::run::RunFault {
        kind: crate::run::RunFaultKind::Combat(fault), ..
    })) = result else { panic!("accepted malformed obtain must fault in the shared resolver") };
    assert_eq!(fault.record.kind, FaultKind::Invariant);
    assert_eq!(fault.record.message, "generated potion obtain input invalid");
    assert_eq!(fault.current, Some(malformed));
    assert_eq!(fault.pending.len(), 0);
    assert_eq!(combat.state.item_context.as_ref().unwrap().potions()[0], Some(PotionId::StrengthPotion));

    let before = combat.snapshot();
    assert!(matches!(combat.step_full(&CombatAction::EndTurn), Err(crate::run::RunError::Faulted(_))));
    assert_eq!(combat.snapshot(), before, "repeated inspection/step preserves the first fault and accepted prefix");
}

#[test]
fn m15_sozu_does_not_silence_an_accepted_malformed_obtain_fault() {
    let mut combat = brew_combat(3, true);
    let malformed = Effect {
        source: Source::Potion { slot: 0, id: PotionId::EntropicBrew },
        target: None,
        kind: EffectKind::ObtainPotion { id: PotionId::Unknown("bad_under_sozu") },
    };

    assert_eq!(super::super::engine::validate_effect(&malformed, &combat.state), Ok(()));
    let result = combat.enqueue_native_effects(vec![malformed.clone()], None);
    let Err(crate::run::RunError::Faulted(crate::run::RunFault {
        kind: crate::run::RunFaultKind::Combat(fault), ..
    })) = result else { panic!("accepted malformed obtain must fault even when Sozu blocks valid obtains") };
    assert_eq!(fault.record.kind, FaultKind::Invariant);
    assert_eq!(fault.record.message, "generated potion obtain input invalid");
    assert_eq!(fault.current, Some(malformed));
    assert_eq!(fault.pending.len(), 0);
    assert_eq!(combat.state.item_context.as_ref().unwrap().potions()[0], Some(PotionId::EntropicBrew));
}

#[test]
fn m15_generated_obtain_accepts_an_owned_card_source_diagnostic() {
    let mut combat = brew_combat(3, false);
    combat.state.item_context = Some(CombatItemContext::new(vec![], vec![None, None, None]).with_m15_profile());
    let parent = combat.state.piles.hand.pop().expect("owned card parent diagnostic");
    let source = Source::Card(parent.handle());
    combat.state.in_play = Some(parent);
    let effect = Effect { source, target: None,
        kind: EffectKind::ObtainPotion { id: PotionId::StrengthPotion } };

    assert_eq!(super::super::engine::validate_effect(&effect, &combat.state), Ok(()));
    combat.state.pending.push_back(effect);
    let transition = super::super::engine::resolve(&mut combat.state);

    assert_eq!(transition.status, TransitionStatus::Decision);
    assert_eq!(combat.state.item_context.as_ref().unwrap().potions()[0], Some(PotionId::StrengthPotion));
    assert!(combat.state.fault.is_none());
}

#[test]
fn m15_combat_brew_triggers_ornithopter_once_not_for_each_generated_obtain() {
    let mut combat = brew_combat(3,false);
    combat.state.player.max_hp = 60;
    combat.state.player.hp = 50;
    combat.state.item_context = Some(CombatItemContext::new(
        vec![RelicId::ToyOrnithopter], vec![Some(PotionId::EntropicBrew), None, None],
    ).with_m15_profile());

    combat.step_full(&CombatAction::UsePotion{slot:0,target:None}).unwrap();

    assert_eq!(combat.state.player.hp, 55, "Brew's manual use callback runs once; generated obtain actions have no use hook");
    assert_eq!(combat.state.resolved_effects, 5, "Brew, three obtains, and one Ornithopter heal");
}

#[test]
fn m15_potion_belt_uses_the_actual_fifth_slot_through_the_shared_potion_action(){
    let mut combat=brew_combat(5,true);
    combat.state.item_context=Some(CombatItemContext::new(vec![RelicId::PotionBelt,RelicId::Sozu],vec![None,None,None,None,Some(PotionId::EntropicBrew)]).with_m15_profile());
    let before=combat.state.card_rng.clone();
    combat.step_full(&CombatAction::UsePotion{slot:4,target:None}).unwrap();
    assert_ne!(combat.state.card_rng,before);assert!(combat.item_context().unwrap().potions().iter().all(Option::is_none));
}
