//! Test-only injected exhaustion at the existing Juggernaut RNG boundary.
use super::super::*;
pub(crate) fn exhaust_juggernaut_rng(combat:&mut Combat){
    combat.state.player_full_powers.push(FullPowerEntry{id:FullPowerId("juggernaut"),state:PowerState{amount:5,skip_first:false,monster_source_grace:false}});
    let stream=combat.state.card_rng.as_ref().unwrap();
    combat.state.card_rng=Some(rng::StreamState::new(stream.id().clone(),*stream.key(),1_u128<<64).unwrap());
}
pub(crate) fn corrupt_mutation_journal(combat:&mut Combat,mode:u8){
    match mode{
        0=>combat.state.defect.permanent_mutations[0].ordinal=2,
        1=>combat.state.defect.permanent_mutations[0].id=crate::content::CardId::Claw,
        2=>combat.state.defect.permanent_mutations[0].instance=CardInstanceId(999),
        3=>combat.state.defect.permanent_mutations[0].misc=123,
        4=>{let duplicate=combat.state.piles.exhaust[0].clone();combat.state.inactive_cards.push(duplicate);}
        5=>{combat.state.piles.exhaust.clear();}
        _=>{let replay=combat.state.defect.permanent_mutations[0].clone();combat.state.defect.permanent_mutations.push(replay);}
    }
}
/// Diagnostic continuation guard after two real potion dispatches, before heal.
pub(crate) fn guarded_potion_continuation(combat:&mut Combat,effects:Vec<Effect>,items:crate::items::CombatItemContext){
    combat.state.item_context=Some(items);combat.state.pending.extend(effects);combat.state.resolved_effects=99_997;
    assert_eq!(super::super::engine::resolve(&mut combat.state).status,TransitionStatus::Fault);
    assert_eq!(combat.state.character.focus,2);
}
pub(crate) fn set_defect_items(combat:&mut Combat,items:crate::items::CombatItemContext){
    combat.state.character=CharacterState::for_character(crate::content::Character::Defect);
    combat.state.silent.source_character=Some(crate::content::Character::Defect);combat.state.item_context=Some(items);
}
pub(crate) fn ready_fixture()->Combat{super::full_runtime::entered_combat(3,3, |_|{})}

/// Read-only accepted-prefix facts after the ordinary public view fails closed.
pub(crate) fn m20_item_fault_facts(combat:&Combat)->(i32,usize){
    (combat.state.enemies[0].hp,combat.state.character.orb_slots.iter().flatten().filter(|orb|orb.kind==OrbKind::Dark).count())
}

pub(crate) fn m20_item_card_rng_counter(combat:&Combat)->u128{
    combat.state.card_rng.as_ref().expect("M20 item fixture card RNG").word_index()
}

/// Literal unit-fixture input, before any potion use; never a production import.
pub(crate) fn m20_item_set_orbs(combat:&mut Combat,orbs:&[Option<OrbKind>]){
    assert!(orbs.len()<=10);
    combat.state.character.orb_slots=orbs.iter().map(|kind|kind.map(OrbState::new)).collect();
}
