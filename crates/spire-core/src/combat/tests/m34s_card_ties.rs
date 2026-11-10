//! M34s-a: unpublished card fields (a0(iv) follow-up). Each field is published, held equal by an invariant, or shown
//! non-distinguishing; the autoplay free-play lifecycle is a global fix (the original clears `freeToPlayOnce` and
//! `isInAutoplay` after every use, `UseCardAction.update`).
use super::super::*;
use crate::{combat::{card_state,model::OwnedCard},content::{CardId,Character,EncounterId,PotionId},items::CombatItemContext,run::RunRng};
use CardId::*;
fn fixture(character:Character,cards:&[(CardId,u32)])->Combat{fixture_with(character,cards,vec![None,None,None],0)}
fn fixture_with(character:Character,cards:&[(CardId,u32)],potions:Vec<Option<PotionId>>,opening:usize)->Combat{
    let mut entry=enemies::setup(EncounterId::Cultist,1,0,&mut RunRng::new([0x43;32],rng::Partition::Test,0),enemies::SetupContext{
        player:PlayerState{hp:80,max_hp:80,block:0,energy:3,max_energy:3,draw_per_turn:5,powers:vec![]},player_powers:vec![],character,
        cards:vec![permanent_live_card(Defend,CardInstanceId(99),0)],opening_draw:opening,encounter_visit:0}).unwrap();
    entry.cards=cards.iter().enumerate().map(|(index,(id,up))|complete_live_card(*id,CardInstanceId(index as u64+1),*up).unwrap()).collect();
    entry.opening_draw=opening;
    let mut items=Some(CombatItemContext::new(vec![],potions).with_complete_cards().with_complete_items());
    Combat::enter_with_item_context(entry,&mut items).unwrap()
}
fn take(c:&mut Combat,instance:u64)->OwnedCard{
    let handle=card_state::handle(CardInstanceId(instance));
    let piles=&mut c.state.piles;
    for pile in [&mut piles.draw,&mut piles.hand,&mut piles.discard,&mut piles.exhaust]{
        if let Some(index)=pile.iter().position(|o|o.handle()==handle){return pile.remove(index)}
    }
    panic!("card {instance} not in a pile")
}
/// Havoc autoplays the top card; once that use finishes the card is an ordinary card again: not published as free and
/// paid for in full when next played from hand (the original clears the autoplay flag in `UseCardAction.update`).
#[test]
fn autoplayed_card_is_not_free_after_it_finishes(){
    let mut c=fixture(Character::Ironclad,&[(Havoc,0),(Strike,0)]);
    let (havoc,strike)=(take(&mut c,1),take(&mut c,2));
    c.state.piles.hand=vec![havoc];c.state.piles.draw=vec![strike];
    c.step_full(&CombatAction::PlayCard{instance:1,target:None}).unwrap();
    let view=c.public_full().unwrap();
    let played=view.exhaust.iter().find(|card|card.instance==CardInstanceId(2)).expect("Havoc exhausts its child");
    assert!(!played.values.as_ref().unwrap().free_play,"a finished autoplay child is not published as free");
    let strike=take(&mut c,2);c.state.piles.hand.push(strike);
    let energy=c.state.player.energy;
    c.step_full(&CombatAction::PlayCard{instance:2,target:Some(c.public_full().unwrap().targets[0].0.clone())}).unwrap();
    assert_eq!(c.state.player.energy,energy-1,"the former autoplay child costs its price when played from hand");
}
/// The same law through the Standard potion path: Distilled Chaos autoplays the top card, which then sits in discard
/// as an ordinary card (no autoplay waiver; full price from hand).
#[test]
fn distilled_chaos_card_is_not_free_after_it_finishes(){
    let mut c=fixture_with(Character::Silent,&[(Neutralize,0),(Survivor,0)],vec![Some(PotionId::DistilledChaos),None,None],0);
    let (neutralize,survivor)=(take(&mut c,1),take(&mut c,2));
    c.state.piles.hand=vec![];c.state.piles.draw=vec![neutralize,survivor];
    c.step_full(&CombatAction::UsePotion{slot:0,target:None}).unwrap();
    let view=c.public_full().unwrap();
    for instance in [1,2]{
        let card=view.discard.iter().find(|card|card.instance==CardInstanceId(instance)).expect("Distilled Chaos discards its children");
        assert!(!card.values.as_ref().unwrap().free_play&&!card.lifecycle.free_play,"card {instance} keeps no autoplay waiver");
    }
    let survivor=take(&mut c,2);c.state.piles.hand.push(survivor);
    let energy=c.state.player.energy;
    c.step_full(&CombatAction::PlayCard{instance:2,target:None}).unwrap();
    if c.public_full().unwrap().selection.is_some(){
        let pick=c.public_full().unwrap().selection.unwrap().candidates[0].instance.0;
        c.step_full(&CombatAction::SelectCard{instance:pick}).unwrap();
    }
    assert_eq!(c.state.player.energy,energy-1,"Survivor costs 1 from hand after Distilled Chaos played it");
}

// --- canonical_key_ties_are_gameplay_identical (a0(iv); PM ruling on open item 5) ---
// Two draw cards equal in every published value (row and lifecycle) may still differ in an unpublished field. For a
// field classified non-distinguishing, exchanging that field between the two physical cards leaves every public
// observation and the outcome identical along a fixed play script. The negative control shows the harness detects a
// relational difference (shared copy family), which is why live families must be singletons (the a2 knowledge fails
// closed otherwise).
fn normalized(view:&CombatPublic)->String{
    // CombatLocal allocation ids are labels (quotiented); everything else is compared verbatim.
    let text=format!("{view:?}");let mut out=String::new();let mut rest=text.as_str();
    while let Some(at)=rest.find("CombatLocal("){
        out.push_str(&rest[..at+12]);rest=&rest[at+12..];
        let digits=rest.chars().take_while(char::is_ascii_digit).count();out.push('_');rest=&rest[digits..];
    }
    out.push_str(rest);out
}
fn trace(mut c:Combat,steps:usize)->Vec<String>{
    let mut out=vec![];
    for _ in 0..steps{
        let view=c.public_full().unwrap();out.push(normalized(&view));
        if view.outcome.is_some(){break}
        let actions=c.legal_actions_full().unwrap();
        let target=view.targets.iter().find(|(_,live)|*live).map(|(t,_)|t.clone());
        let action=actions.iter().find(|a|matches!(a,CombatAction::PlayCard{..})).cloned().map(|a|match a{
            CombatAction::PlayCard{instance,target:Some(_)}=>CombatAction::PlayCard{instance,target:target.clone()},a=>a})
            .or_else(||actions.iter().find(|a|matches!(a,CombatAction::SelectCard{..}|CombatAction::ConfirmSelection)).cloned())
            .unwrap_or(CombatAction::EndTurn);
        c.step_full(&action).unwrap();
    }
    out
}
type Edit=fn(&mut LiveCard,bool);
fn tie_pair(cards:&[(CardId,u32)],edit:Edit)->(Combat,Combat){
    let make=|swap:bool|{
        let mut c=fixture_with(Character::Ironclad,cards,vec![None,None,None],0);
        for owned in &mut c.state.piles.draw{
            let live=owned.full_mut().unwrap();
            if live.instance.0<=2{live.origin=CombatCardRef::CombatLocal(live.instance.0+40);edit(live,(live.instance.0==1)!=swap);}
        }
        c
    };
    (make(false),make(true))
}
fn assert_tie_is_gameplay_identical(name:&str,cards:&[(CardId,u32)],edit:Edit){
    let (a,b)=tie_pair(cards,edit);
    let card=|c:&Combat,i:u64|super::super::view::live_card_view(&c.state,c.state.piles.draw.iter().find(|o|o.handle()==card_state::handle(CardInstanceId(i))).unwrap().full().unwrap());
    let quotient=|v:LiveCardView|(v.id,v.upgrades,v.misc,v.effective_cost,v.values,v.watcher,v.lifecycle);
    assert_eq!(quotient(card(&a,1)),quotient(card(&a,2)),"{name}: the two cards publish equal rows");
    assert_ne!(a.state,b.state,"{name}: the exchanged field really differs");
    assert_eq!(trace(a,40),trace(b,40),"{name}: exchanging the unpublished field changes gameplay");
}
#[test]
fn canonical_key_ties_are_gameplay_identical(){
    let deck=[(Strike,1),(Strike,1),(Defend,0),(Bash,0),(Defend,0),(Strike,0),(Defend,0),(Strike,0)];
    assert_tie_is_gameplay_identical("temporary_upgrades",&deck,|c,first|c.local.temporary_upgrades=u32::from(first));
    assert_tie_is_gameplay_identical("copy_provenance",&deck,|c,first|c.copy_provenance=if first{card_state::CopyProvenance::Generated}else{card_state::CopyProvenance::StatEquivalent});
    assert_tie_is_gameplay_identical("combat_local_id",&deck,|c,first|c.origin=CombatCardRef::CombatLocal(if first{41}else{42}));
    let rampage=[(Rampage,0),(Rampage,0),(Defend,0),(Bash,0),(Defend,0),(Rampage,0),(Defend,0),(Strike,0)];
    assert_tie_is_gameplay_identical("copy_family_label",&rampage,|c,first|c.copy_family=if first{70}else{71});
}
/// Negative control: a shared copy family is a hidden relation that changes gameplay (Rampage grows its whole family).
#[test]
fn shared_copy_family_is_detected_by_the_tie_harness(){
    let rampage=[(Rampage,0),(Rampage,0),(Defend,0),(Bash,0),(Defend,0),(Rampage,0),(Defend,0),(Strike,0)];
    let (singletons,_)=tie_pair(&rampage,|c,first|c.copy_family=if first{70}else{71});
    let (shared,_)=tie_pair(&rampage,|c,_|c.copy_family=70);
    assert_ne!(trace(singletons,40),trace(shared,40),"the harness must detect a shared family");
}
