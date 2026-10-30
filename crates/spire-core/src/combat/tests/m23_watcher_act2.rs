//! M23 Watcher x Act2 composition diagnostics on the accepted shared owners.
//! Every test below is a labelled constructed owner-state diagnostic: actual A0
//! setup, actual actors and the shared resolver, with listed pile/HP/intent
//! arrangements. None is a played route, natural entry or earned resource, and
//! none counts as a boss win. Expected values are literal source outcomes.
use super::super::*;
use crate::{content::{CardId,Character,EncounterId,PotionId,RelicId},items::CombatItemContext,run::RunRng};
use cards::watcher::WatcherEffect;

fn handle(id:u64)->CardHandle{card_state::handle(CardInstanceId(id))}
fn amount(list:&[FullPowerEntry],id:&str)->Option<i32>{list.iter().find(|p|p.id.0==id).map(|p|p.state.amount)}
fn card(c:&Combat,id:u64)->&LiveCard{piles::owned_full_cards(&c.state).find(|c|c.instance.0==id).unwrap()}
fn in_hand(c:&Combat,id:u64)->bool{c.state.piles.hand.iter().any(|o|o.handle()==handle(id))}
fn in_pile(pile:&[model::OwnedCard],id:u64)->bool{pile.iter().any(|o|o.handle()==handle(id))}
/// Public current price of a hand card at a ready boundary.
fn price(c:&Combat,id:u64)->Option<i32>{c.public_full().unwrap().hand.iter().find(|v|v.instance.0==id).unwrap().effective_cost}
fn census(c:&Combat)->std::collections::BTreeSet<CardHandle>{piles::owned_full_cards(&c.state).map(|c|card_state::handle(c.instance)).collect()}
/// Seed7, 72/72HP, actual A0 Act2 setup and M22 Watcher startup (PureWater pre-draw Miracle),
/// permanent IDs 1..N in listed order.
fn fight(encounter:EncounterId,deck:&[(CardId,u32)],relics:Vec<RelicId>,potions:Vec<Option<PotionId>>)->Combat{
    let cards=deck.iter().enumerate().map(|(i,(id,up))|m21_watcher_live_card(*id,CardInstanceId(i as u64+1),*up).unwrap()).collect::<Vec<_>>();
    let e=enemies::setup(encounter,2,0,&mut RunRng::new([7;32],rng::Partition::Test,0),enemies::SetupContext{
        player:PlayerState{hp:72,max_hp:72,block:0,energy:3,max_energy:3,draw_per_turn:5,powers:vec![]},
        player_powers:vec![],character:Character::Watcher,cards,opening_draw:5,encounter_visit:0}).unwrap();
    let mut items=Some(CombatItemContext::new(relics,potions).with_m22_profile());
    let c=Combat::enter_with_item_context(e,&mut items).unwrap();assert!(items.is_none());c
}
/// Constructed arrangement: move an owned instance between ordinary piles (no issue, no hook).
fn place(c:&mut Combat,id:u64,zone:SelectionSourceZone){
    let h=handle(id);let take=|p:&mut Vec<model::OwnedCard>|p.iter().position(|o|o.handle()==h).map(|i|p.remove(i));
    let owned=take(&mut c.state.piles.hand).or_else(||take(&mut c.state.piles.draw)).or_else(||take(&mut c.state.piles.discard)).unwrap();
    match zone{SelectionSourceZone::Hand=>c.state.piles.hand.push(owned),SelectionSourceZone::Draw=>c.state.piles.draw.push(owned),SelectionSourceZone::Discard=>c.state.piles.discard.push(owned),_=>unreachable!()}
    piles::refresh_hand_values(&mut c.state);
}
// Same prepare/detach/effects/finish/decision ownership as step_full, paused at the actual
// queue boundary (accepted m21_watcher_review pattern) for mid-card inspection.
fn stage_play(c:&mut Combat,instance:u64){
    let prepared=cards::prepare(c,&CombatAction::PlayCard{instance,target:None}).unwrap();
    let source=Source::Card(handle(instance));let mut work=prepared.effects;
    c.state.in_play=Some(c.state.piles.hand.remove(prepared.hand_index));c.state.player.energy=prepared.remaining_energy;c.state.phase=Phase::PlayerAction;
    cards::capture_silent_targets(&mut c.state,&mut work).unwrap();
    c.state.pending.push_back(Effect{source:source.clone(),target:None,kind:EffectKind::Hook(Hook::CardPlayed)});
    c.state.pending.extend(work);c.state.pending.push_back(Effect{source:source.clone(),target:None,kind:EffectKind::Hook(Hook::CardAfterUse)});
    c.state.pending.push_back(Effect{source,target:None,kind:EffectKind::FinishCard});c.state.pending.push_back(Effect{source:Source::None,target:None,kind:EffectKind::Decision});
}
fn play(c:&mut Combat,id:u64,target:Option<u64>){c.step_full(&CombatAction::PlayCard{instance:id,target:target.map(TargetId)}).unwrap_or_else(|e|panic!("play {id}: {e:?} {:?}",c.fault_record()));}

#[test]
fn m23_watcher_stances_compose_with_actual_act2_damage(){
    // constructed owner-state diagnostic; not a played route/natural entry/earned resource.
    // Actual ThreeByrds (A0 Flight3) with the listed Flight deck/belt; only the block in
    // the blocked case below is constructed. Source: card damage is computed at play in
    // the current stance (Wrath x2, Divinity x3, NORMAL only), then Flight halves NORMAL
    // received damage (floor), and a surviving unblocked NORMAL hit queues Flight-1.
    use CardId::*;
    let mut c=fight(EncounterId::ThreeByrds,&[(Eruption,1),(Vigilance,0),(EmptyFist,0),(FlurryOfBlows,0),(Prostrate,0)],vec![RelicId::PureWater,RelicId::VioletLotus],vec![Some(PotionId::StancePotion),Some(PotionId::Ambrosia),None]);
    assert_eq!(c.state.enemies.len(),3);
    for e in &c.state.enemies{assert_eq!(e.enemy_id,ActorEnemyId::Full(crate::content::EnemyId::Byrd));assert_eq!(amount(&e.full_powers,"flight"),Some(3));}
    assert_eq!(c.state.piles.hand.len(),6);assert_eq!((c.state.player.energy,c.state.character.stance),(3,Stance::Neutral));
    let hp=c.state.enemies.iter().map(|e|e.hp).collect::<Vec<_>>();
    // Eruption+ (cost1, 9): hit resolves before the Wrath transition -> floor(9/2)=4.
    play(&mut c,1,Some(1));
    assert_eq!((c.state.enemies[0].hp,amount(&c.state.enemies[0].full_powers,"flight")),(hp[0]-4,Some(2)));
    assert_eq!((c.state.character.stance,c.state.player.energy),(Stance::Wrath,2));
    // Flurry (0, 4) in Wrath: 8 -> Flight 4.
    play(&mut c,4,Some(1));
    assert_eq!((c.state.enemies[0].hp,amount(&c.state.enemies[0].full_powers,"flight")),(hp[0]-8,Some(1)));
    assert!(in_pile(&c.state.piles.discard,4));
    // Paid StancePotion -> Calm: Wrath exit has no energy; Flurry returns from discard once.
    c.step_full(&CombatAction::UsePotion{slot:0,target:None}).unwrap();
    assert_eq!(c.item_context().unwrap().potions()[0],None);
    c.step_full(&CombatAction::ChooseStance{stance:Stance::Calm}).unwrap();
    assert_eq!((c.state.character.stance,c.state.player.energy),(Stance::Calm,2));assert!(in_hand(&c,4));
    // Same-stance Vigilance (2, 8 block): no transition callbacks, no energy, no extra return.
    let hand=c.state.piles.hand.len();play(&mut c,2,None);
    assert_eq!((c.state.character.stance,c.state.player.energy,c.state.player.block),(Stance::Calm,0,8));assert_eq!(c.state.piles.hand.len(),hand-1);
    // Ambrosia -> Divinity: Calm exit 2 + VioletLotus 1, Divinity entry 3.
    c.step_full(&CombatAction::UsePotion{slot:1,target:None}).unwrap();
    assert_eq!((c.state.character.stance,c.state.player.energy),(Stance::Divinity,6));
    // Flurry in Divinity: 12 -> Flight 6; last Flight stack removes and grounds Byrd 1.
    play(&mut c,4,Some(1));
    assert_eq!(c.state.enemies[0].hp,hp[0]-14);assert_eq!(amount(&c.state.enemies[0].full_powers,"flight"),None);
    assert_eq!(c.state.enemies[0].intent_id,ActorIntentId::Full(FullIntentId::from_name("byrd_stunned")));
    // Constructed low HP on flying Byrd 3: EmptyFist (1, 9) in Divinity = 27 -> Flight 13 is
    // lethal; a dying hit queues no Flight reaction, the other Byrds stay, no Victory.
    c.state.enemies[2].hp=10;play(&mut c,3,Some(3));
    assert_eq!(c.state.enemies[2].hp,0);assert_ne!(amount(&c.state.enemies[2].full_powers,"flight"),Some(2));
    assert!(c.state.outcome.is_none());assert!(c.state.enemies[1].hp==hp[1]&&c.state.enemies[0].hp==hp[0]-14);
    // EmptyFist exits to Neutral after its hit; Divinity exit has no energy, Flurry returns again.
    assert_eq!((c.state.character.stance,c.state.player.energy),(Stance::Neutral,5));assert!(in_hand(&c,4));
    // Constructed block on flying Byrd 2: a fully blocked NORMAL hit (4 -> Flight 2) keeps Flight3.
    c.state.enemies[1].block=50;play(&mut c,4,Some(2));
    assert_eq!((c.state.enemies[1].hp,c.state.enemies[1].block,amount(&c.state.enemies[1].full_powers,"flight")),(hp[1],48,Some(3)));
    // Wrath incoming NORMAL doubles; HP_LOSS is unscaled (source atDamageReceive NORMAL only).
    let mut wrath=c.clone();wrath.state.character.stance=Stance::Wrath;effects::refresh_enemy_values(&mut wrath.state).unwrap();
    let mut neutral=c.clone();effects::refresh_enemy_values(&mut neutral.state).unwrap();
    for (w,n) in wrath.state.enemies.iter().zip(&neutral.state.enemies){for (id,entry) in &w.full_damage_entries{assert_eq!(entry.output,n.full_damage_entries[id].output*2);}}
    let before=wrath.state.player.hp;wrath.state.pending.push_back(Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::Damage(DamagePayload::HpLoss(3))});
    assert_ne!(engine::resolve(&mut wrath.state).status,TransitionStatus::Fault);assert_eq!(wrath.state.player.hp,before-3);
    assert!(c.state.outcome.is_none());assert!(engine::validate_runtime(&c.state).is_ok());
}

/// Successive source cardRandomRng tickets (Confused random(3) == uniform 0..=3), using the
/// shared RNG primitive only to name the consumed tickets, never the Confused/cost writers.
fn card_tickets(c:&Combat,stream:&rng::StreamState,n:usize)->(Vec<i32>,rng::StreamState){
    let _=c;let mut s=stream.clone();let mut out=vec![];for _ in 0..n{let (t,next)=rng::uniform_below(&s,4).unwrap();out.push(t as i32);s=next;}(out,s)
}
#[test]
fn m23_confused_draw_and_retention_use_current_instance_price(){
    // constructed owner-state diagnostic; not a played route/natural entry/earned resource.
    // Actual Snecko (A0 opening Glare -> Confused 1). Constructed arrangement: Sands starts in
    // draw so that it is genuinely drawn under Confused. Source ConfusionPower.onCardDraw sets
    // the drawn card's combat cost and turn cost to cardRandomRng.random(3); Establishment and
    // Sands then call modifyCostForCombat(-1) on that same instance; movement is not a draw.
    use CardId::*;
    let mut c=fight(EncounterId::Snecko,&[(SandsOfTime,0),(Establishment,1),(Meditate,0),(WindmillStrike,0),(Vigilance,0)],vec![RelicId::PureWater],vec![None;3]);
    assert_eq!(c.state.enemies[0].intent_id,ActorIntentId::Full(FullIntentId::from_name("snecko_glare")));
    assert_eq!(c.state.piles.hand.len(),6);place(&mut c,1,SelectionSourceZone::Draw);
    let miracle=c.state.piles.hand.iter().filter_map(|o|o.full()).find(|c|c.id==Miracle).unwrap().instance.0;
    play(&mut c,2,None);play(&mut c,5,None);assert_eq!((c.state.player.energy,c.state.character.stance),(0,Stance::Calm));
    let stream=c.state.card_rng.clone().unwrap();
    c.step_full(&CombatAction::EndTurn).unwrap();
    assert_eq!(amount(&c.state.player_full_powers,"confused"),Some(1));assert_eq!(c.state.turn,2);
    // Retained, not drawn: Windmill 7+4=11 and Establishment 2-1=1; Miracle stays 0.
    assert_eq!((card(&c,4).local.base_damage,price(&c,4),price(&c,miracle)),(Some(11),Some(1),Some(0)));
    // Exactly three actual draws (Sands, then the reshuffled Meditate/Vigilance), one ticket each.
    let drawn=c.state.piles.hand.iter().skip(2).map(|o|o.handle()).collect::<Vec<_>>();
    assert_eq!(drawn.len(),3);assert_eq!(drawn[0],handle(1));
    let (t,after)=card_tickets(&c,&stream,3);assert_eq!(c.state.card_rng.as_ref(),Some(&after));
    let r=|id:u64|t[drawn.iter().position(|h|*h==handle(id)).unwrap()];
    for id in [1,3,5]{assert_eq!(price(&c,id),Some(r(id)),"Confused price of {id}");}
    // Exact current prices decide legality: Miracle +1 => 4 energy.
    play(&mut c,miracle,None);assert_eq!(c.state.player.energy,4);
    play(&mut c,5,None);let energy=4-r(5);assert_eq!((c.state.player.energy,c.state.character.stance),(energy,Stance::Calm));
    let meditated=r(3)<=energy;
    if meditated{
        // Discard holds only Vigilance, so source Meditate (discard<=amount) returns it without a modal.
        assert_eq!(c.state.piles.discard.iter().map(|o|o.handle()).collect::<Vec<_>>(),vec![handle(5)]);
        let mut paused=c.clone();stage_play(&mut paused,3);
        while !paused.state.pending.is_empty()&&!matches!(paused.state.pending.front().map(|e|&e.kind),Some(EffectKind::Watcher(WatcherEffect::RequestEndTurn))){engine::dispatch_one(&mut paused.state);assert!(paused.fault_record().is_none());}
        // Meditate's movement is not a draw: same instance, same Confused price, no card ticket.
        assert!(in_hand(&paused,5));assert_eq!(paused.state.card_rng,c.state.card_rng);assert_eq!(card(&paused,5).local.turn_cost,Some(r(5)));
        play(&mut c,3,None);assert!(c.public_full().unwrap().selection.is_none());
    }else{
        let before=c.clone();assert!(c.step_full(&CombatAction::PlayCard{instance:3,target:None}).is_err());assert_eq!(c,before);
        c.step_full(&CombatAction::EndTurn).unwrap();
    }
    assert_eq!(c.state.turn,3);
    // Retained instances keep their own current price (source cost persists for the combat):
    // Windmill 1->0; Sands r-1 (Establishment) then -1 (onRetained); Meditated Vigilance r-1.
    let floor=|v:i32|v.max(0);
    assert_eq!((card(&c,4).local.base_damage,price(&c,4)),(Some(15),Some(0)));
    // Law under test: collect every retained-price mismatch so one run reports all of them.
    let mut wrong=vec![];let mut check=|what:&str,c:&Combat,id:u64,r:i32,expected:i32|{let got=price(c,id);if got!=Some(expected){wrong.push(format!("{what}: drawn r={r} expected {expected} got {got:?} local={:?}",card(c,id).local));}};
    check("Establishment+retained Confused Sands",&c,1,r(1),floor(floor(r(1)-1)-1));
    if meditated{check("Establishment+Meditated Confused Vigilance",&c,5,r(5),floor(r(5)-1));}

    // Second constructed fixture without Establishment: self-retained cards genuinely drawn under
    // Confused keep that drawn price on later turns; the undrawn sibling keeps its base price.
    let mut c=fight(EncounterId::Snecko,&[(WindmillStrike,0),(Perseverance,0),(SandsOfTime,0),(DefendPurple,0),(DefendPurple,0),(DefendPurple,0)],vec![],vec![None;3]);
    for id in [1,2,3]{place(&mut c,id,SelectionSourceZone::Draw);}
    for id in [4,5,6]{if !in_hand(&c,id){place(&mut c,id,SelectionSourceZone::Hand);}}
    let stream=c.state.card_rng.clone().unwrap();c.step_full(&CombatAction::EndTurn).unwrap();
    let drawn=c.state.piles.hand.iter().map(|o|o.handle()).collect::<Vec<_>>();assert_eq!(drawn.len(),5);
    let (t,_)=card_tickets(&c,&stream,5);let r=|id:u64|t[drawn.iter().position(|h|*h==handle(id)).unwrap()];
    for id in [1,2,3]{assert_eq!(price(&c,id),Some(r(id)),"Confused price of {id}");}
    let sibling=[4,5,6].into_iter().find(|id|in_pile(&c.state.piles.draw,*id)).unwrap();
    let s=card(&c,sibling);assert_eq!((s.local.randomized_cost,s.local.mutable_combat_cost),(None,Some(1)),"undrawn sibling");
    // Creation while Confused is not a draw: a queued Miracle child consumes no card ticket.
    let rng=c.state.card_rng.clone();let kind=cards::watcher::create_action(&c.state,Miracle,1).unwrap();
    c.state.pending.push_back(Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind});
    assert_ne!(engine::resolve(&mut c.state).status,TransitionStatus::Fault);assert_eq!(c.state.card_rng,rng);
    let child=c.state.piles.hand.iter().filter_map(|o|o.full()).find(|c|c.id==Miracle).unwrap().instance.0;assert_eq!(price(&c,child),Some(0));
    c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.state.turn,3);
    check("retained Confused Windmill",&c,1,r(1),r(1));
    check("retained Confused Perseverance",&c,2,r(2),r(2));
    check("retained Confused Sands",&c,3,r(3),floor(r(3)-1));
    assert!(wrong.is_empty(),"Confused current-price owner mismatches:\n{}",wrong.join("\n"));
}

fn handles(pile:&[model::OwnedCard])->Vec<CardHandle>{pile.iter().map(|o|o.handle()).collect()}
fn ids(v:&[u64])->Vec<CardHandle>{v.iter().map(|id|handle(*id)).collect()}
fn set(v:&[u64])->std::collections::BTreeSet<CardHandle>{v.iter().map(|id|handle(*id)).collect()}
fn dazed(pile:&[model::OwnedCard])->Vec<&LiveCard>{pile.iter().filter_map(|o|o.full()).filter(|c|c.id==CardId::Dazed).collect()}
#[test]
fn m23_power_scry_and_hex_suspend_real_parents(){
    // constructed owner-state diagnostic; not a played route/natural entry/earned resource.
    // Actual ChosenAndByrds ([Byrd, Chosen]); Chosen opens Poke then Hex. Constructed: the
    // PowerScry deck is extended with Defends 6..15 and the opening piles are arranged
    // (hand 1..5, draw 6..15 with 15 on top) so real Scry windows exist.
    use CardId::*;
    let mut deck=vec![(Foresight,1),(ThirdEye,0),(Weave,0),(Nirvana,0),(CutThroughFate,0)];deck.extend((6..=15).map(|_|(DefendPurple,0)));
    let mut c=fight(EncounterId::ChosenAndByrds,&deck,vec![RelicId::GoldenEye],vec![None;3]);
    assert_eq!(c.state.enemies[1].enemy_id,ActorEnemyId::Full(crate::content::EnemyId::Chosen));
    assert_eq!(c.state.enemies[1].intent_id,ActorIntentId::Full(FullIntentId::from_name("chosen_poke")));
    for id in 1..=5{place(&mut c,id,SelectionSourceZone::Hand);}for id in 6..=15{place(&mut c,id,SelectionSourceZone::Draw);}
    assert_eq!(handles(&c.state.piles.draw),ids(&(6..=15).collect::<Vec<_>>()));assert_eq!(c.state.piles.hand.len(),5);
    let chosen=c.state.enemies[1].hp;
    play(&mut c,1,None);play(&mut c,4,None);assert_eq!((amount(&c.state.player_full_powers,"foresight"),amount(&c.state.player_full_powers,"nirvana")),(Some(4),Some(3)));
    play(&mut c,3,Some(2));assert_eq!(c.state.enemies[1].hp,chosen-4);
    // CutThroughFate: paid hit, then card-parent Scry 2+GoldenEye2, then the suspended draw 1.
    play(&mut c,5,Some(2));assert_eq!((c.state.enemies[1].hp,c.state.player.energy),(chosen-11,0));
    let s=c.public_full().unwrap().selection.unwrap();assert_eq!(s.parent,SelectionParent::Card(handle(5)));
    assert_eq!(s.candidates.iter().map(|v|v.instance.0).collect::<Vec<_>>(),vec![15,14,13,12]);assert_eq!((s.minimum,s.maximum),(0,4));
    assert_eq!(c.public_full().unwrap().hand.len(),1,"only ThirdEye; the later draw has not happened at the modal");
    // Stale/hidden/cross-zone picks reject atomically: below the window, in discard, in hand.
    let before=c.clone();for bad in [6,3,2,999]{assert!(c.step_full(&CombatAction::SelectCard{instance:bad}).is_err());assert_eq!(c,before);}
    for id in [15,14]{c.step_full(&CombatAction::SelectCard{instance:id}).unwrap();}
    c.step_full(&CombatAction::DeselectCard{instance:15}).unwrap();c.step_full(&CombatAction::SelectCard{instance:15}).unwrap();
    assert_eq!(c.public_full().unwrap().selection.unwrap().selected.iter().map(|i|i.0).collect::<Vec<_>>(),vec![14,15]);
    c.step_full(&CombatAction::ConfirmSelection).unwrap();assert!(c.public_full().unwrap().selection.is_none());
    // Selected order reaches discard; Weave returns once; the draw takes the new top 13; Nirvana once.
    assert_eq!(handles(&c.state.piles.discard),ids(&[14,15,5]));assert_eq!(set(&handles(&c.state.piles.hand).iter().map(|h|h.as_str().parse().unwrap()).collect::<Vec<u64>>()),set(&[2,3,13]));
    assert_eq!(handles(&c.state.piles.draw),ids(&(6..=12).collect::<Vec<_>>()));assert_eq!(c.state.player.block,3);
    // Foresight owns the turn-2 pre-draw Scry (4+2) with the normal draw queued behind it.
    c.step_full(&CombatAction::EndTurn).unwrap();
    let s=c.public_full().unwrap().selection.expect("Foresight suspends the turn-start draw");
    assert_eq!(s.parent,SelectionParent::Power{turn:2,id:FullPowerId("foresight")});
    assert_eq!(s.candidates.iter().map(|v|v.instance.0).collect::<Vec<_>>(),vec![12,11,10,9,8,7]);
    assert!(c.state.piles.hand.is_empty());assert_eq!(c.public_full().unwrap().draw_count,7);
    let before=c.clone();for bad in [6,14,3]{assert!(c.step_full(&CombatAction::SelectCard{instance:bad}).is_err());assert_eq!(c,before);}
    assert!(c.step_full(&CombatAction::EndTurn).is_err());assert_eq!(c,before);
    for id in [12,11]{c.step_full(&CombatAction::SelectCard{instance:id}).unwrap();}
    c.step_full(&CombatAction::DeselectCard{instance:12}).unwrap();c.step_full(&CombatAction::SelectCard{instance:12}).unwrap();
    c.step_full(&CombatAction::ConfirmSelection).unwrap();assert!(c.public_full().unwrap().selection.is_none());
    let tail=handles(&c.state.piles.discard);assert_eq!(tail[tail.len()-2..].to_vec(),ids(&[11,12]));
    // Resumed once: normal draw 5 takes 10..6, Weave returns once from discard, Nirvana +3 once.
    assert_eq!(set(&handles(&c.state.piles.hand).iter().map(|h|h.as_str().parse().unwrap()).collect::<Vec<u64>>()),set(&[3,6,7,8,9,10]));
    assert!(c.state.piles.draw.is_empty());assert_eq!(c.state.player.block,3);
    assert_eq!(c.state.enemies[1].intent_id,ActorIntentId::Full(FullIntentId::from_name("chosen_hex")));
    c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(amount(&c.state.player_full_powers,"hex"),Some(1));
    // Turn 3: source Foresight shuffles the empty draw pile first (addToTop), then owns a real
    // Scry 6 of the reshuffled cards; Confirm0 keeps them all and resumes the normal draw once.
    let s=c.public_full().unwrap().selection.expect("Foresight Scry after its own empty-draw shuffle");
    assert_eq!(s.parent,SelectionParent::Power{turn:3,id:FullPowerId("foresight")});assert_eq!(s.candidates.len(),6);
    let top=s.candidates.iter().take(5).map(|v|card_state::handle(v.instance)).collect::<std::collections::BTreeSet<_>>();
    assert!(c.state.piles.hand.is_empty());c.step_full(&CombatAction::ConfirmSelection).unwrap();
    assert!(c.public_full().unwrap().selection.is_none());assert_eq!(handles(&c.state.piles.hand).into_iter().collect::<std::collections::BTreeSet<_>>(),top);
    // Constructed arrangement: two owned Defends and Weave in hand for the Hex checks.
    let defends=piles::owned_full_cards(&c.state).filter(|c|c.id==DefendPurple).map(|c|c.instance.0).take(2).collect::<Vec<_>>();
    for id in defends.iter().copied().chain([3]){if !in_hand(&c,id){place(&mut c,id,SelectionSourceZone::Hand);}}
    let issued=c.state.card_ids_issued.clone();let before=dazed(&c.state.piles.draw).len();assert_eq!(before,0);
    // Non-Attack under Hex: one actual Dazed into the shuffled draw pile, fresh generated identity.
    play(&mut c,defends[0],None);let d=dazed(&c.state.piles.draw);assert_eq!(d.len(),1);
    assert!(matches!(d[0].origin,CombatCardRef::CombatLocal(_)));assert_eq!(d[0].upgrades,0);
    assert_eq!(c.state.card_ids_issued.difference(&issued).cloned().collect::<Vec<_>>(),vec![card_state::handle(d[0].instance)]);
    // Attack under Hex: no Dazed.
    play(&mut c,3,Some(2));assert_eq!(dazed(&c.state.piles.draw).len(),1);
    // Installed MasterReality does not upgrade the status child.
    c.state.player_full_powers.push(FullPowerEntry{id:FullPowerId("master_reality"),state:PowerState{amount:-1,skip_first:false,monster_source_grace:false}});
    play(&mut c,defends[1],None);let d=dazed(&c.state.piles.draw);assert_eq!(d.len(),2);assert!(d.iter().all(|d|d.upgrades==0));
    // Status cannot be played as a Watcher card.
    let id=d[0].instance.0;place(&mut c,id,SelectionSourceZone::Hand);let before=c.clone();
    assert!(c.step_full(&CombatAction::PlayCard{instance:id,target:None}).is_err());assert_eq!(c,before);
    assert!(engine::validate_runtime(&c.state).is_ok());assert_eq!(census(&c),c.state.card_ids_issued);
}

fn wounds(c:&Combat)->Vec<&LiveCard>{piles::owned_full_cards(&c.state).filter(|c|c.id==CardId::Wound).collect()}
fn intent(c:&Combat,i:usize)->&'static str{match c.state.enemies[i].intent_id{ActorIntentId::Full(id)=>id.as_str(),_=>unreachable!()}}
/// Constructed AI arrangement: replace the selected next move with a listed actual move of the
/// same actor; its real take_turn/memory owner executes it.
fn set_intent(c:&mut Combat,i:usize,name:&'static str){
    let id=ActorIntentId::Full(FullIntentId::from_name(name));let e=&mut c.state.enemies[i];e.intent_id=id.clone();e.selected_move=id.clone();*e.move_history.last_mut().unwrap()=id;
    effects::refresh_enemy_values(&mut c.state).unwrap();
}
fn plays(c:&Combat)->Vec<u64>{c.legal_actions_full().unwrap().into_iter().filter_map(|a|match a{CombatAction::PlayCard{instance,..}=>Some(instance),_=>None}).collect()}
#[test]
fn m23_wound_and_entangle_respect_live_legality(){
    // constructed owner-state diagnostic; not a played route/natural entry/earned resource.
    // Actual Slavers ([Blue, Taskmaster, Red]) with the Wounds deck plus a constructed Strike (6)
    // for Attack legality; constructed hand placements and Red's Entangle move are labelled below.
    use CardId::*;
    let deck=[(Vigilance,0),(Meditate,0),(Perseverance,0),(Prostrate,0),(DefendPurple,0),(StrikePurple,0)];
    let mut c=fight(EncounterId::Slavers,&deck,vec![RelicId::PureWater],vec![None;3]);
    assert_eq!((intent(&c,1),intent(&c,2)),("taskmaster_whip","slaver_stab"));
    for id in 1..=6{if !in_hand(&c,id){place(&mut c,id,SelectionSourceZone::Hand);}}
    play(&mut c,1,None);play(&mut c,4,None);play(&mut c,5,None);
    assert_eq!((c.state.player.block,c.state.player.energy,c.state.character.stance),(17,0,Stance::Calm));
    let blue=match intent(&c,0){"slaver_stab"=>12,"slaver_rake"=>7,other=>panic!("{other}")};
    c.step_full(&CombatAction::EndTurn).unwrap();
    // Calm leaves incoming NORMAL unscaled: blue + whip 7 + stab 13 against 17 block.
    assert_eq!(c.state.player.hp,72-(blue+7+13-17));
    // Scouring Whip: exactly one generated Wound, independent of the HP outcome.
    let w=wounds(&c);assert_eq!(w.len(),1);assert!(matches!(w[0].origin,CombatCardRef::CombatLocal(_)));
    // Inherent Retain grows Perseverance on the same instance (5+2).
    assert!(in_hand(&c,3));assert_eq!(card(&c,3).local.base_block,Some(7));
    // Constructed: hand for turn 2 holds Meditate, Defend and Strike; Red's next move is Entangle.
    for id in [2,5,6]{if !in_hand(&c,id){place(&mut c,id,SelectionSourceZone::Hand);}}
    set_intent(&mut c,2,"slaver_entangle");
    play(&mut c,5,None);
    if c.state.piles.discard.len()>1{play(&mut c,2,None);c.step_full(&CombatAction::SelectCard{instance:5}).unwrap();c.step_full(&CombatAction::ConfirmSelection).unwrap();}else{play(&mut c,2,None);}
    // Meditate ended the turn: temporary Retain kept Defend 5 once, Perseverance grew 7+2.
    assert_eq!(c.state.turn,3);assert!(in_hand(&c,5));assert!(!card(&c,5).local.retain);assert_eq!(card(&c,3).local.base_block,Some(9));
    assert_eq!(amount(&c.state.player_full_powers,"entangled"),Some(1));
    assert!(matches!(c.state.enemies[2].full_memory,Some(enemies::EnemyMemory::SlaverRed{used_entangle:true})));
    let w=wounds(&c);assert_eq!(w.len(),2);assert_ne!(w[0].instance,w[1].instance);assert!(w.iter().all(|w|matches!(w.origin,CombatCardRef::CombatLocal(_))));
    // Entangled: no Attack candidate or play; Skills and retained cards remain legal; statuses never.
    let wound=w[0].instance.0;for id in [6,wound]{if !in_hand(&c,id){place(&mut c,id,SelectionSourceZone::Hand);}}
    let legal=plays(&c);assert!(!legal.contains(&6)&&!legal.contains(&wound));assert!(legal.contains(&5)&&legal.contains(&3));
    let before=c.clone();for id in [6,wound]{assert!(c.step_full(&CombatAction::PlayCard{instance:id,target:if id==6{Some(TargetId(1))}else{None}}).is_err());assert_eq!(c,before);}
    // Upgrade keeps past growth (9+2) and later growth uses the upgraded +3.
    cards::ironclad::rules::upgrade(c.state.piles.hand.iter_mut().find(|o|o.handle()==handle(3)).unwrap().full_mut().unwrap()).unwrap();assert_eq!(card(&c,3).local.base_block,Some(11));
    c.step_full(&CombatAction::EndTurn).unwrap();
    assert_eq!(amount(&c.state.player_full_powers,"entangled"),None);assert_eq!(card(&c,3).local.base_block,Some(14));
    assert!(engine::validate_runtime(&c.state).is_ok());

    // Separate actual BookOfStabbing: opening Multi with count 2 (6 each) and Painful Stabs.
    let mut c=fight(EncounterId::BookOfStabbing,&deck[..5],vec![RelicId::PureWater],vec![None;3]);
    assert_eq!(intent(&c,0),"book_multi");assert_eq!(amount(&c.state.enemies[0].full_powers,"painful_stabs"),Some(1));
    for id in [1,4,5]{play(&mut c,id,None);}assert_eq!(c.state.player.block,17);
    c.step_full(&CombatAction::EndTurn).unwrap();
    // Fully blocked hits lose no HP, so no reaction Wound is fabricated.
    assert_eq!(c.state.player.hp,72);assert!(wounds(&c).is_empty());
    let (hits,each)=match intent(&c,0){"book_multi"=>(3,6),"book_big"=>(1,21),o=>panic!("{o}")};
    c.step_full(&CombatAction::EndTurn).unwrap();
    // Each unblocked HP-losing hit queues one source-owned Wound to discard.
    assert_eq!(c.state.player.hp,72-hits*each);let w=wounds(&c);assert_eq!(w.len(),hits as usize);
    assert!(w.iter().all(|w|matches!(w.origin,CombatCardRef::CombatLocal(_))&&w.upgrades==0));
    // Constructed low HP (7, no block): the real Book turn kills the player; remaining stabs and
    // queued reaction Wounds are cancelled by the shared Death terminal, never delivered after it.
    let n=w.len();c.state.player.hp=7;c.state.player.block=0;
    assert!(matches!(intent(&c,0),"book_multi"|"book_big"),"every A0 Book move deals at least 6 per hit");
    c.step_full(&CombatAction::EndTurn).unwrap();
    assert_eq!(c.state.outcome,Some(Outcome::Death));assert_eq!(c.state.player.hp,0);assert!(c.state.pending.is_empty());
    assert_eq!(wounds(&c).len(),n,"no reaction Wound is delivered after the player's Death");
    assert!(c.exit().is_ok());
}

fn orbs(c:&Combat)->Vec<usize>{c.state.enemies.iter().enumerate().filter(|(_,e)|e.enemy_id==ActorEnemyId::Full(crate::content::EnemyId::BronzeOrb)).map(|(i,_)|i).collect()}
fn drawn(c:&Combat,id:u64)->usize{c.state.hook_events.iter().filter(|e|e.hook==Hook::CardDrawn&&e.source==Source::Card(handle(id))).count()}
/// Constructed owner issue of a listed permanent filler into hand (accepted m21_watcher_review `add`).
fn add_hand(c:&mut Combat,id:u64,kind:CardId){let card=m21_watcher_live_card(kind,CardInstanceId(id),0).unwrap();assert!(c.state.card_ids_issued.insert(handle(id)));c.state.piles.hand.push(model::OwnedCard::Full(card));}
#[test]
fn m23_stasis_preserves_permanent_generated_and_power_custody(){
    // constructed owner-state diagnostic; not a played route/natural entry/earned resource.
    // Actual BronzeAutomaton (opening Spawn of two Orbs). Constructed: Orb next moves are
    // arranged to its own listed Stasis/Beam, hand/discard placements and fillers are listed,
    // and Orb/Automaton deaths use the accepted m5/m6 Fixed-damage death trigger. That
    // trigger is a custody checkpoint only and is never counted as a boss win.
    use CardId::*;
    let mut c=fight(EncounterId::BronzeAutomaton,&[(WindmillStrike,0),(Perseverance,0),(SandsOfTime,0),(Establishment,1),(BattleHymn,1)],vec![RelicId::PureWater],vec![None;3]);
    assert_eq!(intent(&c,0),"automaton_spawn");let original=c.state.card_ids_issued.clone();
    play(&mut c,4,None);play(&mut c,5,None);assert_eq!(c.state.played_powers.len(),2);
    c.step_full(&CombatAction::EndTurn).unwrap();
    let o=orbs(&c);assert_eq!(o.len(),2);
    // Turn 2: Battle Hymn's actual Smite child; retained Sands is now 4-1-1=2.
    let smites=c.state.piles.hand.iter().filter_map(|o|o.full()).filter(|c|c.id==Smite).map(|c|c.instance.0).collect::<Vec<_>>();assert_eq!(smites.len(),1);
    assert_eq!(price(&c,3),Some(2));play(&mut c,3,Some(1));assert!(in_pile(&c.state.piles.discard,3));
    let sands=card(&c,3).clone();assert_eq!((sands.local.mutable_combat_cost,sands.local.base_damage),(Some(2),Some(20)));
    set_intent(&mut c,o[0],"orb_stasis");if intent(&c,o[1])=="orb_stasis"{set_intent(&mut c,o[1],"orb_beam");}
    c.step_full(&CombatAction::EndTurn).unwrap();
    // Empty draw: Stasis takes the only discard candidate, the same permanent instance and price.
    let orb_a=c.state.enemies[o[0]].instance_id.clone();assert_eq!(c.state.stasis.get(&orb_a).and_then(|c|c.full()),Some(&sands));
    assert_eq!(amount(&c.state.enemies[o[0]].full_powers,"stasis"),Some(1));
    let view=c.public_full().unwrap();assert_eq!(view.stasis.len(),1);assert_eq!((view.stasis[0].card.instance.0,view.stasis[0].card.origin.clone()),(3,CombatCardRef::Permanent(CardInstanceId(3))));
    assert!(!plays(&c).contains(&3));assert_eq!(census(&c),c.state.card_ids_issued);
    // Orb A dies: the held Sands returns to hand as the same object, with no draw trigger.
    let draws=drawn(&c,3);let issued=c.state.card_ids_issued.clone();super::m5_elites::hit(&mut c,o[0],999);
    assert!(c.state.stasis.is_empty());assert!(in_hand(&c,3));assert_eq!(card(&c,3),&sands);assert_eq!(drawn(&c,3),draws);assert_eq!(c.state.card_ids_issued,issued);
    assert_eq!(price(&c,3),Some(2));assert!(c.state.outcome.is_none());
    // Generated variant: the Smite child alone in discard is captured by Orb B with its own identity.
    let smite=smites[0];place(&mut c,smite,SelectionSourceZone::Discard);let held=card(&c,smite).clone();
    assert!(matches!(held.origin,CombatCardRef::CombatLocal(_)));assert!(held.local.exhaust_on_finish);
    set_intent(&mut c,o[1],"orb_stasis");c.step_full(&CombatAction::EndTurn).unwrap();
    let orb_b=c.state.enemies[o[1]].instance_id.clone();assert_eq!(c.state.stasis.get(&orb_b).and_then(|c|c.full()),Some(&held));
    // Full hand (10) at the Automaton's scripted cleanup: the return overflows to discard before Victory.
    let mut next=20;while c.state.piles.hand.len()<10{add_hand(&mut c,next,DefendPurple);next+=1;}
    super::m5_elites::hit(&mut c,0,999);
    assert_eq!(c.state.outcome,Some(Outcome::Victory));assert!(c.state.stasis.is_empty());assert_eq!(c.state.piles.hand.len(),10);
    assert!(in_pile(&c.state.piles.discard,smite));assert_eq!(card(&c,smite),&held);
    // Complete census: every original occurs exactly once (including played powers), generated
    // children stay combat-local, and no permanent gain is recorded.
    let exit=c.exit().unwrap();assert!(exit.stasis.is_empty());
    let all=exit.hand.iter().chain(&exit.draw).chain(&exit.discard).chain(&exit.exhaust).chain(&exit.played_powers).chain(exit.in_play.iter()).chain(&exit.inactive_cards).collect::<Vec<_>>();
    let mut permanent=all.iter().filter_map(|c|match c.origin{CombatCardRef::Permanent(id)=>Some(id.0),_=>None}).collect::<Vec<_>>();permanent.sort_unstable();
    let mut expected=original.iter().filter_map(|h|h.as_str().parse::<u64>().ok()).filter(|id|*id<=5).collect::<Vec<_>>();expected.sort_unstable();
    expected.extend(20..next);assert_eq!(permanent,expected);
    assert!(all.iter().filter(|c|c.id==Smite||c.id==Miracle).all(|c|matches!(c.origin,CombatCardRef::CombatLocal(_))));
    assert!(c.state.permanent_gains.is_empty());
}

/// A retained fault is frozen: no view/step/export/resolve can pay, create, move or draw again.
fn frozen(c:&mut Combat){
    assert!(c.fault_record().is_some());let before=c.clone();
    assert!(c.public_full().is_err());assert!(c.legal_actions_full().is_err());
    for a in [CombatAction::EndTurn,CombatAction::ConfirmSelection,CombatAction::UsePotion{slot:0,target:None},CombatAction::ChooseStance{stance:Stance::Calm},CombatAction::PlayCard{instance:1,target:Some(TargetId(1))}]{assert!(c.step_full(&a).is_err());}
    assert!(c.exit().is_err());engine::resolve(&mut c.state);engine::dispatch_one(&mut c.state);assert_eq!(*c,before);
}
fn life(c:&Combat,i:usize)->ActorLife{c.state.enemies[i].life}
#[test]
fn m23_accepted_combat_fault_prefix_is_not_replayed(){
    // constructed owner-state diagnostic; not a played route/natural entry/earned resource.
    // Actual Act2 boss/elite setups. Faults use the shared native effect guard (100000) at a
    // listed checkpoint; deaths use labelled constructed HP or the accepted Fixed-damage death
    // trigger. None supplies a boss win or a natural Death witness.
    use CardId::*;
    let deck=[(StrikePurple,0),(DefendPurple,0),(Eruption,0),(Vigilance,0),(DefendPurple,0)];
    // (a) HolyWater startup: three base Miracles before the normal draw; then a fault after the
    // first queued Miracle child keeps that child, its issued ID and the exact next cursor.
    let c=fight(EncounterId::Champ,&deck,vec![RelicId::HolyWater],vec![Some(PotionId::StancePotion),None,None]);
    let m=c.state.piles.hand.iter().filter_map(|o|o.full()).filter(|c|c.id==Miracle).collect::<Vec<_>>();
    assert_eq!(m.len(),3);assert!(m.iter().all(|m|m.upgrades==0&&matches!(m.origin,CombatCardRef::CombatLocal(_))));assert_eq!(c.state.piles.hand.len(),8);
    let mut f=c.clone();let issued=f.state.card_ids_issued.clone();let kind=cards::watcher::create_action(&f.state,Miracle,3).unwrap();
    f.state.resolved_effects=99998;f.state.pending.push_back(Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind});
    assert_eq!(engine::resolve(&mut f.state).status,TransitionStatus::Fault);
    assert_eq!(f.state.card_ids_issued.difference(&issued).count(),1);
    assert!(matches!(f.state.current.as_ref().map(|e|&e.kind),Some(EffectKind::Watcher(WatcherEffect::Create{count:3,next:1,..}))));
    // Detached inventory stays with the faulted combat, unchanged.
    assert_eq!(f.item_context().unwrap().relics(),&[RelicId::HolyWater]);assert_eq!(f.item_context().unwrap().potions()[0],Some(PotionId::StancePotion));
    frozen(&mut f);
    // (b) Paid StancePotion: the slot is consumed at use; a fault while resolving the choice keeps
    // it consumed and the choice cannot be re-entered, re-paid or re-chosen.
    let mut p=c.clone();p.step_full(&CombatAction::UsePotion{slot:0,target:None}).unwrap();
    assert_eq!(p.item_context().unwrap().potions()[0],None);assert_eq!(p.public_full().unwrap().stance_choice.unwrap().options,vec![Stance::Wrath,Stance::Calm]);
    let before=p.clone();for bad in [CombatAction::UsePotion{slot:0,target:None},CombatAction::ChooseStance{stance:Stance::Divinity},CombatAction::EndTurn]{assert!(p.step_full(&bad).is_err());assert_eq!(p,before);}
    p.step_full(&CombatAction::ChooseStance{stance:Stance::Calm}).unwrap();assert_eq!(p.state.character.stance,Stance::Calm);
    let before=p.clone();assert!(p.step_full(&CombatAction::ChooseStance{stance:Stance::Wrath}).is_err());assert_eq!(p,before);
    // A later shared-resolver fault (accepted M21 guard pattern on a queued Calm->Wrath request)
    // keeps the paid slot empty and the first committed stance change.
    p.state.resolved_effects=99998;p.state.pending.push_back(Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::Character(CharacterEffect::Stance(Stance::Wrath))});
    assert_eq!(engine::resolve(&mut p.state).status,TransitionStatus::Fault);
    assert_eq!(p.item_context().unwrap().potions()[0],None);assert_eq!(p.state.character.stance,Stance::Wrath);frozen(&mut p);
    // (c) Real Champ phase latch: constructed HP 209 < 420/2; the actual next roll is Anger, whose
    // real turn adds Strength 6, and the angered roll then forces Execute.
    let mut h=c.clone();h.state.enemies[0].hp=209;
    h.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(intent(&h,0),"champ_anger");
    assert!(matches!(h.state.enemies[0].full_memory,Some(enemies::EnemyMemory::Champ{angered:true,..})));
    let strength=amount(&h.state.enemies[0].full_powers,"strength").unwrap_or(0);h.step_full(&CombatAction::EndTurn).unwrap();
    assert_eq!(amount(&h.state.enemies[0].full_powers,"strength"),Some(strength+6));assert_eq!(intent(&h,0),"champ_execute");
    // (d) Player Death: constructed 1 HP and the Champ's own Slash; shared Death cancels later work.
    let mut d=c.clone();d.state.player.hp=1;set_intent(&mut d,0,"champ_slash");d.step_full(&CombatAction::EndTurn).unwrap();
    assert_eq!(d.state.outcome,Some(Outcome::Death));assert!(d.state.pending.is_empty());let exit=d.exit().unwrap();assert_eq!((exit.outcome,exit.hp),(Outcome::Death,0));
    let before=d.clone();assert!(d.step_full(&CombatAction::EndTurn).is_err());assert_eq!(d,before);
    // (e) Gremlin Leader death: real Escape of the remaining live minions, then Victory once.
    let mut g=fight(EncounterId::GremlinLeader,&deck,vec![RelicId::PureWater],vec![None;3]);
    let leader=g.state.enemies.iter().position(|e|e.enemy_id==ActorEnemyId::Full(crate::content::EnemyId::GremlinLeader)).unwrap();
    let minions=(0..g.state.enemies.len()).filter(|i|*i!=leader&&g.state.enemies[*i].hp>0).collect::<Vec<_>>();assert_eq!(minions.len(),2);
    super::m5_elites::hit(&mut g,leader,999);
    assert_eq!(g.state.outcome,Some(Outcome::Victory));for i in &minions{assert_eq!(life(&g,*i),ActorLife::Escaped);assert!(g.state.enemies[*i].hp>0);}
    assert!(g.state.pending.is_empty());assert_eq!(g.exit().unwrap().outcome,Outcome::Victory);
    // (f) Collector: actual opening Spawn of two TorchHeads, then Collector death runs the
    // scripted minion cleanup before the single Victory.
    let mut k=fight(EncounterId::Collector,&deck,vec![RelicId::PureWater],vec![None;3]);assert_eq!(intent(&k,0),"collector_spawn");
    k.step_full(&CombatAction::EndTurn).unwrap();
    let heads=(0..k.state.enemies.len()).filter(|i|k.state.enemies[*i].enemy_id==ActorEnemyId::Full(crate::content::EnemyId::TorchHead)).collect::<Vec<_>>();assert_eq!(heads.len(),2);
    let collector=k.state.enemies.iter().position(|e|e.enemy_id==ActorEnemyId::Full(crate::content::EnemyId::Collector)).unwrap();
    super::m5_elites::hit(&mut k,collector,999);
    assert_eq!(k.state.outcome,Some(Outcome::Victory));for i in heads{assert_eq!(k.state.enemies[i].hp,0);}assert!(k.state.pending.is_empty());
    // (g) Stasis release fault: the held card is returned at most once and never duplicated.
    let mut s=fight(EncounterId::BronzeAutomaton,&deck,vec![RelicId::PureWater],vec![None;3]);s.step_full(&CombatAction::EndTurn).unwrap();
    let o=orbs(&s);place(&mut s,1,SelectionSourceZone::Draw);set_intent(&mut s,o[0],"orb_stasis");if intent(&s,o[1])=="orb_stasis"{set_intent(&mut s,o[1],"orb_beam");}
    s.step_full(&CombatAction::EndTurn).unwrap();let orb=s.state.enemies[o[0]].instance_id.clone();assert!(s.state.stasis.contains_key(&orb));
    // Accepted m21_watcher_review cursor pattern: dispatch the real Orb death up to its queued
    // ReleaseStasis, commit that release, then fault the next shared effect.
    let issued=s.state.card_ids_issued.clone();let owner=Actor::Enemy(orb.clone());
    s.state.pending.push_front(Effect{source:Source::Actor(Actor::Player),target:Some(owner),kind:EffectKind::Damage(DamagePayload::Fixed(999))});
    s.state.pending.push_back(Effect{source:Source::None,target:None,kind:EffectKind::Decision});
    for _ in 0..40{if matches!(s.state.pending.front().map(|e|&e.kind),Some(EffectKind::ReleaseStasis{..})){break}engine::dispatch_one(&mut s.state);assert!(s.fault_record().is_none(),"{:?}",s.fault_record());}
    assert!(matches!(s.state.pending.front().map(|e|&e.kind),Some(EffectKind::ReleaseStasis{orb:o}) if *o==orb));
    engine::dispatch_one(&mut s.state);assert!(s.state.stasis.is_empty());assert!(in_hand(&s,1));
    assert!(!s.state.pending.is_empty());s.state.resolved_effects=99999;engine::dispatch_one(&mut s.state);
    assert_eq!(s.fault_record().unwrap().kind,FaultKind::EffectGuard);assert_eq!(s.state.card_ids_issued,issued);
    assert_eq!(piles::owned_full_cards(&s.state).filter(|c|c.instance.0==1).count(),1);assert!(in_hand(&s,1)&&s.state.stasis.is_empty());frozen(&mut s);
}
