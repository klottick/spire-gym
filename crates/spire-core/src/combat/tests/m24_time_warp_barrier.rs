//! Global Time Warp barrier law (M24 shared-owner fix): every ordinary action queued
//! through `enemies::act2::bottom`, including work created after the twelfth card's
//! after-use checkpoint, resolves before Time Eater's early end (static findings
//! Task8: the early-end sequence preserves ordinary actions). Character-independent.
use super::super::*;
use crate::{content::{CardId,Character,EncounterId},items::CombatItemContext};
fn enter(character:Character,cards:&[CardId])->Combat{
    let mut e=super::m8_shapes::entry(EncounterId::TimeEater);e.setup.character=character;
    e.cards=cards.iter().enumerate().map(|(i,id)|permanent_live_card(*id,CardInstanceId(i as u64+1),0)).collect();e.opening_draw=cards.len().min(10);e.setup.player.draw_per_turn=cards.len().clamp(1,10) as i32;
    let mut items=Some(CombatItemContext::new(vec![],vec![None;3]));Combat::enter_with_item_context(e,&mut items).unwrap()
}
fn time_warp(c:&Combat)->i32{c.state.enemies[0].full_powers.iter().find(|p|p.id.0=="time_warp").unwrap().state.amount}
fn strength(c:&Combat)->i32{c.state.enemies[0].full_powers.iter().find(|p|p.id.0=="strength").map_or(0,|p|p.state.amount)}

#[test]
fn m24_time_warp_barrier_defect_evoke_after_twelfth_card(){
    // Twelfth card Zap channels Lightning into full slots: the oldest Lightning is
    // evoked, and its damage child is queued through the shared bottom helper while
    // the card payload resolves, i.e. after Time Warp's barrier request.
    let mut c=enter(Character::Defect,&[CardId::Zap]);c.state.enemies[0].full_powers.iter_mut().find(|p|p.id.0=="time_warp").unwrap().state.amount=11;
    c.state.character.orb_slots=vec![Some(OrbKind::Lightning.into());3];let hp=c.state.enemies[0].hp;
    c.step_full(&CombatAction::PlayCard{instance:1,target:None}).expect("evoke work resolves before the early end, no barrier fault");
    assert_eq!(c.state.turn,2,"exactly one forced end");assert_eq!(time_warp(&c),0);assert_eq!(strength(&c),2);assert!(c.state.pending_time_warp.is_none());
    assert_eq!(c.state.hook_events.iter().filter(|e|e.hook==Hook::TurnEnded).count(),1);
    // Zero-Focus Lightning: evoke 8 before the end, then three end-of-turn passives of 3.
    assert_eq!(hp-c.state.enemies[0].hp,8+3*3,"evoke damage landed before the early end, passives once");
    assert!(engine::validate_runtime(&c.state).is_ok());
}

#[test]
fn m24_time_warp_barrier_defect_storm_after_twelfth_power(){
    // Twelfth card is a Power while Storm is installed and every slot holds Lightning:
    // Storm's after-use channel and the resulting evoke both enter through `bottom`.
    let mut c=enter(Character::Defect,&[CardId::Defragment]);c.state.enemies[0].full_powers.iter_mut().find(|p|p.id.0=="time_warp").unwrap().state.amount=11;
    c.state.player_full_powers.push(FullPowerEntry{id:FullPowerId("storm"),state:PowerState{amount:1,skip_first:false,monster_source_grace:false}});
    c.state.character.orb_slots=vec![Some(OrbKind::Lightning.into());3];let hp=c.state.enemies[0].hp;
    c.step_full(&CombatAction::PlayCard{instance:1,target:None}).expect("Storm channel/evoke resolve before the early end, no barrier fault");
    assert_eq!(c.state.turn,2);assert_eq!(time_warp(&c),0);assert_eq!(strength(&c),2);assert!(c.state.pending_time_warp.is_none());
    assert_eq!(c.state.hook_events.iter().filter(|e|e.hook==Hook::TurnEnded).count(),1);
    // Defragment +1 Focus: evoke 8+1, then three end-of-turn Lightning passives of 3+1.
    assert_eq!(hp-c.state.enemies[0].hp,9+3*4,"Storm evoke landed before the early end, passives once");
    assert!(engine::validate_runtime(&c.state).is_ok());
}

#[test]
fn m24_time_warp_barrier_ironclad_dark_embrace_draws_void_after_twelfth_card(){
    // Twelfth card Impervious exhausts at disposition, after the Time Warp request.
    // Dark Embrace's draw then pulls Void, whose actual-draw callback queues
    // LoseEnergy(1) before Decision: ordinary work that must precede the early end.
    let mut e=super::m8_shapes::entry(EncounterId::TimeEater);e.setup.character=Character::Ironclad;
    e.cards=vec![permanent_live_card(CardId::Impervious,CardInstanceId(1),0),permanent_live_card(CardId::Void,CardInstanceId(2),0)];
    e.opening_draw=2;e.setup.player.draw_per_turn=1;
    let mut items=Some(CombatItemContext::new(vec![],vec![None;3]));let mut c=Combat::enter_with_item_context(e,&mut items).unwrap();
    // Constructed placement: return the opening Void to the draw pile so the only
    // later draw is Dark Embrace's (owned identity preserved, no reissue).
    let at=c.state.piles.hand.iter().position(|card|card.full().is_some_and(|f|f.id==CardId::Void)).unwrap();let void=c.state.piles.hand.remove(at);c.state.piles.draw.push(void);
    let energy_before_void=c.state.player.energy;assert!(energy_before_void>=2);
    c.state.enemies[0].full_powers.iter_mut().find(|p|p.id.0=="time_warp").unwrap().state.amount=11;
    c.state.player_full_powers.push(FullPowerEntry{id:FullPowerId("dark_embrace"),state:PowerState{amount:1,skip_first:false,monster_source_grace:false}});
    c.step_full(&CombatAction::PlayCard{instance:1,target:None}).expect("Void's energy loss resolves before the early end, no barrier fault");
    assert_eq!(c.state.turn,2,"exactly one forced end");assert_eq!(time_warp(&c),0);assert_eq!(strength(&c),2);assert!(c.state.pending_time_warp.is_none());
    assert_eq!(c.state.hook_events.iter().filter(|e|e.hook==Hook::TurnEnded).count(),1);
    assert!(engine::validate_runtime(&c.state).is_ok());
}
