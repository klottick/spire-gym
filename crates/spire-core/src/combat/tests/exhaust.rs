use super::super::*;
use crate::content::{CardId,Character,EncounterId,RelicId};
use crate::combat::model::{FullPowerEntry,OwnedCard,PlayContext};

fn feel_no_pain()->FullPowerEntry{
    FullPowerEntry{id:FullPowerId("feel_no_pain"),state:PowerState{amount:3,skip_first:false,monster_source_grace:false}}
}

fn ironclad_acquired_ashes_context()->crate::items::CombatItemContext{
    let mut run=crate::run::Run::new(crate::run::RunConfig{character:Character::Ironclad,ascension:0},
        crate::run::CreationProfile::experienced(),crate::run::RunRng::new([41;32],crate::combat::rng::Partition::Test,0)).unwrap();
    crate::items::acquire_relic(run.state_mut_for_test(),RelicId::CharonsAshes).unwrap();
    crate::items::CombatItemContext::take_from_run(run.state_mut_for_test())
}

fn selected_character(character:Character,cards:Vec<LiveCard>,draw:usize)->Combat{
    let mut entry=super::m8_shapes::entry(EncounterId::OrbWalker);
    entry.setup.character=character;
    entry.setup.player_powers.push(feel_no_pain());
    entry.cards=cards;
    entry.opening_draw=draw;
    Combat::enter(entry).expect("valid authored combat fixture accepts each supported character and the supplied cards")
}

#[test]
fn character_neutral_ethereal_defaults_cover_dazed_void_and_equivalent_copies(){
    for id in [CardId::Dazed,CardId::Void]{
        let permanent=card_state::permanent(id,CardInstanceId(800),0);
        let generated=card_state::generated(id,CardInstanceId(801),0).expect("status constructor");
        assert!(permanent.local.ethereal,"permanent {id:?} must own its intrinsic Ethereal property");
        assert!(generated.local.ethereal,"generated {id:?} must own its intrinsic Ethereal property");
        assert!(permanent.stat_equivalent_copy(CardInstanceId(802)).local.ethereal);
        assert!(permanent.copy_as(CardInstanceId(803)).local.ethereal,"adapter clones retain live flags");
    }
}

#[test]
fn all_four_character_setups_use_the_shared_ethereal_end_turn_exhaust_path_once(){
    for (index,character) in [Character::Ironclad,Character::Silent,Character::Defect,Character::Watcher].into_iter().enumerate(){
        let id=CardInstanceId(810+index as u64);
        let mut combat=selected_character(character,vec![permanent_live_card(CardId::Dazed,id,0)],1);
        let source=card_state::handle(id);
        assert!(engine::validate_runtime(&combat.state).is_ok(),"valid {character:?} full setup");
        combat.step_full(&CombatAction::EndTurn).expect("ordinary end turn");
        assert!(combat.state.piles.exhaust.iter().any(|owned|owned.handle()==source),"Dazed exhausts for {character:?}");
        assert_eq!(combat.state.hook_events.iter().filter(|event|event.hook==Hook::CardExhausted&&event.source==Source::Card(source.clone())).count(),1,
            "one shared notification for {character:?}");
        assert!(engine::validate_runtime(&combat.state).is_ok(),"post-end-turn ownership for {character:?}");
    }
}

#[test]
fn non_ironclad_exhaust_sources_reach_character_neutral_power_listener(){
    for (index,(character,id)) in [(Character::Silent,CardId::Shiv),(Character::Defect,CardId::Slimed),(Character::Watcher,CardId::Miracle)].into_iter().enumerate(){
        let live=card_state::generated(id,CardInstanceId(850+index as u64),0).expect("supported generated source");
        let mut combat=selected_character(character,vec![permanent_live_card(CardId::Defend,CardInstanceId(849),0)],0);
        let source=card_state::handle(live.instance);
        combat.state.card_ids_issued.insert(source.clone());
        combat.state.piles.hand.push(OwnedCard::Full(live));
        engine::validate_runtime(&combat.state).expect("issued cross-character source is valid runtime custody");
        let at=combat.state.piles.hand.iter().position(|owned|owned.handle()==source).unwrap();
        let owned=combat.state.piles.hand.remove(at);
        piles::exhaust_owned_card(&mut combat.state,owned).unwrap();
        engine::resolve(&mut combat.state);
        assert!(combat.state.piles.exhaust.iter().any(|owned|owned.handle()==source),"{character:?} {id:?} exhaust reaches shared destination");
        assert_eq!(combat.state.hook_events.iter().filter(|event|event.hook==Hook::CardExhausted&&event.source==Source::Card(source.clone())).count(),1);
        assert!(combat.state.player.block>=3,"FNP listens to an actually exhausted non-Ironclad source ({character:?}/{id:?})");
        engine::validate_runtime(&combat.state).expect("cross-color callback retains valid issued-card ownership");
    }
}

#[test]
fn shared_exhaust_callback_dispatches_supplied_ironclad_ashes_context_for_every_hero(){
    for (index,character) in [Character::Ironclad,Character::Silent,Character::Defect,Character::Watcher].into_iter().enumerate(){
        let id=CardInstanceId(880+index as u64);
        let mut combat=selected_character(character,vec![permanent_live_card(CardId::Defend,id,0)],1);
        // This is an actual Ironclad-acquired context supplied to another
        // character solely as a dispatcher diagnostic, not normal acquisition.
        combat.state.item_context=Some(ironclad_acquired_ashes_context());
        engine::validate_runtime(&combat.state).expect("supplied fixture has real, unique card and actor owners");
        let before=combat.state.enemies[0].hp;
        let owned=combat.state.piles.hand.remove(0);
        piles::exhaust_owned_card(&mut combat.state,owned).unwrap();
        engine::resolve(&mut combat.state);
        assert_eq!(combat.state.enemies[0].hp,before-3,"shared callback dispatches Charon's Ashes for supplied {character:?} setup");
        let source=combat.state.hook_events.iter().find(|event|event.hook==Hook::CardExhausted).unwrap().source.clone();
        assert!(matches!(source,Source::Card(_)));
        assert!(engine::validate_runtime(&combat.state).is_ok());
    }
}

#[test]
fn captured_use_only_exhaust_override_is_not_limited_to_a_corruption_skill(){
    let mut combat=super::full_runtime::entered_combat(3,3,|_|{});
    let index=combat.state.piles.hand.iter().position(|owned|owned.full().is_some_and(|card|card.id==CardId::Strike)).unwrap();
    let owned=combat.state.piles.hand.remove(index);
    let handle=owned.handle();
    combat.state.in_play=Some(owned);
    combat.state.play_context=Some(PlayContext{card:handle.clone(),target:Some(Actor::Enemy(combat.state.enemies[0].instance_id.clone())),
        energy_on_use:3,paid:2,disposition:super::super::model::FinishDisposition::Exhaust});
    combat.state.phase=Phase::PlayerAction;
    combat.state.pending.push_back(Effect{source:Source::Card(handle),target:None,kind:EffectKind::FinishCard});
    assert!(engine::validate_runtime(&combat.state).is_ok(),"a generic use-only exhaust override is valid for an Attack without Corruption");
    engine::resolve(&mut combat.state);
    assert!(combat.state.piles.exhaust.iter().any(|owned|owned.handle()==card_state::handle(CardInstanceId(1))));
    assert!(!combat.state.piles.exhaust.iter().find(|owned|owned.handle()==card_state::handle(CardInstanceId(1)))
        .unwrap().full().unwrap().local.exhaust_on_finish,"use-only override leaves intrinsic disposition untouched");
    assert!(engine::validate_runtime(&combat.state).is_ok());
}
