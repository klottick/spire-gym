//! M24 Watcher x Act3 composition diagnostics (design §9 rows W1-W14, W2b) on the
//! accepted shared owners. Every combat enters an actual A0 Act3 setup through the
//! accepted `m8_shapes::entry`/`single` fixtures (enemies::setup, seed 41, 300 HP) with
//! the player changed to Watcher, M21 cards and the existing M22 Watcher item context;
//! actions go through `step_full`. Listed pile, counter, intent and HP arrangements are
//! constructed owner-state diagnostics, each citing the accepted test that uses the same
//! helper for the same purpose. Expected values are literal source facts
//! (data/enemies/m8-static-findings.md "findings", docs/research/
//! 2026-10-19-act1-content-watcher.md "research"), never the engine's own damage helper.
//! None is a played route, natural entry or boss win.
use super::super::*;
use crate::{content::{CardId,EncounterId,EnemyId,PotionId,RelicId},items::CombatItemContext};
use super::m5_elites::{amount,hit};
use super::m8_shapes::{entry,single};

fn handle(id:u64)->CardHandle{card_state::handle(CardInstanceId(id))}
fn player_power(c:&Combat,id:&str)->i32{c.state.player_full_powers.iter().filter(|p|p.id.0==id).map(|p|p.state.amount).sum()}
fn card(c:&Combat,id:u64)->&LiveCard{piles::owned_full_cards(&c.state).find(|c|c.instance.0==id).unwrap()}
fn in_pile(pile:&[model::OwnedCard],id:u64)->bool{pile.iter().any(|o|o.handle()==handle(id))}
fn count(pile:&[model::OwnedCard],kind:CardId)->usize{pile.iter().filter(|o|o.full().is_some_and(|c|c.id==kind)).count()}
fn find(pile:&[model::OwnedCard],kind:CardId)->u64{pile.iter().find_map(|o|o.full().filter(|c|c.id==kind)).unwrap().instance.0}
fn hooks(c:&Combat,hook:Hook)->usize{c.state.hook_events.iter().filter(|e|e.hook==hook).count()}
/// Total actual-draw notifications (CardDrawn's source is the drawing effect, not the card).
fn draws(c:&Combat)->usize{hooks(c,Hook::CardDrawn)}
fn candidates(c:&Combat)->Vec<u64>{c.public_full().unwrap().selection.expect("open selection").candidates.iter().map(|v|v.instance.0).collect()}
fn intent(c:&Combat,i:usize)->&'static str{c.state.enemies[i].intent_id.as_str()}

/// Accepted M21 Watcher-versus-Act3 entry (m21_watcher_scry.rs:97-104): actual A0 Act3
/// setup, character Watcher, M21 permanent IDs 1..N in listed order, all drawn at entry.
/// `energy`/`draw` are constructed player-setup values (accepted m8_awakened.rs:223 sets
/// max/energy; m19_defect_interactions.rs:9 and m8_elites.rs:46 set draw_per_turn; entry
/// admits draw 1..=10, so every row uses the minimum ordinary start draw of 1).
/// Items use the existing Watcher context `with_m22_profile()` (m23_watcher_act2.rs:25).
fn watcher(mut e:CombatEntry,deck:&[(CardId,u32)],energy:i32,draw:i32,relics:Vec<RelicId>,potions:Vec<Option<PotionId>>)->Combat{
    e.setup.character=crate::content::Character::Watcher;
    e.cards=deck.iter().enumerate().map(|(i,(id,up))|m21_watcher_live_card(*id,CardInstanceId(i as u64+1),*up).unwrap()).collect();
    e.opening_draw=deck.len().min(10);e.setup.player.energy=energy;e.setup.player.max_energy=energy;e.setup.player.draw_per_turn=draw;
    let mut items=Some(CombatItemContext::new(relics,potions).with_m22_profile());
    let c=Combat::enter_with_item_context(e,&mut items).unwrap();assert!(items.is_none());c
}
/// Accepted M21/M19/M8 Time Warp staging (m21_watcher_scry.rs:101-102): counter and intent
/// are set on the actual setup before entry.
fn time_eater(count:i32,move_id:&'static str)->CombatEntry{
    let mut e=entry(EncounterId::TimeEater);
    e.setup.enemies[0].powers.iter_mut().find(|p|p.id.0=="time_warp").unwrap().state.amount=count;
    e.setup.enemies[0].intent=FullIntentId::from_name(move_id);e
}
fn with_intent(mut e:CombatEntry,slot:usize,move_id:&'static str)->CombatEntry{e.setup.enemies[slot].intent=FullIntentId::from_name(move_id);e}
fn play(c:&mut Combat,id:u64,target:Option<u64>){c.step_full(&CombatAction::PlayCard{instance:id,target:target.map(TargetId)}).unwrap_or_else(|e|panic!("play {id}: {e:?} turn{} legal={:?} fault={:?}",c.state.turn,c.legal_actions_full(),c.fault_record()));}
fn end(c:&mut Combat){c.step_full(&CombatAction::EndTurn).unwrap();}
/// Accepted constructed AI arrangement (m19_defect_interactions.rs:14): replace the selected
/// next move with a listed actual move; its real take_turn owner executes it.
fn set_intent(c:&mut Combat,slot:usize,name:&'static str){let id=ActorIntentId::Full(FullIntentId::from_name(name));let enemy=&mut c.state.enemies[slot];enemy.intent_id=id.clone();enemy.selected_move=id.clone();enemy.move_history.push(id);effects::refresh_enemy_values(&mut c.state).unwrap();}
/// Accepted constructed arrangement (m23_watcher_act2.rs:29): move an owned instance between
/// ordinary piles; no issue, draw or hook. `Draw` pushes on top.
fn place(c:&mut Combat,id:u64,zone:SelectionSourceZone){
    let h=handle(id);let take=|p:&mut Vec<model::OwnedCard>|p.iter().position(|o|o.handle()==h).map(|i|p.remove(i));
    let owned=take(&mut c.state.piles.hand).or_else(||take(&mut c.state.piles.draw)).or_else(||take(&mut c.state.piles.discard)).unwrap();
    match zone{SelectionSourceZone::Hand=>c.state.piles.hand.push(owned),SelectionSourceZone::Draw=>c.state.piles.draw.push(owned),SelectionSourceZone::Discard=>c.state.piles.discard.push(owned),_=>unreachable!()}
    piles::refresh_hand_values(&mut c.state);
}
/// Actual Awakened One path to Sludge (accepted m8_awakened.rs:64 / m19:36-38): a supplied
/// phase-one depletion via `hit`, the real Rebirth turn, then the real `awa_sludge` move.
fn sludge(c:&mut Combat){
    hit(c,2,300);assert_eq!(c.state.enemies[2].life,ActorLife::HalfDead);end(c);assert_eq!(c.state.enemies[2].hp,300);
    set_intent(c,2,"awa_sludge");end(c);
    assert_eq!(c.state.piles.draw.first().unwrap().full().unwrap().id,CardId::Void,"Sludge inserts Void at the draw BOTTOM (findings Task7)");
    assert!(c.state.piles.draw.len()>1,"turn-3 draw took a card above the bottom Void");
    // Constructed arrangement: leave only the bottom Void in draw.
    let above=c.state.piles.draw[1..].iter().map(|o|o.full().unwrap().instance.0).collect::<Vec<_>>();
    for id in above{place(c,id,SelectionSourceZone::Discard);}
    assert_eq!(c.state.piles.draw.len(),1);
}

#[test]
fn m24_w01_timewarp_counts_watcher_plays(){
    // findings Task8: Time Warp starts 0, persists across turns, +1 per unsuppressed card use;
    // potions are not card uses. research §6.1: an actual stance change returns discard
    // Flurry to hand (a move, not a play). PureWater Miracle, CarveReality Smite and
    // DeceiveReality Safety are generated cards (research §5.1); Windmill is self-Retain.
    use CardId::*;
    let mut c=watcher(time_eater(0,"time_ripple"),&[(Prostrate,0),(FlurryOfBlows,0),(CarveReality,0),(DeceiveReality,0),(WindmillStrike,0)],3,1,vec![RelicId::PureWater],vec![Some(PotionId::StancePotion),None,None]);
    let tw=|c:&Combat|amount(c,0,"time_warp");assert_eq!(tw(&c),0);
    let miracle=find(&c.state.piles.hand,Miracle);play(&mut c,miracle,None);assert_eq!((tw(&c),c.state.player.energy),(1,4),"generated 0-cost Miracle counts");
    play(&mut c,1,None);assert_eq!(tw(&c),2,"0-cost Prostrate");
    play(&mut c,2,Some(1));assert_eq!(tw(&c),3,"0-cost Flurry");assert!(in_pile(&c.state.piles.discard,2));
    play(&mut c,3,Some(1));assert_eq!(tw(&c),4);let smite=find(&c.state.piles.hand,Smite);
    play(&mut c,4,None);assert_eq!(tw(&c),5);let safety=find(&c.state.piles.hand,Safety);
    play(&mut c,smite,Some(1));assert_eq!(tw(&c),6,"generated Smite");
    play(&mut c,safety,None);assert_eq!(tw(&c),7,"generated Safety");assert_eq!(c.state.player.energy,0);
    c.step_full(&CombatAction::UsePotion{slot:0,target:None}).unwrap();assert_eq!(tw(&c),7,"potion is not a card use");
    c.step_full(&CombatAction::ChooseStance{stance:Stance::Calm}).unwrap();
    assert_eq!(c.state.character.stance,Stance::Calm);assert!(in_pile(&c.state.piles.hand,2),"Flurry returned on the actual stance change");
    assert_eq!(tw(&c),7,"Flurry's return is a move, not a play");
    play(&mut c,2,Some(1));assert_eq!(tw(&c),8);
    end(&mut c);assert_eq!((c.state.turn,tw(&c)),(2,8),"counter persists over the turn");assert!(in_pile(&c.state.piles.hand,5),"Windmill retained");
    play(&mut c,5,Some(1));assert_eq!(tw(&c),9,"retained card counts");
    assert_eq!(amount(&c,0,"strength"),0);assert!(engine::validate_runtime(&c.state).is_ok());
}

#[test]
fn m24_w02_timewarp_twelfth_stance_card(){
    // findings Task8: at 12 the counter resets to 0, every actor gains queued Strength 2 and
    // one normal early end runs; the current card's payload precedes after-use. research §2:
    // Eruption 9 then Wrath; Vigilance Block 8 then Calm. research §4: MentalFortress Block 4
    // per actual stance change. findings Task8: Reverberate 7x3. research §6.1: Wrath doubles
    // received NORMAL. Open (not asserted): Rushdown's queued draw versus the barrier.
    use CardId::*;
    let mut c=watcher(time_eater(9,"time_reverb"),&[(MentalFortress,0),(Rushdown,0),(Eruption,1),(StrikePurple,0),(StrikePurple,0)],3,1,vec![],vec![None;3]);
    play(&mut c,1,None);play(&mut c,2,None);assert_eq!((amount(&c,0,"time_warp"),c.state.turn),(11,1));
    play(&mut c,3,Some(1));
    assert_eq!((c.state.turn,amount(&c,0,"time_warp"),amount(&c,0,"strength")),(2,0,2));
    assert_eq!(hooks(&c,Hook::TurnEnded),1);assert_eq!(hooks(&c,Hook::CardPlayed),3,"no thirteenth card use");
    assert_eq!(c.state.enemies[0].hp,456-9,"Eruption's own 9 resolved before the early end");
    assert_eq!(c.state.character.stance,Stance::Wrath);
    // Reverberate (7+2) x3, each doubled in Wrath = 54, minus MentalFortress Block 4.
    assert_eq!(c.state.player.hp,300-50);
    assert!(c.state.in_play.is_none()&&c.state.pending_time_warp.is_none());assert!(engine::validate_runtime(&c.state).is_ok());

    let mut c=watcher(time_eater(10,"time_reverb"),&[(MentalFortress,0),(Vigilance,0),(StrikePurple,0)],3,1,vec![],vec![None;3]);
    play(&mut c,1,None);play(&mut c,2,None);
    assert_eq!((c.state.turn,amount(&c,0,"time_warp"),amount(&c,0,"strength"),c.state.character.stance),(2,0,2,Stance::Calm));
    assert_eq!(hooks(&c,Hook::TurnEnded),1);
    // Reverberate (7+2) x3 = 27 against Vigilance 8 + MentalFortress 4.
    assert_eq!(c.state.player.hp,300-15);assert!(engine::validate_runtime(&c.state).is_ok());

    // Twelfth Eruption out of Calm: research §6.1 Calm exit queues +2 energy; the stance
    // change still resolves before the early end (findings Task8).
    let mut c=watcher(time_eater(10,"time_ripple"),&[(Vigilance,0),(Eruption,1),(StrikePurple,0)],3,1,vec![],vec![None;3]);
    play(&mut c,1,None);assert_eq!((c.state.character.stance,c.state.player.energy,amount(&c,0,"time_warp")),(Stance::Calm,1,11));
    play(&mut c,2,Some(1));
    assert_eq!((c.state.turn,amount(&c,0,"time_warp"),amount(&c,0,"strength"),c.state.character.stance),(2,0,2,Stance::Wrath));
    assert_eq!(c.state.enemies[0].hp,456-9);assert_eq!(hooks(&c,Hook::TurnEnded),1);assert!(engine::validate_runtime(&c.state).is_ok());
}

#[test]
fn m24_w02b_timewarp_twelfth_card_with_selection(){
    // Review Focus 2. findings Task8: the current selection precedes after-use, then one
    // early end; never a thirteenth input. research §3: Third Eye Block 7, Scry 3.
    // Meditate at 11 is the accepted m21_review_i2_meditate_timewarp11_* case (private
    // fixture, not duplicated here). No finite M21 card opens a stance choice (only
    // StancePotion does, and potions are not card uses), so that variant has no card row.
    use CardId::*;
    let mut c=watcher(time_eater(11,"time_ripple"),&[(ThirdEye,0),(DefendPurple,0),(DefendPurple,0),(StrikePurple,0)],3,1,vec![],vec![None;3]);
    place(&mut c,2,SelectionSourceZone::Draw);place(&mut c,3,SelectionSourceZone::Draw);
    play(&mut c,1,None);
    assert_eq!(c.state.player.block,7);assert_eq!((c.state.turn,amount(&c,0,"time_warp")),(1,11));
    let view=c.public_full().unwrap().selection.unwrap();assert_eq!(view.parent,SelectionParent::Card(handle(1)));
    assert_eq!(candidates(&c),vec![3,2],"top of draw first");
    assert!(c.legal_actions_full().unwrap().iter().all(|a|!matches!(a,CombatAction::PlayCard{..}|CombatAction::EndTurn|CombatAction::UsePotion{..})),"no thirteenth action during the twelfth card's Scry");
    c.step_full(&CombatAction::SelectCard{instance:3}).unwrap();c.step_full(&CombatAction::ConfirmSelection).unwrap();
    assert_eq!((c.state.turn,amount(&c,0,"time_warp"),amount(&c,0,"strength")),(2,0,2));
    assert_eq!(hooks(&c,Hook::TurnEnded),1);assert_eq!(hooks(&c,Hook::CardPlayed),1);
    assert!(in_pile(&c.state.piles.discard,3),"scried card discarded under its parent");assert!(in_pile(&c.state.piles.hand,2)&&c.state.piles.draw.is_empty(),"unselected card stayed on top and is the turn-2 draw");
    assert!(c.state.pending_selection.is_none()&&c.state.in_play.is_none()&&c.state.pending_time_warp.is_none());assert!(engine::validate_runtime(&c.state).is_ok());
}

#[test]
fn m24_w03_early_end_turn_effects_once(){
    // research §4: LikeWater Block 5 in Calm at pre-end-turn; Establishment discounts
    // Retain/selfRetain hand cards by 1; Perseverance +2 Block, Sands -1 cost, Windmill +4
    // damage per actual onRetained. M21 reconciliation: Establishment scans before limbo;
    // unretained Ethereal exhausts. findings Task8: the forced end is the normal end-turn.
    // Constructed: one generated unretained Dazed in hand (accepted m21_watcher_retain.rs:32).
    use CardId::*;
    let mut c=watcher(time_eater(8,"time_reverb"),&[(Vigilance,0),(LikeWater,0),(Establishment,0),(Perseverance,0),(SandsOfTime,0),(WindmillStrike,0)],3,1,vec![RelicId::PureWater],vec![None;3]);
    let dazed=card_state::generated(Dazed,CardInstanceId(50),0).unwrap();c.state.card_ids_issued.insert(card_state::handle(dazed.instance));c.state.piles.hand.push(model::OwnedCard::Full(dazed));
    let miracle=find(&c.state.piles.hand,Miracle);play(&mut c,miracle,None);play(&mut c,1,None);play(&mut c,2,None);
    assert_eq!((amount(&c,0,"time_warp"),c.state.turn,c.state.character.stance),(11,1,Stance::Calm));
    play(&mut c,3,None);
    assert_eq!((c.state.turn,amount(&c,0,"time_warp"),hooks(&c,Hook::TurnEnded)),(2,0,1));
    // Reverberate (7+2) x3 = 27 against Vigilance 8 + one LikeWater 5.
    assert_eq!(c.state.player.hp,300-14);
    for id in [4,5,6]{assert!(in_pile(&c.state.piles.hand,id),"retained physical instance {id}");}
    let p=card(&c,4);assert_eq!((p.local.base_block,p.local.mutable_combat_cost,p.local.turn_cost),(Some(7),Some(0),Some(0)));
    let s=card(&c,5);assert_eq!((s.local.base_damage,s.local.mutable_combat_cost,s.local.turn_cost),(Some(20),Some(2),Some(2)));
    let w=card(&c,6);assert_eq!((w.local.base_damage,w.local.mutable_combat_cost,w.local.turn_cost),(Some(11),Some(1),Some(1)));
    assert!(in_pile(&c.state.piles.exhaust,50));assert_eq!(count(&c.state.piles.exhaust,Dazed),1);
    assert!(engine::validate_runtime(&c.state).is_ok());

    // research §6.1: Divinity entry +3 energy; its start-turn hook queues Neutral. The
    // second Worship (Mantra 5+5) is the twelfth card. Next turn recharges to max only.
    let mut c=watcher(time_eater(9,"time_ripple"),&[(Worship,0),(Worship,0),(StrikePurple,0)],3,1,vec![RelicId::PureWater],vec![None;3]);
    let miracle=find(&c.state.piles.hand,Miracle);play(&mut c,miracle,None);play(&mut c,1,None);assert_eq!(player_power(&c,"mantra"),5);
    play(&mut c,2,None);
    assert_eq!((c.state.turn,amount(&c,0,"time_warp"),hooks(&c,Hook::TurnEnded)),(2,0,1));
    assert_eq!((c.state.character.stance,c.state.player.energy,player_power(&c,"mantra")),(Stance::Neutral,3,0));
    assert!(engine::validate_runtime(&c.state).is_ok());
}

#[test]
fn m24_w04_curiosity_counts_watcher_powers(){
    // findings Task7: Curiosity 1 queues boss Strength per actual POWER use only.
    // Constructed energy 9/10 (accepted m8_awakened.rs:223) so one hand holds every case.
    use CardId::*;
    let powers=[MentalFortress,Rushdown,Foresight,Nirvana,Devotion,Establishment,BattleHymn,LikeWater,MasterReality];
    let mut c=watcher(entry(EncounterId::AwakenedOne),&powers.map(|p|(p,0)),9,1,vec![],vec![None;3]);
    assert_eq!((c.state.enemies[2].hp,amount(&c,2,"curiosity"),amount(&c,2,"strength")),(300,1,0));
    for (i,kind) in powers.iter().enumerate(){play(&mut c,i as u64+1,None);assert_eq!(amount(&c,2,"strength"),i as i32+1,"{kind:?} is a Power use");}
    assert_eq!(c.state.played_powers.len(),9);

    let mut c=watcher(entry(EncounterId::AwakenedOne),&[(DefendPurple,0),(StrikePurple,0),(CarveReality,0),(Vigilance,0),(Eruption,0),(ThirdEye,0)],10,1,vec![RelicId::PureWater],vec![None;3]);
    let miracle=find(&c.state.piles.hand,Miracle);play(&mut c,miracle,None);
    play(&mut c,1,None);play(&mut c,2,Some(3));play(&mut c,3,Some(3));
    let smite=find(&c.state.piles.hand,Smite);play(&mut c,smite,Some(3));
    play(&mut c,4,None);assert_eq!(c.state.character.stance,Stance::Calm);
    play(&mut c,5,Some(3));assert_eq!(c.state.character.stance,Stance::Wrath);
    play(&mut c,6,None);
    assert_eq!(amount(&c,2,"strength"),0,"Skills, Attacks, Smite, Miracle and stance changes are not Power uses");
    assert_eq!(amount(&c,2,"curiosity"),1);assert!(engine::validate_runtime(&c.state).is_ok());
}

#[test]
fn m24_w05_awakened_rebirth_with_stance_damage(){
    // findings Task7: first depletion -> same actor halfDead, cannotLose (no Victory);
    // removes DEBUFFs and Curiosity/Unawakened/Shackled, keeps positive Strength; Rebirth heals
    // to 300; final phase death makes living Cultists escape. research §2/§6.1: Eruption 9
    // then Wrath; Strike 6; Wrath x2, Divinity x3 NORMAL. Supplied depletion via `hit` as
    // accepted m19_defect_interactions.rs:35 and m8_awakened.rs:64. Open: overkill amount.
    use CardId::*;
    let mut c=watcher(entry(EncounterId::AwakenedOne),&[(MentalFortress,0),(Eruption,1),(StrikePurple,0)],3,1,vec![],vec![None;3]);
    for slot in [0,1]{let hp=c.state.enemies[slot].hp;hit(&mut c,slot,hp);}
    hit(&mut c,2,286);let actor=c.state.enemies[2].instance_id.clone();
    play(&mut c,1,None);assert_eq!(amount(&c,2,"strength"),1,"Curiosity");
    play(&mut c,2,Some(3));assert_eq!((c.state.enemies[2].hp,c.state.character.stance),(5,Stance::Wrath));
    play(&mut c,3,Some(3));
    assert_eq!((c.state.enemies[2].hp,c.state.enemies[2].life,c.state.outcome),(0,ActorLife::HalfDead,None),"lethal Wrath 12 into 5 HP: half-dead, no Victory");
    assert_eq!(c.state.enemies[2].instance_id,actor);assert_eq!(intent(&c,2),"awa_rebirth");assert_eq!(hooks(&c,Hook::CombatSettled),0);
    assert_eq!(amount(&c,2,"strength"),1);for id in ["curiosity","unawakened"]{assert_eq!(amount(&c,2,id),0,"{id}");}
    assert!(!c.public_full().unwrap().targets[2].1,"half-dead actor is not targetable");
    end(&mut c);
    assert_eq!((c.state.enemies[2].hp,c.state.enemies[2].max_hp,c.state.enemies[2].life,c.state.outcome),(300,300,ActorLife::Active,None));
    assert_eq!(c.state.enemies[2].instance_id,actor);assert_eq!(amount(&c,2,"strength"),1);assert_eq!(intent(&c,2),"awa_echo");
    assert!(engine::validate_runtime(&c.state).is_ok());

    let mut c=watcher(entry(EncounterId::AwakenedOne),&[(Worship,0),(Worship,0),(MentalFortress,0),(StrikePurple,0)],5,1,vec![],vec![None;3]);
    hit(&mut c,2,290);let actor=c.state.enemies[2].instance_id.clone();
    play(&mut c,1,None);play(&mut c,2,None);assert_eq!((c.state.character.stance,c.state.player.energy),(Stance::Divinity,4));
    play(&mut c,3,None);play(&mut c,4,Some(3));
    assert_eq!((c.state.enemies[2].hp,c.state.enemies[2].life,c.state.outcome),(0,ActorLife::HalfDead,None),"lethal Divinity 18 into 10 HP");
    assert!(c.state.enemies[..2].iter().all(|e|e.life==ActorLife::Active&&e.hp>0));
    end(&mut c);assert_eq!((c.state.enemies[2].hp,c.state.enemies[2].instance_id.clone(),amount(&c,2,"strength")),(300,actor,1));
    let cultists=c.state.enemies[..2].iter().map(|e|e.hp).collect::<Vec<_>>();
    hit(&mut c,2,300);
    assert_eq!(c.state.outcome,Some(Outcome::Victory));assert!(c.state.enemies[..2].iter().all(|e|e.life==ActorLife::Escaped));
    assert_eq!(c.state.enemies[..2].iter().map(|e|e.hp).collect::<Vec<_>>(),cultists);
    assert_eq!((hooks(&c,Hook::HalfDeath),hooks(&c,Hook::Death),hooks(&c,Hook::CombatSettled)),(1,1,1));
}

#[test]
fn m24_w06_void_with_watcher_energy(){
    // findings Task7: Void's actual draw loses 1 energy clamped at 0; reveal/placement does
    // not. research §6.1: Calm exit +2, Divinity entry +3, Calm entry none; VioletLotus +1
    // on Calm exit (accepted m23_watcher_stances_compose_with_actual_act2_damage).
    // research §3: Cut Through Fate attacks 7, Scry 2, then draws 1. Teardrop starts Calm.
    // Open (not asserted): interleave of the Void draw inside queued transitions; only the
    // fully settled final energy is pinned.
    use CardId::*;
    let mut c=watcher(entry(EncounterId::AwakenedOne),&[(Worship,0),(Worship,0),(CutThroughFate,0),(DefendPurple,0)],5,1,vec![RelicId::TeardropLocket,RelicId::VioletLotus],vec![None;3]);
    assert_eq!((c.state.character.stance,c.state.player.energy),(Stance::Calm,5),"Teardrop Calm entry grants no energy");
    sludge(&mut c);let void=find(&c.state.piles.draw,Void);
    assert_eq!((c.state.turn,c.state.character.stance,c.state.player.energy),(3,Stance::Calm,5));
    for id in [1,2,3]{place(&mut c,id,SelectionSourceZone::Hand);}
    play(&mut c,1,None);play(&mut c,2,None);
    assert_eq!((c.state.character.stance,c.state.player.energy),(Stance::Divinity,5-4+2+1+3));
    let mut scried=c.clone();let before=draws(&c);
    play(&mut c,3,Some(3));assert_eq!(candidates(&c),vec![void]);c.step_full(&CombatAction::ConfirmSelection).unwrap();
    assert_eq!(draws(&c),before+1);assert!(in_pile(&c.state.piles.hand,void),"Cut Through Fate's draw 1 drew the kept Void");assert_eq!(c.state.player.energy,7-1-1,"Void draw loses 1");
    place(&mut scried,4,SelectionSourceZone::Draw);
    play(&mut scried,3,Some(3));assert_eq!(candidates(&scried),vec![4,void]);
    scried.step_full(&CombatAction::SelectCard{instance:void}).unwrap();scried.step_full(&CombatAction::ConfirmSelection).unwrap();
    assert_eq!(draws(&scried),before+1);assert!(in_pile(&scried.state.piles.discard,void));assert!(in_pile(&scried.state.piles.hand,4),"the draw 1 took the card above, not the scried Void");
    assert_eq!(scried.state.player.energy,7-1,"Scry-discarded Void has no draw effect");
    assert!(engine::validate_runtime(&c.state).is_ok()&&engine::validate_runtime(&scried.state).is_ok());
}

#[test]
fn m24_w07_artifact_vs_ancient_potion(){
    // M21 profile potency: AncientPotion = 1 Artifact (items/mod.rs initial_potency). findings
    // Task8: Ripple Weak 1 + Vulnerable 1; Task3: Maw Roar Weak 3 + Frail 3. Artifact spends
    // one charge per DEBUFF application. Main binding: which debuff is blocked is open, so
    // only "exactly one applies and Artifact 1 -> 0" is pinned.
    for (encounter,move_id,debuffs,want) in [(EncounterId::TimeEater,"time_ripple",["weak","vulnerable"],1),(EncounterId::Maw,"maw_roar",["weak","frail"],3)]{
        let mut c=watcher(with_intent(entry(encounter),0,move_id),&[(CardId::DefendPurple,0)],3,1,vec![],vec![Some(PotionId::AncientPotion),None,None]);
        c.step_full(&CombatAction::UsePotion{slot:0,target:None}).unwrap();assert_eq!(player_power(&c,"artifact"),1);
        end(&mut c);
        assert_eq!(player_power(&c,"artifact"),0,"{move_id}");
        let applied=debuffs.map(|id|player_power(&c,id));
        assert_eq!(applied.iter().filter(|a|**a!=0).count(),1,"{move_id}: exactly one debuff applies {applied:?}");
        assert_eq!(applied.iter().sum::<i32>(),want,"{move_id}");
    }
}

#[test]
fn m24_w08_deca_dazed_with_meditate_retain(){
    // findings Task9: Deca opens Beam 10x2 then two Dazed into discard. research §4: Meditate
    // returns 1 discard instance with temporary Retain, Calm, then end turn. Cited ruling,
    // not re-decided: M21 reconciliation ("explicitly retained Dazed ... survives that
    // disposition"), m21_watcher_retain_ethereal_survives_and_unretained_exhausts.
    use CardId::*;
    let mut deck=vec![(Meditate,0)];deck.extend([(DefendPurple,0);10]);
    let mut c=watcher(entry(EncounterId::DonuAndDeca),&deck,3,1,vec![],vec![None;3]);
    assert_eq!(c.state.enemies[0].enemy_id,ActorEnemyId::Full(EnemyId::Deca));assert_eq!(intent(&c,0),"deca_beam");
    end(&mut c);assert_eq!((count(&c.state.piles.discard,Dazed),count(&c.state.piles.draw,Dazed),count(&c.state.piles.hand,Dazed)),(2,0,0));
    place(&mut c,1,SelectionSourceZone::Hand);let d=find(&c.state.piles.discard,Dazed);
    play(&mut c,1,None);assert!(candidates(&c).contains(&d));
    c.step_full(&CombatAction::SelectCard{instance:d}).unwrap();c.step_full(&CombatAction::ConfirmSelection).unwrap();
    assert_eq!((c.state.turn,c.state.character.stance),(3,Stance::Calm),"Meditate requested the end of turn 2");
    assert!(in_pile(&c.state.piles.hand,d),"Meditate-retained Dazed survives end-turn Ethereal disposition");
    assert!(!card(&c,d).local.retain,"temporary Retain cleared after restoration");
    assert_eq!(count(&c.state.piles.exhaust,Dazed),0);assert!(engine::validate_runtime(&c.state).is_ok());
}

#[test]
fn m24_w09_slow_after_stance_multiplier(){
    // findings Task5: Giant Head A0 500; Slow starts 0, +1 after each card's queued payload;
    // NORMAL receive x(1+0.1*Slow) after attacker modifiers and stance, one final floor.
    // research §2/§3: Eruption 9, Flurry 4, Strike 6, EmptyFist 9 (-> Neutral), Worship 5
    // Mantra; §6.1 Wrath x2, Divinity x3. No finite M21 Watcher card is multi-hit, so the
    // "not between a card's own hits" clause has no Watcher card row here (M8/M19 cover it).
    use CardId::*;
    let mut c=watcher(entry(EncounterId::GiantHead),&[(Eruption,1),(FlurryOfBlows,0),(StrikePurple,0),(EmptyFist,0),(Worship,0),(Worship,0),(StrikePurple,0)],10,1,vec![],vec![None;3]);
    assert_eq!((c.state.enemies[0].hp,amount(&c,0,"slow")),(500,0));
    let step=|c:&mut Combat,id:u64,target:Option<u64>,hp:i32,slow:i32|{play(c,id,target);assert_eq!((c.state.enemies[0].hp,amount(c,0,"slow")),(hp,slow),"card {id}");};
    step(&mut c,1,Some(1),491,1);            // 9 x1.0
    step(&mut c,2,Some(1),483,2);            // Wrath 8 x1.1 = 8.8
    step(&mut c,3,Some(1),469,3);            // Wrath 12 x1.2 = 14.4
    step(&mut c,4,Some(1),446,4);            // Wrath 18 x1.3 = 23.4, then Neutral
    step(&mut c,5,None,446,5);step(&mut c,6,None,446,6);assert_eq!(c.state.character.stance,Stance::Divinity);
    step(&mut c,7,Some(1),418,7);            // Divinity 18 x1.6 = 28.8
    step(&mut c,2,Some(1),398,8);            // returned Flurry, Divinity 12 x1.7 = 20.4
    assert_eq!(c.state.player.energy,10-1-1-1-2-2+3-1);assert!(engine::validate_runtime(&c.state).is_ok());
}

#[test]
fn m24_w10_intangible_caps_stance_hits(){
    // findings Task5: Nemesis A0 185, Burn move puts 3 Burns into discard, absent-only
    // Intangible 1 after its payload; Nemesis replaces every positive hit by 1. Burn's
    // end-turn damage is THORNS 2 (accepted m5_status Burn test); research §6.1: Wrath
    // multiplies NORMAL only. Open/not exercised: zero-damage Watcher hit (no M21 card).
    use CardId::*;
    let mut deck=vec![(Eruption,1),(FlurryOfBlows,0),(Worship,0),(Worship,0),(StrikePurple,0)];deck.extend([(DefendPurple,0);6]);
    let mut c=watcher(with_intent(entry(EncounterId::Nemesis),0,"nemesis_burn"),&deck,10,1,vec![],vec![None;3]);
    assert_eq!((c.state.enemies[0].hp,amount(&c,0,"intangible")),(185,0));
    end(&mut c);assert_eq!((count(&c.state.piles.discard,Burn),amount(&c,0,"intangible"),c.state.player.hp),(3,1,300));
    for id in 1..=5{place(&mut c,id,SelectionSourceZone::Hand);}let burn=find(&c.state.piles.discard,Burn);place(&mut c,burn,SelectionSourceZone::Hand);
    play(&mut c,1,Some(1));assert_eq!((c.state.enemies[0].hp,c.state.character.stance),(184,Stance::Wrath));
    play(&mut c,2,Some(1));assert_eq!(c.state.enemies[0].hp,183,"Wrath 8 capped to 1");
    let mut burning=c.clone();set_intent(&mut burning,0,"nemesis_burn");end(&mut burning);
    assert_eq!(burning.state.player.hp,298,"Burn THORNS 2 in Wrath is not doubled");
    play(&mut c,3,None);play(&mut c,4,None);assert_eq!(c.state.character.stance,Stance::Divinity);
    play(&mut c,5,Some(1));assert_eq!(c.state.enemies[0].hp,182,"Divinity 18 capped to 1");
    play(&mut c,2,Some(1));assert_eq!(c.state.enemies[0].hp,181,"returned Flurry, Divinity 12 capped to 1");
    assert!(engine::validate_runtime(&c.state).is_ok());
}

#[test]
fn m24_w11_thorns_type_not_doubled_by_wrath(){
    // findings Task1: Thorns retaliation is THORNS per hit (Spiker base 3, accepted
    // m8_shapes::single); Explosive deals fixed 30 THORNS; Task3: Constricted 10 is THORNS
    // at player end. research §6.1: Wrath doubles received NORMAL only. Supplied shape
    // actors via accepted `single`; low player HP for the lethal case as accepted
    // m19_defect_interactions.rs:30. Open: multi-hit mid-card cancellation (no M21 multi-hit).
    use CardId::*;
    let mut c=watcher(single(EnemyId::Spiker),&[(Eruption,1),(FlurryOfBlows,0)],3,1,vec![],vec![None;3]);
    assert_eq!((c.state.enemies[0].hp,amount(&c,0,"thorns")),(42,3));
    play(&mut c,1,Some(1));assert_eq!((c.state.enemies[0].hp,c.state.player.hp,c.state.character.stance),(33,297,Stance::Wrath));
    let mut lethal=c.clone();lethal.state.player.hp=1;
    play(&mut c,2,Some(1));assert_eq!((c.state.enemies[0].hp,c.state.player.hp),(25,294),"Wrath does not double Thorns 3");
    play(&mut lethal,2,Some(1));assert_eq!((lethal.state.outcome,lethal.state.player.hp,lethal.state.enemies[0].hp),(Some(Outcome::Death),0,25));

    // Exploder: accepted m8_exploder_countdown_and_early_kill attacks 9 for two turns, then
    // explodes. In Wrath the NORMAL attacks double, the THORNS 30 explosion does not.
    let mut c=watcher(single(EnemyId::Exploder),&[(Eruption,1)],3,1,vec![],vec![None;3]);
    play(&mut c,1,Some(1));assert_eq!((c.state.enemies[0].hp,c.state.character.stance),(21,Stance::Wrath));
    end(&mut c);assert_eq!(c.state.player.hp,282);end(&mut c);assert_eq!(c.state.player.hp,264);
    end(&mut c);assert_eq!((c.state.player.hp,c.state.outcome),(234,Some(Outcome::Victory)));

    // Spire Growth (actual A0 170): Constrict -> Constricted 10; in Wrath the end-turn 10
    // THORNS is not doubled while Quick 16 NORMAL is (accepted m8_growth_checks_actual_constricted).
    let mut c=watcher(with_intent(entry(EncounterId::SpireGrowth),0,"growth_constrict"),&[(Eruption,1),(DefendPurple,0)],3,1,vec![],vec![None;3]);
    end(&mut c);assert_eq!((player_power(&c,"constricted"),c.state.player.hp),(10,300));
    place(&mut c,1,SelectionSourceZone::Hand);play(&mut c,1,Some(1));assert_eq!(c.state.character.stance,Stance::Wrath);
    set_intent(&mut c,0,"growth_quick");end(&mut c);assert_eq!(c.state.player.hp,300-10-32);
    assert!(engine::validate_runtime(&c.state).is_ok());
}

#[test]
fn m24_w12_writhing_reactive_malleable_implant(){
    // findings Task4: Writhing Mass A0 160; Malleable 4 on exactly NORMAL positive damage
    // below current HP queues current amount as Block at bottom and increments; Reactive
    // queues one RollMove per qualifying hit; Parasite obtain is one permanent gain.
    // Implant through the accepted pub(crate) m19 implant_checkpoint. No M21 multi-hit card.
    use CardId::*;
    let mut c=watcher(entry(EncounterId::WrithingMass),&[(Eruption,1),(FlurryOfBlows,0),(StrikePurple,0)],3,1,vec![],vec![None;3]);
    assert_eq!((c.state.enemies[0].hp,amount(&c,0,"malleable"),c.state.enemies[0].move_history.len()),(160,4,1));
    let e=|c:&Combat|(c.state.enemies[0].hp,c.state.enemies[0].block,amount(c,0,"malleable"),c.state.enemies[0].move_history.len());
    play(&mut c,1,Some(1));assert_eq!(e(&c),(151,4,5,2),"Neutral 9");
    play(&mut c,2,Some(1));assert_eq!(e(&c),(147,5,6,3),"Wrath 8 less Block 4");
    play(&mut c,3,Some(1));assert_eq!(e(&c),(140,6,7,4),"Wrath 12 less Block 5");
    super::m19_defect_interactions::implant_checkpoint(&mut c);end(&mut c);
    assert_eq!(c.permanent_gains().len(),1);assert_eq!((c.permanent_gains()[0].ordinal,c.permanent_gains()[0].id,c.permanent_gains()[0].upgrades),(1,Parasite,0));
    assert!(piles::owned_full_cards(&c.state).all(|c|c.id!=Parasite));assert!(engine::validate_runtime(&c.state).is_ok());
}

#[test]
fn m24_w13_transient_darkling_dagger_targets(){
    use CardId::*;
    // findings Task3: Transient A0 999, Attack 30 +10 per executed action; Shifting queues
    // Strength loss of positive post-Block damage, Shackled restores at its end-turn.
    let mut c=watcher(entry(EncounterId::Transient),&[(Eruption,1),(FlurryOfBlows,0)],3,1,vec![],vec![None;3]);
    play(&mut c,1,Some(1));play(&mut c,2,Some(1));
    assert_eq!((c.state.enemies[0].hp,amount(&c,0,"strength"),amount(&c,0,"shackled")),(999-17,-17,17),"Neutral 9 + Wrath 8");
    end(&mut c);assert_eq!((c.state.player.hp,amount(&c,0,"strength")),(300-26,0),"(30-17) doubled in Wrath");
    // Fading 5: the fifth attack resolves, then actual self-death is a Victory with one record.
    let mut c=watcher(entry(EncounterId::Transient),&[(DefendPurple,0)],3,1,vec![],vec![None;3]);
    for _ in 0..5{end(&mut c);}
    assert_eq!((c.state.player.hp,c.state.outcome),(300-250,Some(Outcome::Victory)));assert_eq!((hooks(&c,Hook::Death),hooks(&c,Hook::CombatSettled)),(1,1));

    // findings Task2: half-dead Darkling is not targetable; Regrowing, then Reincarnate heals
    // floor(maxHP/2) and reapplies Life Link. Supplied depletion via `hit` (m8_darklings.rs:31).
    let mut c=watcher(entry(EncounterId::ThreeDarklings),&[(StrikePurple,0),(Eruption,1)],3,1,vec![],vec![None;3]);
    let max=c.state.enemies[0].max_hp;hit(&mut c,0,max);assert_eq!(c.state.enemies[0].life,ActorLife::HalfDead);
    let targets=|c:&Combat|c.legal_actions_full().unwrap().into_iter().filter_map(|a|match a{CombatAction::PlayCard{instance:1,target}=>target.map(|t|t.0),_=>None}).collect::<Vec<_>>();
    assert_eq!(targets(&c),vec![2,3]);
    play(&mut c,2,Some(2));assert_eq!(c.state.character.stance,Stance::Wrath);
    for _ in 0..2{for slot in [1,2]{set_intent(&mut c,slot,"darkling_harden");}end(&mut c);}
    assert_eq!((c.state.enemies[0].hp,c.state.enemies[0].life,amount(&c,0,"life_link")),(max/2,ActorLife::Active,1));
    assert!(c.public_full().unwrap().targets[0].1);

    // findings Task6: initial [Dagger, Reptomancer, Dagger]; Spawn adds one Dagger (A0);
    // Dagger Wound 9 then NORMAL 25 explosion and self HP loss. Accepted m8 reptomancer test:
    // 282 after turn 1. In Wrath both Dagger NORMAL attacks double.
    let mut c=watcher(entry(EncounterId::Reptomancer),&[(Eruption,1),(StrikePurple,0)],3,1,vec![],vec![None;3]);
    assert_eq!(intent(&c,1),"repto_spawn");end(&mut c);
    assert_eq!((c.state.enemies.len(),c.state.player.hp),(4,282));assert_eq!(c.state.enemies[3].enemy_id,ActorEnemyId::Full(EnemyId::Dagger));
    assert!((20..=25).contains(&c.state.enemies[3].hp));assert_eq!(amount(&c,3,"minion"),1);
    place(&mut c,1,SelectionSourceZone::Hand);place(&mut c,2,SelectionSourceZone::Hand);
    let strike=|c:&Combat|c.legal_actions_full().unwrap().into_iter().filter_map(|a|match a{CombatAction::PlayCard{instance:2,target}=>target.map(|t|t.0),_=>None}).collect::<Vec<_>>();
    assert_eq!(strike(&c),vec![1,2,3,4],"every live actor is a legal target, in actor order");
    play(&mut c,1,Some(2));assert_eq!(c.state.character.stance,Stance::Wrath);
    set_intent(&mut c,1,"repto_spawn");end(&mut c);
    assert_eq!(c.state.player.hp,282-2*50-18,"two Wrath-doubled 25 explosions and the newborn's doubled Wound 9");
    assert_eq!((c.state.enemies[0].hp,c.state.enemies[2].hp),(0,0));assert!(engine::validate_runtime(&c.state).is_ok());
}

#[test]
fn m24_w14_scry_act3_statuses(){
    // research §3: Third Eye Block 7, Scry 3 (top cards); Weave returns on Scry from discard.
    // findings Task1: Orb Walker Laser Burn into draw + one into discard; Repulsor 2 Dazed into
    // draw; Task9 Deca 2 Dazed and Task5 Nemesis 3 Burns into discard; Task7 Sludge Void at
    // draw bottom. Constructed hand/draw placement via accepted `place` (m23_watcher_act2.rs:29).
    use CardId::*;
    let mut c=watcher(with_intent(entry(EncounterId::OrbWalker),0,"orb_laser"),&[(ThirdEye,0),(Weave,0),(StrikePurple,0)],3,1,vec![],vec![None;3]);
    // The Laser Burn enters the empty draw pile and is the single turn-2 start draw.
    end(&mut c);assert_eq!((count(&c.state.piles.hand,Burn),count(&c.state.piles.draw,Burn),count(&c.state.piles.discard,Burn)),(1,0,1));
    let burn=find(&c.state.piles.hand,Burn);
    place(&mut c,burn,SelectionSourceZone::Draw);place(&mut c,1,SelectionSourceZone::Hand);place(&mut c,3,SelectionSourceZone::Draw);assert!(in_pile(&c.state.piles.discard,2));
    let before=draws(&c);play(&mut c,1,None);assert_eq!(candidates(&c),vec![3,burn],"draw-pile Burn revealed; discard Burn not");
    c.step_full(&CombatAction::SelectCard{instance:burn}).unwrap();c.step_full(&CombatAction::ConfirmSelection).unwrap();
    assert_eq!(count(&c.state.piles.discard,Burn),2);assert_eq!(draws(&c),before,"Scry moves the Burn without drawing it");assert!(in_pile(&c.state.piles.hand,2),"Weave returns on Scry");
    assert_eq!(c.state.player.block,7);

    let mut c=watcher(with_intent(single(EnemyId::Repulsor),0,"repulsor_daze"),&[(ThirdEye,0)],3,1,vec![],vec![None;3]);
    // Both Dazed enter the empty draw pile; the turn-2 start draw takes the top one.
    end(&mut c);assert_eq!((count(&c.state.piles.draw,Dazed),count(&c.state.piles.hand,Dazed),count(&c.state.piles.discard,Dazed)),(1,1,0));
    let top=find(&c.state.piles.hand,Dazed);let other=find(&c.state.piles.draw,Dazed);
    place(&mut c,top,SelectionSourceZone::Draw);place(&mut c,1,SelectionSourceZone::Hand);let before=draws(&c);play(&mut c,1,None);assert_eq!(candidates(&c),vec![top,other]);
    c.step_full(&CombatAction::SelectCard{instance:other}).unwrap();c.step_full(&CombatAction::ConfirmSelection).unwrap();
    assert!(in_pile(&c.state.piles.discard,other)&&in_pile(&c.state.piles.draw,top));assert_eq!(draws(&c),before);

    for (e,status,n) in [(entry(EncounterId::DonuAndDeca),Dazed,2),(with_intent(entry(EncounterId::Nemesis),0,"nemesis_burn"),Burn,3)]{
        let mut deck=vec![(ThirdEye,0)];deck.extend([(StrikePurple,0);10]);
        let mut c=watcher(e,&deck,3,1,vec![],vec![None;3]);
        end(&mut c);assert_eq!((count(&c.state.piles.discard,status),count(&c.state.piles.draw,status),count(&c.state.piles.hand,status)),(n,0,0));
        place(&mut c,1,SelectionSourceZone::Hand);place(&mut c,2,SelectionSourceZone::Draw);
        play(&mut c,1,None);assert_eq!(candidates(&c),vec![2],"{status:?} in discard is not revealed");
    }

    let mut c=watcher(entry(EncounterId::AwakenedOne),&[(ThirdEye,0),(StrikePurple,0),(DefendPurple,0),(DefendPurple,0)],3,1,vec![],vec![None;3]);
    sludge(&mut c);let void=find(&c.state.piles.draw,Void);
    place(&mut c,1,SelectionSourceZone::Hand);place(&mut c,2,SelectionSourceZone::Draw);place(&mut c,3,SelectionSourceZone::Draw);
    let mut hidden=c.clone();
    let before=draws(&c);play(&mut c,1,None);assert_eq!(candidates(&c),vec![3,2,void],"draw pile no larger than Scry 3 reveals the bottom Void");
    c.step_full(&CombatAction::SelectCard{instance:void}).unwrap();c.step_full(&CombatAction::ConfirmSelection).unwrap();
    assert!(in_pile(&c.state.piles.discard,void));assert_eq!((draws(&c),c.state.player.energy),(before,2),"Scry-discarded Void is not drawn");
    place(&mut hidden,4,SelectionSourceZone::Draw);play(&mut hidden,1,None);assert_eq!(candidates(&hidden),vec![4,3,2],"bottom Void below Scry 3 stays hidden");
    assert!(engine::validate_runtime(&c.state).is_ok()&&engine::validate_runtime(&hidden.state).is_ok());
}

#[test]
fn m24_w07b_red_mask_consumes_artifact(){
    // Main-authored after H1 (combat-lane hold). M24 Watcher inventory context with Red Mask
    // (design §5): findings Task9 Deca/Donu A0 Artifact 2 each; accepted Red Mask oracle
    // m14_silent_interactions.rs:175,224 — Red Mask's combat-start Weak 1 consumes one of
    // each partner's two Artifact charges and applies no Weak.
    let mut e=entry(EncounterId::DonuAndDeca);e.setup.character=crate::content::Character::Watcher;
    e.cards=vec![m21_watcher_live_card(CardId::DefendPurple,CardInstanceId(1),0).unwrap()];e.opening_draw=1;e.setup.player.draw_per_turn=1;
    let mut items=Some(CombatItemContext::new(vec![RelicId::PureWater,RelicId::RedMask],vec![None;3]).with_m24_profile());
    let c=Combat::enter_with_item_context(e,&mut items).expect("the M24 Watcher context admits Red Mask");assert!(items.is_none());
    assert_eq!(c.state.enemies.len(),2);
    for slot in 0..2{assert_eq!(amount(&c,slot,"artifact"),1,"slot {slot}: one of two Artifact charges consumed");assert_eq!(amount(&c,slot,"weak"),0,"slot {slot}: no Weak applied");}
    // The M22/M23 Watcher context still rejects Red Mask at entry.
    let mut e=entry(EncounterId::DonuAndDeca);e.setup.character=crate::content::Character::Watcher;
    e.cards=vec![m21_watcher_live_card(CardId::DefendPurple,CardInstanceId(1),0).unwrap()];e.opening_draw=1;e.setup.player.draw_per_turn=1;
    let mut items=Some(CombatItemContext::new(vec![RelicId::PureWater,RelicId::RedMask],vec![None;3]).with_m22_profile());
    assert!(Combat::enter_with_item_context(e,&mut items).is_err());
}

#[test]
fn m24_curse_parasite_watcher_view_scry_and_meditate_custody(){
    // Main-authored (final review M3). Global Watcher Curse construction law: a Parasite
    // permanent enters Watcher combat with the shared unplayable, costless fields; its public
    // view carries no card values and an empty Watcher price (combat and turn cost None); Scry and Meditate select it like any card.
    // Run decks build permanents with the shared constructor; Watcher combat entry then applies
    // the profile initializer (and its Curse law) to every card, as in Run::enter_owned.
    let mut e=entry(EncounterId::OrbWalker);e.setup.character=crate::content::Character::Watcher;
    e.cards=vec![permanent_live_card(CardId::Parasite,CardInstanceId(1),0)];
    e.cards.extend([CardId::ThirdEye,CardId::Meditate,CardId::DefendPurple].into_iter().enumerate().map(|(i,id)|m21_watcher_live_card(id,CardInstanceId(i as u64+2),0).unwrap()));
    e.opening_draw=4;e.setup.player.energy=3;e.setup.player.max_energy=3;e.setup.player.draw_per_turn=1;
    let mut items=Some(CombatItemContext::new(vec![RelicId::PureWater],vec![None;3]).with_m22_profile());
    let mut c=Combat::enter_with_item_context(e,&mut items).expect("a Curse permanent enters Watcher combat");
    let parasite=1;for id in [1,2,3,4]{place(&mut c,id,SelectionSourceZone::Hand);}
    let view=c.public_full().unwrap().hand.into_iter().find(|v|v.instance.0==parasite).expect("Parasite in hand");
    assert_eq!(view.effective_cost,None,"no price");assert!(view.values.is_none(),"no card values");
    assert_eq!(view.watcher.map(|w|(w.combat_cost,w.turn_cost)),Some((None,None)),"Watcher view carries no combat or turn price");
    assert!(!c.legal_actions_full().unwrap().iter().any(|a|matches!(a,CombatAction::PlayCard{instance,..} if *instance==parasite)),"never a legal play");
    // Scry: the Parasite alone on the draw pile is revealed, selected and discarded (not played or drawn).
    place(&mut c,parasite,SelectionSourceZone::Draw);let before=draws(&c);
    play(&mut c,2,None);assert_eq!(candidates(&c),vec![parasite]);
    c.step_full(&CombatAction::SelectCard{instance:parasite}).unwrap();c.step_full(&CombatAction::ConfirmSelection).unwrap();
    assert!(in_pile(&c.state.piles.discard,parasite),"Scry discarded the Parasite");assert_eq!(draws(&c),before,"Scry is not a draw");
    // Meditate: select the Parasite from the discard; it returns to hand with Retain and survives the end of turn.
    play(&mut c,3,None);assert!(candidates(&c).contains(&parasite));
    c.step_full(&CombatAction::SelectCard{instance:parasite}).unwrap();c.step_full(&CombatAction::ConfirmSelection).unwrap();
    assert!(in_pile(&c.state.piles.hand,parasite),"Meditate returned the Parasite to hand");
    assert_eq!(c.state.turn,2,"Meditate ends the turn");assert!(in_pile(&c.state.piles.hand,parasite),"Meditate Retain kept it in hand across that end of turn");
    let view=c.public_full().unwrap().hand.into_iter().find(|v|v.instance.0==parasite).unwrap();assert_eq!(view.effective_cost,None);assert!(view.values.is_none());
    assert!(engine::validate_runtime(&c.state).is_ok(),"full census keeps the Parasite once");
}
