//! M34s-a: the public draw multiset (`CombatPublic.draw_unknown`) is in canonical card order and
//! never follows the physical draw order (design §4.3).
use super::super::*;
use crate::{combat::card_state,content::{CardId,Character,EncounterId},items::CombatItemContext,run::RunRng};
use CardId::*;
fn fixture(character:Character,cards:&[(CardId,u32)],opening:usize)->Combat{
    let mut entry=enemies::setup(EncounterId::Cultist,1,0,&mut RunRng::new([0x43;32],rng::Partition::Test,0),enemies::SetupContext{
        player:PlayerState{hp:80,max_hp:80,block:0,energy:3,max_energy:3,draw_per_turn:5,powers:vec![]},player_powers:vec![],character,
        cards:vec![permanent_live_card(Defend,CardInstanceId(99),0)],opening_draw:opening,encounter_visit:0}).unwrap();
    entry.cards=cards.iter().enumerate().map(|(index,(id,up))|complete_live_card(*id,CardInstanceId(index as u64+1),*up).unwrap()).collect();
    entry.opening_draw=opening;
    let mut items=Some(CombatItemContext::new(vec![],vec![None,None,None]).with_complete_cards().with_complete_items());
    Combat::enter_with_public_knowledge(entry,&mut items,true).unwrap()
}
#[test]
fn draw_unknown_is_the_draw_multiset_in_canonical_order(){
    let mut c=fixture(Character::Ironclad,&[(Strike,0),(Defend,0),(Bash,0),(Strike,1),(Anger,0),(Strike,0),(Defend,1),(Clothesline,0)],3);
    let view=c.public_full().unwrap();
    assert!(view.known_draw.is_empty());
    assert_eq!(view.draw_unknown.len(),view.draw_count);
    assert!(view.draw_unknown.windows(2).all(|w|super::super::view::canonical_card_order(&w[0],&w[1])!=std::cmp::Ordering::Greater));
    let ids=|v:&CombatPublic|v.draw_unknown.iter().map(|card|card.instance).collect::<Vec<_>>();
    let before=ids(&view);
    // Test-only permutations of the physical draw order leave the public multiset unchanged.
    for rotation in 1..c.state.piles.draw.len(){
        let mut permuted=c.clone();permuted.state.piles.draw.rotate_left(rotation);
        let after=permuted.public_full().unwrap();
        assert_eq!(ids(&after),before,"rotation {rotation}");assert_eq!(after.draw_unknown,view.draw_unknown);
    }
    c.state.piles.draw.reverse();assert_eq!(c.public_full().unwrap().draw_unknown,view.draw_unknown);
}
#[test]
fn draw_unknown_excludes_known_draw_cards(){
    let mut c=fixture(Character::Ironclad,&[(Strike,0),(Defend,0),(Bash,0),(Strike,1),(Anger,0),(Strike,0)],2);
    let top=c.state.piles.draw.last().unwrap().handle();c.state.public.known_draw=vec![top];
    let view=c.public_full().unwrap();
    assert_eq!(view.known_draw.len(),1);assert_eq!(view.draw_unknown.len(),view.draw_count-1);
    assert!(view.draw_unknown.iter().all(|card|card.instance!=view.known_draw[0].card.instance));
}
/// Secret Technique / Secret Weapon publish their draw-pile candidates in the shared grid order (card id, instance),
/// like Seek and Omniscience, never in physical draw order (a0(iii); the original randomises these grids).
#[test]
fn draw_sourced_secret_candidates_hide_order(){
    for (secret,kind) in [(SecretTechnique,[Defend,ShrugItOff,Defend,Armaments]),(SecretWeapon,[Strike,Anger,Strike,Clothesline])]{
        let cards=[(secret,0),(kind[0],0),(kind[1],0),(kind[2],0),(kind[3],0),(Bash,0)];
        let orders=[[2usize,3,4,5,6],[6,5,4,3,2],[4,2,6,3,5]];
        let mut published=vec![];
        for order in orders{
            let mut c=fixture(Character::Ironclad,&cards,0);
            let all=std::mem::take(&mut c.state.piles.draw);
            let by=|i:u64|all.iter().find(|o|o.handle()==card_state::handle(CardInstanceId(i))).unwrap().clone();
            c.state.piles.hand=vec![by(1)];c.state.piles.draw=order.iter().map(|i|by(*i as u64)).collect();
            c.step_full(&CombatAction::PlayCard{instance:1,target:None}).unwrap();
            let candidates=c.public_full().unwrap().selection.expect("secret grid opens").candidates.iter().map(|v|v.instance.0).collect::<Vec<_>>();
            let mut expected=candidates.clone();
            expected.sort_by_key(|i|{let v=c.public_full().unwrap().selection.unwrap().candidates.into_iter().find(|v|v.instance.0==*i).unwrap();(v.id.as_str(),v.instance)});
            assert_eq!(candidates,expected,"{secret:?} order {order:?}");
            published.push(candidates);
        }
        assert!(published.windows(2).all(|w|w[0]==w[1]),"{secret:?}: physical order changes the published candidates {published:?}");
    }
}
