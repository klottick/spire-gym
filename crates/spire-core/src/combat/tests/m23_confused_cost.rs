//! M23 correction (root ruling M23-D1 v2): source ConfusionPower.onCardDraw sets the
//! drawn instance's combat cost for every character. Defect cards carry a preset
//! combat-cost owner, so the Confused price must survive turn expiry. Real Snecko
//! Glare and draw; the turn-expiry call is the shared owner, labelled owner-level.
use super::super::*;
use crate::{content::{CardId,Character,EncounterId},items::CombatItemContext};

fn enter_defect(cards:&[(CardId,u32)])->Combat{
    let mut entry=super::m6_ordinary::entry(EncounterId::Snecko);
    entry.setup.character=Character::Defect;entry.setup.player.energy=3;entry.setup.player.max_energy=3;
    entry.setup.player.draw_per_turn=cards.len() as i32;
    entry.cards=cards.iter().enumerate().map(|(i,(id,up))|permanent_live_card(*id,CardInstanceId(i as u64+1),*up)).collect();
    entry.opening_draw=0;
    let mut inventory=Some(CombatItemContext::new(vec![],vec![None;3]));
    Combat::enter_with_item_context(entry,&mut inventory).unwrap()
}
fn hand(c:&Combat,i:u64)->&LiveCard{c.state.piles.hand.iter().filter_map(model::OwnedCard::full).find(|card|card.instance==CardInstanceId(i)).unwrap()}

#[test]
fn m23_defect_confused_price_survives_turn_expiry_and_resets_next_combat(){
    let deck=[(CardId::Fusion,0),(CardId::Recursion,1),(CardId::Fusion,1)];
    let mut combat=enter_defect(&deck);
    for i in 1..=3{assert_eq!(hand_or_draw_base(&combat,i),Some(match i{1=>2,2=>0,_=>1}));}
    combat.step_full(&CombatAction::EndTurn).unwrap();
    assert!(combat.state.player_full_powers.iter().any(|p|p.id.0=="confused"&&p.state.amount>0),"actual Snecko Glare");
    let mut changed=0;
    for i in 1..=3{
        let card=hand(&combat,i).clone();let base=cards::base_cost(&combat.state,&card).unwrap();let r=card.local.randomized_cost.expect("actual Confused draw");
        assert_eq!(card.local.mutable_combat_cost,Some(r),"Confused sets the combat cost");
        if r!=base{changed+=1;}
    }
    assert!(changed>0,"at least one drawn price differs from its base at this fixed seed, so the check is not vacuous");
    // Shared turn-expiry owner: the Confused price is the card's combat price afterwards.
    piles::expire_turn_scoped_costs(&mut combat.state);
    for i in 1..=3{
        let card=hand(&combat,i);let base=cards::base_cost(&combat.state,card).unwrap();let r=card.local.randomized_cost.unwrap();
        assert_eq!(card_state::combat_price(card,base).unwrap(),r);assert_eq!(card.local.turn_cost,Some(r));
        let visible=combat.public_full().unwrap().hand.into_iter().find(|v|v.instance==CardInstanceId(i)).unwrap();assert_eq!(visible.effective_cost,Some(r));
    }
    // A new combat constructs fresh instances from the permanent cards: base prices again.
    let next=enter_defect(&deck);
    for i in 1..=3{assert_eq!(hand_or_draw_base(&next,i),Some(match i{1=>2,2=>0,_=>1}));}
}
fn hand_or_draw_base(c:&Combat,i:u64)->Option<i32>{
    c.state.piles.hand.iter().chain(&c.state.piles.draw).filter_map(model::OwnedCard::full).find(|card|card.instance==CardInstanceId(i)).and_then(|card|card.local.mutable_combat_cost)
}
