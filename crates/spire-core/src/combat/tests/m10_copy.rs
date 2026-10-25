use super::super::*;
use super::super::model::OwnedCard;
use crate::{content::{CardId,CardRarity,CardType},combat::{card_state,tests::full_runtime}};

fn dual_wield_combat(upgraded:bool)->Combat{
    let mut combat=full_runtime::entered_combat(3,3,|card|{
        *card=card_state::permanent(CardId::DualWield,card.instance,u32::from(upgraded));
    });
    let mut eligible_assigned=false;
    for owned in &mut combat.state.piles.hand{
        let live=owned.full().expect("full runtime hand card");
        if live.id==CardId::DualWield{continue}
        let replacement=card_state::permanent(if !eligible_assigned{eligible_assigned=true;CardId::Strike}else{CardId::Defend},live.instance,0);
        *owned=OwnedCard::Full(replacement);
    }
    assert!(eligible_assigned,"fixture has an Attack distinct from Dual Wield");
    assert!(engine::validate_runtime(&combat.state).is_ok());
    combat
}

#[test]
fn m10_dual_wield_sole_attack_copies_once_and_preserves_parent_disposition(){
    let mut combat=dual_wield_combat(false);
    let definition=crate::content::Catalog::standard().card(CardId::DualWield).unwrap();
    assert_eq!((definition.rarity,definition.card_type),(CardRarity::Uncommon,CardType::Skill));
    assert_eq!(combat.public_full().unwrap().hand.iter().find(|card|card.id==CardId::DualWield).unwrap().effective_cost,Some(1));
    let parent=combat.state.piles.hand.iter().find_map(|owned|owned.full().filter(|card|card.id==CardId::DualWield).map(|card|card.instance.0)).unwrap();
    combat.step_full(&CombatAction::PlayCard{instance:parent,target:None})
        .expect("Dual Wield must resolve instead of falling through to the unsupported-card path");
    assert!(engine::validate_runtime(&combat.state).is_ok());
    assert_eq!(combat.state.piles.hand.iter().filter(|card|card.full().is_some_and(|live|live.id==CardId::Strike)).count(),2);
    assert!(combat.state.piles.discard.iter().any(|card|card.handle().as_str()==&parent.to_string()));
    assert_eq!(combat.state.inactive_cards.len(),0,"sole eligible source remains active");
}

#[test]
fn m10_dual_wield_multi_choice_retires_source_without_disposition_and_keeps_census(){
    let mut combat=dual_wield_combat(false);
    let eligible= combat.state.piles.hand.iter_mut().filter_map(OwnedCard::full_mut)
        .find(|card|card.id==CardId::Defend).unwrap();
    let id=eligible.instance;
    *eligible=card_state::permanent(CardId::Inflame,id,0);
    let parent=combat.state.piles.hand.iter().find_map(|owned|owned.full().filter(|card|card.id==CardId::DualWield).map(|card|card.instance.0)).unwrap();
    combat.step_full(&CombatAction::PlayCard{instance:parent,target:None}).unwrap();
    assert_eq!(combat.state.phase,Phase::Selection);
    let selected=combat.state.pending_selection.as_ref().unwrap().candidates.iter().find(|handle|{
        combat.state.piles.hand.iter().any(|card|card.handle()==**handle&&card.full().is_some_and(|c|c.id==CardId::Strike))
    }).unwrap().as_str().parse::<u64>().unwrap();
    let waiting=combat.state.clone();
    assert_eq!(combat.step_full(&CombatAction::SelectCard{instance:999}),Err(crate::run::RunError::InvalidAction));
    assert_eq!(combat.state,waiting,"stale/invalid source selection is atomic");
    combat.step_full(&CombatAction::SelectCard{instance:selected}).unwrap();
    assert!(combat.state.piles.hand.iter().all(|card|card.handle().as_str()!=selected.to_string()));
    assert!(combat.state.piles.discard.iter().all(|card|card.handle().as_str()!=selected.to_string()));
    assert!(combat.state.piles.exhaust.iter().all(|card|card.handle().as_str()!=selected.to_string()));
    assert_eq!(combat.state.inactive_cards.len(),1);
    assert_eq!(combat.state.inactive_cards[0].full().unwrap().instance,CardInstanceId(selected));
    let strike_copies=combat.state.piles.hand.iter().filter_map(OwnedCard::full).filter(|card|card.id==CardId::Strike).collect::<Vec<_>>();
    assert_eq!(strike_copies.len(),2);
    assert!(strike_copies.iter().all(|card|card.copy_provenance==card_state::CopyProvenance::StatEquivalent));
    assert!(strike_copies[0].copy_family!=strike_copies[1].copy_family);
    assert!(engine::validate_runtime(&combat.state).is_ok());
}

#[test]
fn m10_stat_copy_preserves_mutable_searing_stats_but_not_source_local_disposition_mutations(){
    let mut source=card_state::permanent(CardId::SearingBlow,CardInstanceId(44),3);
    source.local.base_damage=Some(34);
    source.local.randomized_cost=Some(2);source.local.temporary_cost=-1;source.local.free_play=true;
    source.local.exhaust_on_finish=true;source.local.ethereal=true;
    let copy=source.stat_equivalent_copy(CardInstanceId(45));
    assert_eq!(copy.local.base_damage,Some(34));
    assert_eq!(copy.upgrades,3);
    assert_eq!(copy.local.randomized_cost,Some(2));
    assert_eq!(copy.local.temporary_cost,-1);
    assert!(copy.local.free_play);
    assert!(!copy.local.exhaust_on_finish);
    assert!(!copy.local.ethereal);
    assert_eq!(copy.copy_provenance,card_state::CopyProvenance::StatEquivalent);
    assert_ne!(copy.copy_family,source.copy_family);
    let family=source.same_instance_family_copy(CardInstanceId(46));
    assert_eq!(family.copy_family,source.copy_family);
    assert_eq!(family.copy_provenance,card_state::CopyProvenance::SameInstanceFamily);
}

#[test]
fn m10_upgraded_dual_wield_adds_two_children_without_upgrading_them(){
    let mut combat=dual_wield_combat(true);
    let parent=combat.state.piles.hand.iter().find_map(|owned|owned.full().filter(|card|card.id==CardId::DualWield).map(|card|card.instance.0)).unwrap();
    combat.step_full(&CombatAction::PlayCard{instance:parent,target:None}).unwrap();
    let strikes=combat.state.piles.hand.iter().filter_map(OwnedCard::full).filter(|card|card.id==CardId::Strike).collect::<Vec<_>>();
    assert_eq!(strikes.len(),3);
    assert!(strikes.iter().all(|card|card.upgrades==0));
    assert_eq!(combat.state.player.energy,2);
    assert!(engine::validate_runtime(&combat.state).is_ok());
}

#[test]
fn m10_dual_wield_no_eligible_is_noop_and_invalid_target_is_atomic(){
    let mut combat=dual_wield_combat(false);
    for owned in &mut combat.state.piles.hand{
        if owned.full().is_some_and(|card|card.id==CardId::Strike){
            let old=owned.full().unwrap().instance;*owned=OwnedCard::Full(card_state::permanent(CardId::Defend,old,0));
        }
    }
    let parent=combat.state.piles.hand.iter().find_map(|owned|owned.full().filter(|card|card.id==CardId::DualWield).map(|card|card.instance.0)).unwrap();
    let before=combat.state.clone();
    assert_eq!(combat.step_full(&CombatAction::PlayCard{instance:parent,target:Some(TargetId(1))}),Err(crate::run::RunError::InvalidAction));
    assert_eq!(combat.state,before,"invalid target does not pay, move, or consume RNG");
    combat.step_full(&CombatAction::PlayCard{instance:parent,target:None}).unwrap();
    assert!(combat.state.pending_selection.is_none());
    assert_eq!(combat.state.inactive_cards.len(),0);
    assert_eq!(combat.state.piles.hand.iter().filter_map(OwnedCard::full).filter(|card|card.id==CardId::Strike).count(),0);
    assert!(engine::validate_runtime(&combat.state).is_ok());
}

#[test]
fn m10_dual_wield_children_overflow_to_discard_at_real_hand_cap(){
    let mut combat=dual_wield_combat(true);
    for n in 100..107{
        let instance=CardInstanceId(n);let handle=card_state::handle(instance);
        combat.state.card_ids_issued.insert(handle);
        combat.state.piles.hand.push(OwnedCard::Full(card_state::permanent(CardId::Defend,instance,0)));
    }
    assert_eq!(combat.state.piles.hand.len(),10);
    assert!(engine::validate_runtime(&combat.state).is_ok());
    let parent=combat.state.piles.hand.iter().find_map(|owned|owned.full().filter(|card|card.id==CardId::DualWield).map(|card|card.instance.0)).unwrap();
    combat.step_full(&CombatAction::PlayCard{instance:parent,target:None}).unwrap();
    assert_eq!(combat.state.piles.hand.len(),10);
    assert_eq!(combat.state.piles.discard.iter().filter(|card|card.full().is_some_and(|live|live.copy_provenance==card_state::CopyProvenance::StatEquivalent)).count(),1);
    assert!(engine::validate_runtime(&combat.state).is_ok());
}

#[test]
fn m10_dual_wield_partial_allocation_fault_retains_first_child_and_never_retries(){
    let mut combat=dual_wield_combat(true);
    let missing=998-combat.state.card_ids_issued.len();
    for offset in 0..missing{
        let instance=CardInstanceId(1000+offset as u64);let handle=card_state::handle(instance);
        combat.state.card_ids_issued.insert(handle);
        combat.state.piles.draw.push(OwnedCard::Full(card_state::permanent(CardId::Defend,instance,0)));
    }
    assert_eq!(combat.state.card_ids_issued.len(),998);
    assert!(engine::validate_runtime(&combat.state).is_ok());
    let parent=combat.state.piles.hand.iter().find_map(|owned|owned.full().filter(|card|card.id==CardId::DualWield).map(|card|card.instance.0)).unwrap();
    assert!(combat.step_full(&CombatAction::PlayCard{instance:parent,target:None}).is_err());
    assert!(combat.fault_record().is_some());
    assert!(combat.state.in_play.as_ref().is_some_and(|card|card.full().is_some_and(|live|live.instance.0==parent)));
    assert_eq!(combat.state.card_ids_issued.len(),999);
    assert_eq!(combat.state.piles.hand.iter().chain(&combat.state.piles.discard).filter_map(OwnedCard::full)
        .filter(|card|card.copy_provenance==card_state::CopyProvenance::StatEquivalent).count(),1);
    let retained=combat.state.clone();
    assert!(combat.step_full(&CombatAction::PlayCard{instance:parent,target:None}).is_err());
    assert_eq!(combat.state,retained,"faulted copy allocation is not retried or rolled back");
}

#[test]
fn m10_upgraded_multi_dual_wield_copies_selected_power_without_upgrading_or_playing_source(){
    let mut combat=dual_wield_combat(true);
    let power=combat.state.piles.hand.iter_mut().filter_map(OwnedCard::full_mut)
        .find(|card|card.id==CardId::Defend).unwrap();
    let source_id=power.instance;
    *power=card_state::permanent(CardId::Inflame,source_id,0);
    let source=combat.state.piles.hand.iter().filter_map(OwnedCard::full)
        .find(|card|card.instance==source_id).unwrap().clone();
    let parent=combat.state.piles.hand.iter().find_map(|owned|owned.full()
        .filter(|card|card.id==CardId::DualWield).map(|card|card.instance.0)).unwrap();

    combat.step_full(&CombatAction::PlayCard{instance:parent,target:None}).unwrap();
    let selection=combat.state.pending_selection.as_ref().unwrap();
    assert_eq!(selection.candidates.len(),2,"exactly the Strike and Power are eligible");
    assert!(selection.candidates.contains(&card_state::handle(source_id)));
    combat.step_full(&CombatAction::SelectCard{instance:source_id.0}).unwrap();

    assert_eq!(combat.state.inactive_cards.len(),1);
    assert_eq!(combat.state.inactive_cards[0].full().unwrap(),&source);
    assert_eq!(combat.state.inactive_cards[0].full().unwrap().origin,crate::combat::CombatCardRef::Permanent(source_id));
    assert!(combat.state.card_ids_issued.contains(&card_state::handle(source_id)));
    assert!(!combat.state.piles.all(combat.state.in_play.as_ref()).any(|card|card.handle()==card_state::handle(source_id)));
    assert!(combat.state.played_powers.iter().all(|card|card.handle()!=card_state::handle(source_id)));
    assert!(combat.state.hook_events.iter().all(|event|event.hook!=super::super::hooks::Hook::CardExhausted
        ||event.source!=Source::Card(card_state::handle(source_id))),"Dual Wield replacement to inactive custody is not an exhaust");
    assert!(combat.state.piles.discard.iter().chain(&combat.state.piles.exhaust).all(|card|card.handle()!=card_state::handle(source_id)));
    let copies=combat.state.piles.hand.iter().filter_map(OwnedCard::full)
        .filter(|card|card.id==CardId::Inflame).collect::<Vec<_>>();
    assert_eq!(copies.len(),3);
    assert!(copies.iter().all(|card|card.upgrades==0&&card.copy_provenance==card_state::CopyProvenance::StatEquivalent));
    assert_eq!(copies.iter().map(|card|card.copy_family).collect::<std::collections::BTreeSet<_>>().len(),3);
    assert_eq!(combat.state.inactive_cards[0].full().unwrap(),&source,"copy creation does not mutate the selected original");
    assert!(engine::validate_runtime(&combat.state).is_ok());

    let child=copies[0].instance.0;
    combat.step_full(&CombatAction::PlayCard{instance:child,target:None}).unwrap();
    assert_eq!(combat.state.player_full_powers.iter().find(|power|power.id.0=="strength").unwrap().state.amount,2);
    assert!(combat.state.played_powers.iter().any(|card|card.full().is_some_and(|live|
        live.instance.0==child&&live.id==CardId::Inflame&&live.origin==crate::combat::CombatCardRef::CombatLocal(child))));
    assert_eq!(combat.state.inactive_cards[0].full().unwrap(),&source);
    assert!(engine::validate_runtime(&combat.state).is_ok());
}

#[test]
fn m10_multi_dual_wield_allocation_fault_retains_replaced_source_and_exact_selection_custody(){
    let mut combat=dual_wield_combat(true);
    let power=combat.state.piles.hand.iter_mut().filter_map(OwnedCard::full_mut)
        .find(|card|card.id==CardId::Defend).unwrap();
    let source_id=power.instance;
    *power=card_state::permanent(CardId::Inflame,source_id,0);
    let parent=combat.state.piles.hand.iter().find_map(|owned|owned.full()
        .filter(|card|card.id==CardId::DualWield).map(|card|card.instance.0)).unwrap();
    let missing=998-combat.state.card_ids_issued.len();
    for offset in 0..missing{
        let instance=CardInstanceId(1000+offset as u64);let handle=card_state::handle(instance);
        combat.state.card_ids_issued.insert(handle);
        combat.state.piles.draw.push(OwnedCard::Full(card_state::permanent(CardId::Defend,instance,0)));
    }
    assert_eq!(combat.state.card_ids_issued.len(),998);
    assert!(engine::validate_runtime(&combat.state).is_ok());

    combat.step_full(&CombatAction::PlayCard{instance:parent,target:None}).unwrap();
    assert_eq!(combat.state.pending_selection.as_ref().unwrap().candidates.len(),2);
    assert!(combat.step_full(&CombatAction::SelectCard{instance:source_id.0}).is_err());
    assert!(combat.fault_record().is_some());
    assert!(combat.state.in_play.as_ref().is_some_and(|card|card.full().is_some_and(|live|live.instance.0==parent)));
    assert!(combat.state.inactive_cards.iter().any(|card|card.full().is_some_and(|live|live.instance==source_id)));
    assert!(!combat.state.piles.all(combat.state.in_play.as_ref()).any(|card|card.handle()==card_state::handle(source_id)));
    assert_eq!(combat.state.card_ids_issued.len(),999);
    let accepted=combat.state.piles.hand.iter().filter_map(OwnedCard::full)
        .filter(|card|card.copy_provenance==card_state::CopyProvenance::StatEquivalent).collect::<Vec<_>>();
    assert_eq!(accepted.len(),1);
    assert_eq!((accepted[0].id,accepted[0].upgrades),(CardId::Inflame,0));
    assert_ne!(accepted[0].instance,source_id);
    assert_ne!(accepted[0].copy_family,combat.state.inactive_cards.iter().find_map(OwnedCard::full).unwrap().copy_family);
    let retained_selection=combat.state.fault_selection.as_ref().unwrap();
    assert_eq!(retained_selection.parent,super::super::selection::SelectionParent::Card(card_state::handle(CardInstanceId(parent))));
    assert_eq!(retained_selection.source_zone,super::super::selection::SelectionSourceZone::Hand);
    assert_eq!(retained_selection.operation,super::super::selection::SelectionOperation::CopyToHand{copies:3});
    assert_eq!(retained_selection.selected,[card_state::handle(source_id)]);

    let owners=combat.state.piles.all(combat.state.in_play.as_ref())
        .chain(combat.state.stasis.values()).chain(&combat.state.played_powers).chain(&combat.state.inactive_cards)
        .map(OwnedCard::handle).collect::<Vec<_>>();
    assert_eq!(owners.len(),combat.state.card_ids_issued.len());
    assert_eq!(owners.into_iter().collect::<std::collections::BTreeSet<_>>(),combat.state.card_ids_issued);
    assert!(engine::validate_runtime(&combat.state).is_ok(),"fault metadata and resolved-effect frontier remain valid");

    let retained=combat.state.clone();
    assert!(combat.step_full(&CombatAction::SelectCard{instance:source_id.0}).is_err());
    assert_eq!(combat.state,retained,"retry cannot duplicate children, release source, or lose owner state");
    assert!(combat.step_full(&CombatAction::PlayCard{instance:parent,target:None}).is_err());
    assert_eq!(combat.state,retained,"replaying the parent cannot restart the faulted operation");
}
