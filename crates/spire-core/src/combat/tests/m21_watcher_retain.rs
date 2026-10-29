use super::super::*;
#[test]
fn m21_watcher_retain_growth_is_on_the_exact_physical_instance() {
    let mut c=super::full_runtime::entered_combat(3,3,|card|*card=permanent_live_card(crate::content::CardId::WindmillStrike,card.instance,0));
    c.state.silent.source_character=Some(crate::content::Character::Watcher);
    c.step_full(&CombatAction::EndTurn).unwrap();
    let card=c.state.piles.hand.iter().find_map(|o|o.full().filter(|card|card.instance.0==3)).unwrap();
    assert_eq!(card.local.base_damage,Some(11));
}
#[test]
fn m21_watcher_retain_growth_upgrade_does_not_recompute_past_growth() {
    let mut c=super::full_runtime::entered_combat(3,3,|card|*card=permanent_live_card(crate::content::CardId::WindmillStrike,card.instance,0));
    c.state.silent.source_character=Some(crate::content::Character::Watcher);
    for _ in 0..2 {c.step_full(&CombatAction::EndTurn).unwrap();}
    let card=c.state.piles.hand.iter_mut().find_map(|o|o.full_mut().filter(|card|card.instance.0==3)).unwrap();
    cards::ironclad::rules::upgrade(card).unwrap();
    assert_eq!(card.local.base_damage,Some(18));
    c.step_full(&CombatAction::EndTurn).unwrap();
    assert_eq!(c.state.piles.hand.iter().find_map(|o|o.full().filter(|card|card.instance.0==3)).unwrap().local.base_damage,Some(23));
}
#[test]
fn m21_watcher_cost_positive_and_zero_turn_prices_are_distinct() {
    let mut card=permanent_live_card(crate::content::CardId::SandsOfTime,CardInstanceId(19),0);
    card.local.mutable_combat_cost=Some(4);card.local.turn_cost=Some(2);
    card_state::modify_cost_for_combat(&mut card,-1).unwrap();
    assert_eq!((card.local.mutable_combat_cost,card.local.turn_cost),(Some(1),Some(1)));
    card.local.mutable_combat_cost=Some(4);card.local.turn_cost=Some(0);
    card_state::modify_cost_for_combat(&mut card,-1).unwrap();
    assert_eq!((card.local.mutable_combat_cost,card.local.turn_cost),(Some(3),Some(0)));
}
#[test]
fn m21_watcher_retain_ethereal_survives_and_unretained_exhausts(){
    let mut c=super::full_runtime::entered_combat(3,3,|card|*card=permanent_live_card(crate::content::CardId::WindmillStrike,card.instance,0));
    c.state.silent.source_character=Some(crate::content::Character::Watcher);
    for (id,kind,retain) in [(20,crate::content::CardId::Dazed,true),(21,crate::content::CardId::Void,true),(22,crate::content::CardId::Dazed,false)]{
        let mut card=card_state::generated(kind,CardInstanceId(id),0).unwrap();card.local.retain=retain;c.state.card_ids_issued.insert(card_state::handle(card.instance));c.state.piles.hand.push(model::OwnedCard::Full(card));
    }
    c.step_full(&CombatAction::EndTurn).unwrap();
    for id in [20,21]{let card=c.state.piles.hand.iter().find_map(|o|o.full().filter(|c|c.instance.0==id)).unwrap();assert!(!card.local.retain);}
    assert!(c.state.piles.exhaust.iter().any(|c|c.handle().as_str()=="22"));
}
#[test]
fn m21_watcher_retain_establishment_scans_before_limbo_and_sands_later_reduces(){
    let mut c=super::full_runtime::entered_combat(3,3,|card|*card=m21_watcher_live_card(crate::content::CardId::SandsOfTime,card.instance,0).unwrap());
    c.state.silent.source_character=Some(crate::content::Character::Watcher);
    c.state.player_full_powers.push(FullPowerEntry{id:FullPowerId("establishment"),state:PowerState{amount:1,skip_first:false,monster_source_grace:false}});
    c.step_full(&CombatAction::EndTurn).unwrap();
    let card=c.state.piles.hand.iter().find_map(|o|o.full().filter(|c|c.instance.0==3)).unwrap();assert_eq!((card.local.mutable_combat_cost,card.local.turn_cost),(Some(2),Some(2)));
}
#[test]
fn m21_watcher_retain_upgrade_reconciles_all_profile_local_fields(){
    for id in crate::content::watcher::M21_WATCHER_CARDS.iter().chain(crate::content::watcher::M21_WATCHER_GENERATED){
        let mut card=m21_watcher_live_card(*id,CardInstanceId(19),0).unwrap();let expected=m21_watcher_live_card(*id,CardInstanceId(19),1).unwrap();
        cards::ironclad::rules::upgrade(&mut card).unwrap();
        assert_eq!((card.local.base_damage,card.local.base_block,card.local.magic_number,card.local.mutable_combat_cost,card.local.turn_cost,card.local.self_retain,card.local.innate),
            (expected.local.base_damage,expected.local.base_block,expected.local.magic_number,expected.local.mutable_combat_cost,expected.local.turn_cost,expected.local.self_retain,expected.local.innate),"{}",id.as_str());
    }
}
#[test]
fn m21_watcher_retain_fault_keeps_restored_child_and_limbo_sibling(){
    use cards::watcher::WatcherEffect;
    let mut c=super::full_runtime::entered_combat(3,3, |_|{});c.state.silent.source_character=Some(crate::content::Character::Watcher);c.state.piles.hand.clear();
    let mut cards=Vec::new();for id in [20,21]{let card=m21_watcher_live_card(crate::content::CardId::WindmillStrike,CardInstanceId(id),0).unwrap();cards.push(card_state::handle(card.instance));c.state.card_ids_issued.insert(card_state::handle(card.instance));c.state.inactive_cards.push(model::OwnedCard::Full(card));}
    c.state.resolved_effects=99998;c.state.pending.push_back(Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::Watcher(WatcherEffect::RetainRestore{cards,next:0})});
    assert_eq!(engine::resolve(&mut c.state).status,TransitionStatus::Fault);
    assert_eq!(c.state.piles.hand.iter().find_map(|c|c.full().filter(|c|c.instance.0==20)).unwrap().local.base_damage,Some(11));
    assert_eq!(c.state.inactive_cards.iter().find_map(|c|c.full().filter(|c|c.instance.0==21)).unwrap().local.base_damage,Some(7));
    assert!(matches!(c.state.current.as_ref().map(|e|&e.kind),Some(EffectKind::Watcher(WatcherEffect::RetainRestore{next:1,..}))));
}
#[test]
fn m21_watcher_retain_int32_exhaustion_fault_keeps_physical_owner(){
    let mut c=super::full_runtime::entered_combat(3,3,|card|{*card=m21_watcher_live_card(crate::content::CardId::WindmillStrike,card.instance,0).unwrap();card.local.base_damage=Some(i32::MAX);});c.state.silent.source_character=Some(crate::content::Character::Watcher);
    assert!(matches!(c.step_full(&CombatAction::EndTurn),Err(crate::run::RunError::Faulted(_))));
    let card=piles::owned_full_cards(&c.state).find(|c|c.instance.0==3).unwrap();assert_eq!(card.local.base_damage,Some(i32::MAX));assert!(c.state.inactive_cards.iter().any(|c|c.handle().as_str()=="3"));
}
