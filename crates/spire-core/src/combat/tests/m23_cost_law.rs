//! M23 Confused / Snecko Oil cost-law checklist (user rules, PM addendum M23-D1).
//! Source law: ConfusionPower.onCardDraw rerolls cost>=0 cards and, only when the
//! roll differs from the current cost, sets both `cost` (combat) and `costForTurn`.
//! Wiki captures cited are data/cards|powers/sources/wiki-pages.json (title, revision).
//! Owner-level fixtures: the Confused power is installed directly and prior cost
//! states are the exact writes of their named owners; the shared resolver applies
//! draw, Snecko Oil, HP loss, upgrades and turn expiry.
use super::super::*;
use crate::{content::{CardId,Character,EncounterId,PotionId},items::{CombatItemContext,NativeBattleContext}};

fn combat(character:Character)->Combat{combat_items(character,vec![None;3])}
/// Silent uses the accepted M15 item profile and Defect the M20 battle context,
/// so a belt Snecko Oil is a real potion owner.
fn combat_items(character:Character,potions:Vec<Option<PotionId>>)->Combat{
    let mut e=super::m6_ordinary::entry(EncounterId::Snecko);
    e.setup.character=character;e.setup.player.energy=9;e.setup.player.max_energy=9;e.opening_draw=0;
    let items=CombatItemContext::new(vec![],potions);
    let items=match character{Character::Silent=>items.with_m15_profile(),Character::Defect=>items.with_battle_context(NativeBattleContext{elite_trigger:false,boss:false,burning:None,m15:false,m20:true}),_=>items};
    let mut c=Combat::enter_with_item_context(e,&mut Some(items)).unwrap();
    c.state.player_full_powers.push(FullPowerEntry{id:FullPowerId("confused"),state:PowerState{amount:1,skip_first:false,monster_source_grace:false}});
    c.state.piles.draw.clear();c.state.piles.hand.clear();c.state.piles.discard.clear();c
}
fn issue(c:&mut Combat,card:LiveCard){c.state.card_ids_issued.insert(card_state::handle(card.instance));c.state.piles.draw.push(model::OwnedCard::Full(card));}
fn draw(c:&mut Combat){c.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::Draw(1)}],None).unwrap();}
fn hand(c:&Combat,i:u64)->&LiveCard{c.state.piles.hand.iter().filter_map(model::OwnedCard::full).find(|x|x.instance==CardInstanceId(i)).unwrap()}
fn hand_mut(c:&mut Combat,i:u64)->&mut LiveCard{c.state.piles.hand.iter_mut().filter_map(model::OwnedCard::full_mut).find(|x|x.instance==CardInstanceId(i)).unwrap()}
fn visible(c:&Combat,i:u64)->Option<i32>{c.public_full().unwrap().hand.into_iter().find(|v|v.instance==CardInstanceId(i)).unwrap().effective_cost}
fn price(c:&Combat,i:u64)->i32{let card=hand(c,i);card_state::combat_price(card,cards::base_cost(&c.state,card).unwrap()).unwrap()}
/// Draw `card` once under Confused, choosing a fixed card-stream seed (probed on a
/// clone) whose roll differs from `differs_from`: non-vacuous fixture construction.
fn draw_roll(c:&mut Combat,card:LiveCard,differs_from:Option<i32>)->i32{
    let id=card.instance;let stream=c.state.card_rng.as_ref().unwrap().id().clone();
    for seed in 0u8..32{
        let mut probe=c.clone();probe.state.card_rng=Some(rng::StreamState::new(stream.clone(),[seed;32],0).unwrap());issue(&mut probe,card.clone());draw(&mut probe);
        let r=hand(&probe,id.0).local.randomized_cost.expect("Confused roll");
        if differs_from!=Some(r){c.state.card_rng=Some(rng::StreamState::new(stream,[seed;32],0).unwrap());issue(c,card);draw(c);assert_eq!(hand(c,id.0).local.randomized_cost,Some(r));return r}
    }panic!("no differing roll")
}
/// Shared in-combat upgrade owner (Armaments, Apotheosis, Blessing of the Forge).
fn upgrade(c:&mut Combat,i:u64){let card=hand(c,i).clone();let up=card_state::upgrade_live_card(&c.state,&card).unwrap();*hand_mut(c,i)=up;}
fn expire(c:&mut Combat){piles::expire_turn_scoped_costs(&mut c.state);}
/// Actual belt Snecko Oil use (draw from the cleared piles, then randomize hand costs).
fn oil(c:&mut Combat){let slot=c.state.item_context.as_ref().unwrap().potions().iter().position(|p|*p==Some(PotionId::SneckoOil)).expect("belt Oil");c.step_full(&CombatAction::UsePotion{slot,target:None}).unwrap();}
fn play(c:&mut Combat,i:u64){let a=c.legal_actions_full().unwrap().into_iter().find(|a|matches!(a,CombatAction::PlayCard{instance,..} if *instance==i)).expect("legal play");c.step_full(&a).unwrap();}

#[test]
fn m23_cost_law_corruption_keeps_skills_zero_under_confused_and_oil(){
    // Corruption, rev 56614: "the cost of all Skill cards will remain 0".
    let mut c=combat(Character::Ironclad);
    c.state.player_full_powers.push(FullPowerEntry{id:FullPowerId("corruption"),state:PowerState{amount:1,skip_first:false,monster_source_grace:false}});
    let r=draw_roll(&mut c,permanent_live_card(CardId::ShrugItOff,CardInstanceId(1),0),Some(0));assert!(r>0);
    assert_eq!(visible(&c,1),Some(0),"Confused roll does not price a Skill under Corruption");
    // Snecko Oil is not admitted in any Corruption profile (M10); not reachable here.
    let energy=c.state.player.energy;c.step_full(&CombatAction::PlayCard{instance:1,target:None}).unwrap();assert_eq!(c.state.player.energy,energy);
}

#[test]
fn m23_cost_law_force_field_roll_replaces_cost_and_power_discount_continues(){
    // User rule: randomized cost replaces Force Field's current cost; its power-played reduction continues from it.
    let mut c=combat_items(Character::Defect,vec![Some(PotionId::SneckoOil),None,None]);
    let mut ff=permanent_live_card(CardId::ForceField,CardInstanceId(1),0);card_state::discount_force_field(&mut ff);assert_eq!(ff.local.mutable_combat_cost,Some(3));
    let r=draw_roll(&mut c,ff,Some(3));assert_eq!(price(&c,1),r);assert_eq!(visible(&c,1),Some(r));
    card_state::discount_force_field(hand_mut(&mut c,1));assert_eq!(price(&c,1),(r-1).max(0).min(r));
    let after=price(&c,1);expire(&mut c);assert_eq!(price(&c,1),after,"combat cost persists after turn expiry");assert_eq!(visible(&c,1),Some(after));
    // Snecko Oil roll replaces it again; the discount then continues from the new roll.
    oil(&mut c);let o=hand(&c,1).local.randomized_cost.unwrap();if o!=after{assert_eq!(price(&c,1),o);}
    let before=price(&c,1);card_state::discount_force_field(hand_mut(&mut c,1));assert_eq!(price(&c,1),(before-1).max(0).min(before));
}

#[test]
fn m23_cost_law_blood_for_blood_roll_replaces_and_upgrade_always_lowers_by_one(){
    // Blood for Blood, rev 56417: costs more than 0 "will always go down by 1 in Energy-cost when Upgraded, regardless of what cost it was randomized to".
    let mut c=combat(Character::Ironclad);
    let r=draw_roll(&mut c,permanent_live_card(CardId::BloodForBlood,CardInstanceId(1),0),Some(0));assert!(r>0);
    assert_eq!(visible(&c,1),Some(r));
    c.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::Damage(DamagePayload::HpLoss(1))}],None).unwrap();
    assert_eq!(visible(&c,1),Some(r-1),"HP-loss reduction continues from the randomized value");
    let now=r-1;
    upgrade(&mut c,1);
    assert_eq!(visible(&c,1),Some((now-1).max(0)),"upgrade lowers a positive randomized Blood for Blood by exactly one");
}

#[test]
fn m23_cost_law_masterful_stab_roll_overrides_and_increase_continues(){
    // Masterful Stab, rev 57491: randomization overrides; "the Energy-cost will continue increasing as expected".
    let mut c=combat(Character::Silent);
    let mut stab=permanent_live_card(CardId::MasterfulStab,CardInstanceId(1),0);card_state::modify_cost_for_combat(&mut stab,2).unwrap();
    let prior=card_state::combat_price(&stab,0).unwrap();
    let r=draw_roll(&mut c,stab,Some(prior));assert_eq!(price(&c,1),r,"roll overrides the increased cost");
    c.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::Damage(DamagePayload::HpLoss(1))}],None).unwrap();
    assert_eq!(price(&c,1),r+1);assert_eq!(visible(&c,1),Some(r+1));expire(&mut c);assert_eq!(price(&c,1),r+1);
}

#[test]
fn m23_cost_law_enlightenment_madness_and_generated_costs_are_overridden(){
    // Enlightenment rev 66701, Madness rev 66711, Chrysalis rev 66754, Metamorphosis rev 66758: randomization overrides.
    let mut c=combat(Character::Silent);
    // Exact prior writes: Enlightenment+ sets combat cost 1 (modify owner); Madness and generated CombatZero set 0.
    let mut enlightened=permanent_live_card(CardId::Backflip,CardInstanceId(1),0);enlightened.local.mutable_combat_cost=Some(1);enlightened.local.turn_cost=Some(1);
    let mut mad=permanent_live_card(CardId::Backflip,CardInstanceId(2),0);mad.local.mutable_combat_cost=Some(0);mad.local.turn_cost=Some(0);
    let mut generated=card_state::generated(CardId::Backflip,CardInstanceId(3),0).unwrap();generated.local.mutable_combat_cost=Some(0);generated.local.turn_cost=Some(0);
    for (card,prior) in [(enlightened,1),(mad,0),(generated,0)]{
        let i=card.instance.0;let r=draw_roll(&mut c,card,Some(prior));assert_eq!(price(&c,i),r);assert_eq!(visible(&c,i),Some(r));expire(&mut c);assert_eq!(price(&c,i),r,"override persists for the combat");
    }
    // Snecko Oil overrides the same combat-cost owners.
    let mut c=combat_items(Character::Silent,vec![Some(PotionId::SneckoOil);3]);let mut card=permanent_live_card(CardId::Backflip,CardInstanceId(9),0);card.local.mutable_combat_cost=Some(0);card.local.turn_cost=Some(0);
    c.state.card_ids_issued.insert(card_state::handle(card.instance));c.state.piles.hand.push(model::OwnedCard::Full(card));
    for _ in 0..3{oil(&mut c);if hand(&c,9).local.randomized_cost.is_some_and(|o|o!=0){break}}
    let o=hand(&c,9).local.randomized_cost.expect("an Oil roll");
    if o!=0{assert_eq!(price(&c,9),o);expire(&mut c);assert_eq!(price(&c,9),o);}else{assert_eq!(price(&c,9),0);}
}

#[test]
fn m23_cost_law_eviscerate_roll_is_combat_cost_and_equal_roll_keeps_discount(){
    // Eviscerate rev 57257: drawn under Confused the cost is randomized regardless of discards; the
    // discount is turn-only. Source onCardDraw changes cost only when the roll differs, so an equal
    // roll keeps the turn discount.
    let mut c=combat(Character::Silent);c.state.silent.discarded_this_turn=2;
    let r=draw_roll(&mut c,permanent_live_card(CardId::Eviscerate,CardInstanceId(1),0),Some(3));
    assert_eq!(visible(&c,1),Some(r));assert_eq!(price(&c,1),r);
    // Equal roll: probe the next roll on a clone, then make the card's combat cost equal it.
    let mut c=combat(Character::Silent);c.state.silent.discarded_this_turn=2;
    // Non-vacuous: choose a card-stream seed whose roll is at least 1 (a 0 roll hides the discount).
    let stream=c.state.card_rng.as_ref().unwrap().id().clone();let mut roll=0;
    for seed in 0u8..32{let mut probe=c.clone();probe.state.card_rng=Some(rng::StreamState::new(stream.clone(),[seed;32],0).unwrap());issue(&mut probe,permanent_live_card(CardId::Eviscerate,CardInstanceId(1),0));draw(&mut probe);
        roll=hand(&probe,1).local.randomized_cost.unwrap();if roll>=1{c.state.card_rng=Some(rng::StreamState::new(stream,[seed;32],0).unwrap());break}}
    assert!(roll>=1);
    let mut ev=permanent_live_card(CardId::Eviscerate,CardInstanceId(1),0);ev.local.mutable_combat_cost=Some(roll);ev.local.turn_cost=Some(roll);
    issue(&mut c,ev);draw(&mut c);
    assert_eq!(visible(&c,1),Some((roll-2).max(0)),"equal roll leaves the discounted turn price");assert_eq!(price(&c,1),roll);
}

#[test]
fn m23_cost_law_bullet_time_zeroes_randomized_hand_for_the_turn_only(){
    // Bullet Time rev 56919: sets every card to 0 regardless of randomized costs (this turn).
    let mut c=combat(Character::Silent);
    let r=draw_roll(&mut c,permanent_live_card(CardId::Backflip,CardInstanceId(1),0),Some(0));
    let bt=permanent_live_card(CardId::BulletTime,CardInstanceId(2),0);c.state.card_ids_issued.insert(card_state::handle(bt.instance));c.state.piles.hand.push(model::OwnedCard::Full(bt));
    play(&mut c,2);
    assert_eq!(visible(&c,1),Some(0));expire(&mut c);assert_eq!(visible(&c,1),Some(r),"turn ends: the randomized combat cost returns");
}

#[test]
fn m23_cost_law_upgrade_of_randomized_card_uses_upgraded_cost_unless_zero(){
    // Armaments rev 56323: upgrade sets the upgraded cost "unless its cost initially got randomized to 0, in which case it remains 0".
    let mut c=combat(Character::Ironclad);
    let r=draw_roll(&mut c,permanent_live_card(CardId::Entrench,CardInstanceId(1),0),Some(0));assert!(r>0);
    upgrade(&mut c,1);assert_eq!(visible(&c,1),Some(1),"Entrench+ cost 1");
    let mut c=combat(Character::Ironclad);
    let mut zero=permanent_live_card(CardId::Entrench,CardInstanceId(2),0);zero.local.randomized_cost=Some(0);zero.local.mutable_combat_cost=Some(0);zero.local.turn_cost=Some(0);
    c.state.card_ids_issued.insert(card_state::handle(zero.instance));c.state.piles.hand.push(model::OwnedCard::Full(zero));
    upgrade(&mut c,2);assert_eq!(visible(&c,2),Some(0),"randomized-to-0 remains 0 when upgraded");
}

#[test]
fn m23_cost_law_streamline_reduction_continues_from_roll(){
    // User rule: the card's own law continues from the randomized value.
    let mut c=combat(Character::Defect);
    let r=draw_roll(&mut c,permanent_live_card(CardId::Streamline,CardInstanceId(1),0),Some(2));
    play(&mut c,1);
    let card=c.state.piles.discard.iter().filter_map(model::OwnedCard::full).find(|x|x.instance==CardInstanceId(1)).unwrap();
    assert_eq!(card_state::combat_price(card,2).unwrap(),(r-1).max(0),"Streamline's own reduction continues from its randomized cost");
}

#[test]
fn m23_cost_law_confused_draw_clears_setup_and_forethought_free_to_play_once(){
    // Source ConfusionPower.onCardDraw ends with card.freeToPlayOnce=false; Forethought rev 66704 update
    // history: "Fixed Confusion card not affecting cards reduced by Setup or Forethought".
    let mut c=combat(Character::Silent);
    // Exact Setup/Forethought write: positive combat cost -> free_to_play_once.
    let mut card=permanent_live_card(CardId::Backflip,CardInstanceId(1),0);card.local.free_to_play_once=true;
    let r=draw_roll(&mut c,card,Some(0));assert!(r>0);
    assert!(!hand(&c,1).local.free_to_play_once,"Confused draw clears freeToPlayOnce");
    assert_eq!(visible(&c,1),Some(r),"the Setup card now costs its roll");
    let energy=c.state.player.energy;play(&mut c,1);assert_eq!(c.state.player.energy,energy-r,"the roll is paid");
}

#[test]
fn m23_cost_law_snecko_oil_clears_setup_and_forethought_free_to_play_once(){
    // Source RandomizeHandCostAction: every cost>=0 hand card ends with freeToPlayOnce=false.
    let mut c=combat_items(Character::Silent,vec![Some(PotionId::SneckoOil);3]);
    let card=permanent_live_card(CardId::Backflip,CardInstanceId(9),0);c.state.card_ids_issued.insert(card_state::handle(card.instance));c.state.piles.hand.push(model::OwnedCard::Full(card));
    let mut nonzero=None;
    for _ in 0..3{hand_mut(&mut c,9).local.free_to_play_once=true;oil(&mut c);
        assert!(!hand(&c,9).local.free_to_play_once,"Snecko Oil clears freeToPlayOnce for every eligible card");
        let roll=hand(&c,9).local.randomized_cost;if let Some(r)=roll.filter(|r|*r>0){nonzero=Some(r);break}}
    let r=nonzero.expect("a nonzero Oil roll within the three belt potions");assert_eq!(visible(&c,9),Some(r));
}
