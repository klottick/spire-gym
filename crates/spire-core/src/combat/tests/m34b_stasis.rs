//! M34b review I1 (from the reviewer's probe): the public combat state Standard encoding must handle.
use super::super::*;
use crate::content::CardId;

#[test]
fn headbutt_kill_of_a_holding_orb_opens_selection_before_stasis_release() {
    let card=|id:CardId,n:u64|permanent_live_card(id,CardInstanceId(n),0);
    let mut c=super::m6_stasis::combat(vec![card(CardId::Impervious,1),card(CardId::Strike,2),card(CardId::Strike,3),card(CardId::Defend,4)]);
    super::m6_stasis::capture(&mut c,1);
    let orb=c.state.enemies[1].instance_id.clone();
    assert!(c.state.stasis.contains_key(&orb));
    // Two discard cards so Headbutt opens a real selection; Headbutt in hand; orb at 1 HP.
    while let Some(x)=c.state.piles.draw.pop(){c.state.piles.discard.push(x);}
    let hb=card(CardId::Headbutt,10);c.state.card_ids_issued.insert(card_state::handle(hb.instance));c.state.piles.hand.push(super::super::model::OwnedCard::Full(hb));
    c.state.enemies[1].hp=1;
    let view=c.public_full().unwrap();
    let target=view.targets[1].0;
    c.step_full(&CombatAction::PlayCard{instance:10,target:Some(target)}).unwrap();
    let view=c.public_full().unwrap();
    let dead=view.enemies.iter().find(|e|e.instance_id==orb).unwrap();
    // The public state Standard must encode (M34b review I1): a defeated orb (not replaced, escaped or half-dead) still
    // names a held Stasis card while Headbutt's selection is open; the card is released at the next barrier.
    assert_eq!((dead.hp, dead.replaced, dead.escaped, dead.half_dead), (0, false, false, false));
    assert!(view.selection.is_some());
    assert!(view.stasis.iter().any(|s|s.orb==orb), "stasis entry released before the selection");
}
