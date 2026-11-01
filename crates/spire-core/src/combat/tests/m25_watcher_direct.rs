//! M25 direct and stance cards (plan Task C2) on the accepted shared owners.
//! Every combat enters an actual setup (`enemies::setup` through the accepted
//! `m8_shapes::entry` fixture, seed 41, player 300 HP, or the same call at Act 2 for
//! Shelled Parasite) with the player changed to Watcher, permanents built by
//! `m25_watcher_live_card` and the complete-Watcher item context `with_m25_profile()`;
//! plays go through `step_full` only. Listed pile, stance, power, HP and intent
//! arrangements are constructed owner-state diagnostics citing the accepted test that
//! uses the same helper for the same purpose. Expected values are literals from the
//! design §5.1 table and the cited source rows ("research" =
//! docs/research/2026-10-19-act1-content-watcher.md, "annotations" =
//! data/cards/research-annotations.json, "enemies" = data/enemies/enemies.json), never
//! the engine's own damage or Block helper.
use super::super::*;
use crate::{content::{CardId,EncounterId},items::CombatItemContext};
use super::m5_elites::{amount,hit};
use super::m8_shapes::entry;
use CardId::*;

fn handle(id:u64)->CardHandle{card_state::handle(CardInstanceId(id))}
fn in_pile(pile:&[model::OwnedCard],id:u64)->bool{pile.iter().any(|o|o.handle()==handle(id))}
fn power(id:&'static str,n:i32)->FullPowerEntry{FullPowerEntry{id:FullPowerId(id),state:PowerState{amount:n,skip_first:false,monster_source_grace:false}}}
fn damage_hooks(c:&Combat,slot:usize)->usize{
    let owner=Actor::Enemy(c.state.enemies[slot].instance_id.clone());
    c.state.hook_events.iter().filter(|e|e.hook==Hook::Damage&&e.target.as_ref()==Some(&owner)).count()
}

/// Accepted Watcher entry (m24_watcher_act3.rs `watcher`), with the complete-profile
/// constructor and context: character Watcher, permanent IDs 1..N in listed order, all
/// drawn at entry up to 10, constructed energy 9 (m8_awakened.rs:223 sets max/energy).
fn watcher(mut e:CombatEntry,deck:&[(CardId,u32)])->Combat{
    e.setup.character=crate::content::Character::Watcher;
    e.cards=deck.iter().enumerate().map(|(i,(id,up))|m25_watcher_live_card(*id,CardInstanceId(i as u64+1),*up).unwrap()).collect();
    e.opening_draw=deck.len().min(10);e.setup.player.energy=9;e.setup.player.max_energy=9;e.setup.player.draw_per_turn=1;
    let mut items=Some(CombatItemContext::new(vec![],vec![None;3]).with_m25_profile());
    let c=Combat::enter_with_item_context(e,&mut items).unwrap();assert!(items.is_none());c
}
/// The accepted ordinary-encounter Act 2 setup call (encounter_ordinary.rs:4) with the
/// `m8_shapes::entry` player and seed; used only for Shelled Parasite's intents.
fn act2_entry(encounter:EncounterId)->CombatEntry{
    enemies::setup(encounter,2,0,&mut crate::run::RunRng::new([41;32],rng::Partition::Test,0),enemies::SetupContext{
        player:PlayerState{hp:300,max_hp:300,block:0,energy:3,max_energy:3,draw_per_turn:1,powers:vec![]},
        player_powers:vec![],character:crate::content::Character::Ironclad,
        cards:vec![permanent_live_card(Defend,CardInstanceId(1),0)],opening_draw:1,encounter_visit:0}).unwrap()
}
fn play(c:&mut Combat,id:u64,target:Option<u64>){c.step_full(&CombatAction::PlayCard{instance:id,target:target.map(TargetId)}).unwrap_or_else(|e|panic!("play {id}: {e:?} legal={:?} fault={:?}",c.legal_actions_full(),c.fault_record()));}
/// Accepted constructed AI arrangement (m24_watcher_act3.rs `set_intent`, m19_defect_interactions.rs:14).
fn set_intent(c:&mut Combat,slot:usize,name:&'static str){let id=ActorIntentId::Full(FullIntentId::from_name(name));let enemy=&mut c.state.enemies[slot];enemy.intent_id=id.clone();enemy.selected_move=id.clone();enemy.move_history.push(id);effects::refresh_enemy_values(&mut c.state).unwrap();}
/// Accepted constructed arrangement (m24_watcher_act3.rs `place`, m23_watcher_act2.rs:29).
fn place(c:&mut Combat,id:u64,zone:SelectionSourceZone){
    let h=handle(id);let take=|p:&mut Vec<model::OwnedCard>|p.iter().position(|o|o.handle()==h).map(|i|p.remove(i));
    let owned=take(&mut c.state.piles.hand).or_else(||take(&mut c.state.piles.draw)).or_else(||take(&mut c.state.piles.discard)).unwrap();
    match zone{SelectionSourceZone::Hand=>c.state.piles.hand.push(owned),SelectionSourceZone::Draw=>c.state.piles.draw.push(owned),SelectionSourceZone::Discard=>c.state.piles.discard.push(owned),_=>unreachable!()}
    piles::refresh_hand_values(&mut c.state);
}
/// Accepted constructed stance arrangement (full_cards.rs:1309 sets Wrath, :1292 Calm).
fn stance(c:&mut Combat,s:Stance){c.state.character.stance=s;}
fn hp(c:&Combat)->Vec<i32>{c.state.enemies.iter().map(|e|e.hp).collect()}
fn living(c:&Combat)->usize{c.state.enemies.iter().filter(|e|e.life==ActorLife::Active&&e.hp>0).count()}
fn face(up:u32,base:i32,upgraded:i32)->i32{if up==0{base}else{upgraded}}
fn fillers(n:usize)->Vec<(CardId,u32)>{vec![(DefendPurple,0);n]}

/// Design §5.1 literals: (card, base cost, upgraded cost, targeted, Retain, Exhaust).
/// Targets: research rows (A; enemy / S; self / A; all) at lines 74-152.
const FACES:&[(CardId,i32,i32,bool,bool,bool)]=&[
    (BowlingBash,1,1,true,false,false),(Consecrate,0,0,false,false,false),(Crescendo,1,0,false,true,true),
    (EmptyBody,1,1,false,false,false),(FlyingSleeves,1,1,true,true,false),(Halt,0,0,false,false,false),
    (Protect,2,2,false,true,false),(Tranquility,1,0,false,true,true),(EmptyMind,1,1,false,false,false),
    (FearNoEvil,1,1,true,false,false),(Indignation,1,1,false,false,false),(InnerPeace,1,1,false,false,false),
    (WheelKick,2,2,true,false,false),(Scrawl,1,0,false,false,true),(SpiritShield,2,2,false,false,false),
];

#[test]
fn m25_direct_base_plus_values_and_flags(){
    // Faces: cost paid, legal target form, Retain flag and finish zone (Exhaust), both faces.
    for &(id,base,upgraded,targeted,retain,exhaust) in FACES {
        for up in [0,1] {
            let mut deck=vec![(id,up)];deck.extend(fillers(5));
            let mut c=watcher(entry(EncounterId::OrbWalker),&deck);
            place(&mut c,5,SelectionSourceZone::Draw);place(&mut c,6,SelectionSourceZone::Draw);
            let target=targeted.then_some(1);
            let legal=c.legal_actions_full().unwrap();
            assert!(legal.contains(&CombatAction::PlayCard{instance:1,target:target.map(TargetId)}),"{id:?}+{up} legal in its target form");
            assert!(!legal.contains(&CombatAction::PlayCard{instance:1,target:(!targeted).then_some(TargetId(1))}),"{id:?}+{up} wrong target form is illegal");
            assert!(c.step_full(&CombatAction::PlayCard{instance:1,target:(!targeted).then_some(TargetId(1))}).is_err(),"{id:?}+{up} rejects the wrong target form");
            let card=piles::owned_full_cards(&c.state).find(|c|c.instance.0==1).unwrap();
            assert_eq!(card.local.self_retain,retain,"{id:?}+{up} Retain");
            play(&mut c,1,target);
            assert_eq!(c.state.player.energy,9-face(up,base,upgraded),"{id:?}+{up} cost");
            assert_eq!((in_pile(&c.state.piles.exhaust,1),in_pile(&c.state.piles.discard,1)),(exhaust,!exhaust),"{id:?}+{up} finish zone");
        }
    }

    for up in [0,1] {
        // BowlingBash 7/10 once per enemy not dead/escaped at use, all on the selected
        // target as separate hits (research:74; annotations:24-28; W-A17 research:613).
        // Half-dead Darklings are not counted (research §8.2 alive-only exclusions).
        let per=face(up,7,10);
        for downed in 0..3usize {
            let mut c=watcher(entry(EncounterId::ThreeDarklings),&[(BowlingBash,up)]);
            for slot in 0..downed { let n=c.state.enemies[slot].hp;hit(&mut c,slot,n); }
            assert_eq!(living(&c),3-downed);
            let t=2;let before=hp(&c);let hooks=damage_hooks(&c,t);
            play(&mut c,1,Some(t as u64+1));
            assert_eq!(before[t]-c.state.enemies[t].hp,per*(3-downed) as i32,"BowlingBash+{up} with {} living",3-downed);
            assert_eq!(damage_hooks(&c,t)-hooks,3-downed,"one separate hit per counted enemy");
            for slot in 0..2 {assert_eq!(c.state.enemies[slot].hp,before[slot],"only the target is hit");}
        }

        // FlyingSleeves 4/6 twice as two separate hits (research:83).
        let mut c=watcher(entry(EncounterId::OrbWalker),&[(FlyingSleeves,up)]);
        let (before,hooks)=(c.state.enemies[0].hp,damage_hooks(&c,0));
        play(&mut c,1,Some(1));
        assert_eq!((before-c.state.enemies[0].hp,damage_hooks(&c,0)-hooks),(2*face(up,4,6),2));

        // Halt 3/4; in Wrath +9/14, each component modified and floored separately, one gain
        // (research:85, :462-467; annotations:65-69; W-A05 research:601).
        for (wrath,powers,expected) in [
            (false,vec![],face(up,3,4)),
            (true,vec![],face(up,12,18)),
            (true,vec![power("dexterity",2)],face(up,(3+2)+(9+2),(4+2)+(14+2))),
            (true,vec![power("frail",1)],face(up,2+6,3+10)), // floor(3*.75)+floor(9*.75), floor(4*.75)+floor(14*.75)
        ] {
            let mut c=watcher(entry(EncounterId::OrbWalker),&[(Halt,up)]);
            if wrath{stance(&mut c,Stance::Wrath);}
            // Constructed power arrangement (m20_defect_interactions.rs:36); Juggernaut 5 deals
            // one 5-damage hit per positive Block gain, counting the gains.
            c.state.player_full_powers.extend(powers);c.state.player_full_powers.push(power("juggernaut",5));
            let enemy=c.state.enemies[0].hp;
            play(&mut c,1,None);
            assert_eq!(c.state.player.block,expected,"Halt+{up} wrath={wrath}");
            assert_eq!(enemy-c.state.enemies[0].hp,5,"Halt+{up} requests exactly one Block gain");
        }

        // Protect 12/16 Block (research:89).
        let mut c=watcher(entry(EncounterId::OrbWalker),&[(Protect,up)]);
        play(&mut c,1,None);assert_eq!(c.state.player.block,face(up,12,16));

        // Crescendo requests Wrath; Tranquility requests Calm (research:76, :92).
        let mut c=watcher(entry(EncounterId::OrbWalker),&[(Crescendo,up),(Tranquility,up)]);
        play(&mut c,1,None);assert_eq!(c.state.character.stance,Stance::Wrath);
        play(&mut c,2,None);assert_eq!(c.state.character.stance,Stance::Calm);

        // FearNoEvil 8/11; requests Calm iff the action-time intent is ATTACK, ATTACK_BUFF,
        // ATTACK_DEBUFF or ATTACK_DEFEND, the hit queued ahead of that request (research:105).
        // Disclosed project decision (contract Binding 9): FullIntentId carries no category,
        // so m25_direct classifies the current intent as an attack category exactly when the
        // engine holds a damage entry with base >= 0 for it (the intent-damage data Spot
        // Weakness and Go for the Eyes already read). Corroboration, enemies wiki_intent
        // tokens (display tokens, not the game enum): Orb Walker claw "Attack4", laser
        // "DebuffAttack3"; Writhing Mass block_attack "DefendAttack4", parasite
        // "DebuffStrong"; Shelled Parasite life_suck "BuffAttack3", stunned "Stunned";
        // Darkling harden "Defend". Across all 196 enemies-dataset moves, damage presence and
        // an "Attack" token agree except Exploder explode ("Unknown"), which the engine keeps
        // without a damage entry.
        for (e,intent,calm) in [
            (entry(EncounterId::OrbWalker),"orb_claw",true),(entry(EncounterId::OrbWalker),"orb_laser",true),
            (entry(EncounterId::WrithingMass),"writhing_block",true),(act2_entry(EncounterId::ShelledParasite),"shelled_parasite_life_suck",true),
            (entry(EncounterId::WrithingMass),"writhing_parasite",false),(act2_entry(EncounterId::ShelledParasite),"shelled_parasite_stunned",false),
            (entry(EncounterId::ThreeDarklings),"darkling_harden",false),
        ] {
            let mut c=watcher(e,&[(FearNoEvil,up)]);set_intent(&mut c,0,intent);
            let prepared=cards::prepare(&c,&CombatAction::PlayCard{instance:1,target:Some(TargetId(1))}).unwrap();
            let kinds=prepared.effects.iter().map(|e|e.kind.clone()).collect::<Vec<_>>();
            assert!(matches!(kinds.first(),Some(EffectKind::Damage(DamagePayload::Fixed(n))) if *n==face(up,8,11)),"{intent}: hit {kinds:?}");
            assert_eq!(kinds.len(),1+calm as usize,"{intent}: {kinds:?}");
            if calm{assert_eq!(kinds[1],EffectKind::Character(CharacterEffect::Stance(Stance::Calm)),"hit queued ahead of the Calm request");}
            play(&mut c,1,Some(1));
            assert_eq!(c.state.character.stance,if calm{Stance::Calm}else{Stance::Neutral},"FearNoEvil+{up} vs {intent}");
        }
        let mut c=watcher(entry(EncounterId::OrbWalker),&[(FearNoEvil,up)]);let before=c.state.enemies[0].hp;
        play(&mut c,1,Some(1));assert_eq!(before-c.state.enemies[0].hp,face(up,8,11));

        // Indignation: in Wrath 3/5 Vulnerable to every enemy; otherwise only Wrath (research:108).
        let mut c=watcher(entry(EncounterId::ThreeDarklings),&[(Indignation,up),(Indignation,up)]);
        play(&mut c,1,None);
        assert_eq!(c.state.character.stance,Stance::Wrath);
        assert!((0..3).all(|slot|amount(&c,slot,"vulnerable")==0),"the non-Wrath branch applies nothing");
        play(&mut c,2,None);
        assert_eq!(c.state.character.stance,Stance::Wrath);
        assert!((0..3).all(|slot|amount(&c,slot,"vulnerable")==face(up,3,5)),"every enemy gets Vulnerable");

        // InnerPeace: in Calm draw 3/4; otherwise only Calm (research:109).
        let mut deck=vec![(InnerPeace,up),(InnerPeace,up)];deck.extend(fillers(6));
        let mut c=watcher(entry(EncounterId::OrbWalker),&deck);
        for id in 3..=8{place(&mut c,id,SelectionSourceZone::Draw);}
        play(&mut c,1,None);
        assert_eq!((c.state.character.stance,c.state.piles.hand.len(),c.state.piles.draw.len()),(Stance::Calm,1,6),"non-Calm branch draws nothing");
        play(&mut c,2,None);
        assert_eq!((c.state.character.stance,c.state.piles.hand.len()),(Stance::Calm,face(up,3,4) as usize));

        // WheelKick 15/20, then draw 2 (research:129).
        let mut deck=vec![(WheelKick,up)];deck.extend(fillers(3));
        let mut c=watcher(entry(EncounterId::OrbWalker),&deck);
        for id in 2..=4{place(&mut c,id,SelectionSourceZone::Draw);}
        let before=c.state.enemies[0].hp;play(&mut c,1,Some(1));
        assert_eq!((before-c.state.enemies[0].hp,c.state.piles.hand.len(),c.state.piles.draw.len()),(face(up,15,20),2,1));

        // Scrawl: draw until the hand has 10 cards (research:151; ExpertiseAction(10)); the
        // playing card is not in hand. Constructed piles via `place`.
        let mut deck=vec![(Scrawl,up)];deck.extend(fillers(12));
        let mut c=watcher(entry(EncounterId::OrbWalker),&deck);
        for id in 2..=13{place(&mut c,id,SelectionSourceZone::Draw);}
        for id in [1,2,3]{place(&mut c,id,SelectionSourceZone::Hand);}
        assert_eq!((c.state.piles.hand.len(),c.state.piles.draw.len()),(3,10));
        play(&mut c,1,None);
        assert_eq!((c.state.piles.hand.len(),c.state.piles.draw.len()),(10,2),"2 in hand: 8 drawn, stops at 10 with 2 left");
        let mut deck=vec![(Scrawl,up)];deck.extend(fillers(6));
        let mut c=watcher(entry(EncounterId::OrbWalker),&deck);
        for id in 3..=5{place(&mut c,id,SelectionSourceZone::Draw);}
        for id in 6..=7{place(&mut c,id,SelectionSourceZone::Discard);}
        play(&mut c,1,None);
        assert_eq!((c.state.piles.hand.len(),c.state.piles.draw.len(),c.state.piles.discard.len()),(6,0,0),
            "1 in hand + 3 draw + 2 reshuffled discard: stops at 6 when both piles are empty");

        // SpiritShield: 3/4 per other card in hand at use, Block modifiers once
        // (research:152, :469; annotations:85-89; W-A06 research:602).
        for (powers,others,expected) in [
            (vec![],3usize,face(up,9,12)),(vec![power("dexterity",2)],3,face(up,9+2,12+2)),(vec![],0,0),
        ] {
            let mut deck=vec![(SpiritShield,up)];deck.extend(fillers(others));
            let mut c=watcher(entry(EncounterId::OrbWalker),&deck);
            c.state.player_full_powers.extend(powers);
            play(&mut c,1,None);
            assert_eq!(c.state.player.block,expected,"SpiritShield+{up} with {others} others");
        }
    }
}

#[test]
fn m25_direct_existing_payloads_match_watcher_values(){
    for up in [0,1] {
        // Consecrate 0: 5/8 to every enemy (design §5.1; research:75).
        let mut c=watcher(entry(EncounterId::ThreeDarklings),&[(Consecrate,up)]);let before=hp(&c);
        play(&mut c,1,None);
        assert_eq!((0..3).map(|i|before[i]-c.state.enemies[i].hp).collect::<Vec<_>>(),vec![face(up,5,8);3]);
        assert_eq!(c.state.player.energy,9);
        // EmptyBody 1: 7/10 Block, then exit the stance (research:79); leaving Calm grants 2
        // energy (accepted CalmExit owner).
        let mut c=watcher(entry(EncounterId::OrbWalker),&[(EmptyBody,up)]);stance(&mut c,Stance::Calm);
        play(&mut c,1,None);
        assert_eq!((c.state.player.block,c.state.character.stance,c.state.player.energy),(face(up,7,10),Stance::Neutral,9-1+2));
        // EmptyMind 1: draw 2/3, then exit the stance (research:103).
        let mut deck=vec![(EmptyMind,up)];deck.extend(fillers(4));
        let mut c=watcher(entry(EncounterId::OrbWalker),&deck);stance(&mut c,Stance::Wrath);
        for id in 2..=5{place(&mut c,id,SelectionSourceZone::Draw);}
        play(&mut c,1,None);
        assert_eq!((c.state.piles.hand.len(),c.state.character.stance,c.state.player.energy),(face(up,2,3) as usize,Stance::Neutral,8));
    }
}

/// Independent model of the source random-enemy law (research §8.2 "Ragnarok", lines
/// 525-534): each hit draws from the card-random stream over living enemies in group
/// order, including a request at n=1, and draws nothing once none is living. Returns the
/// final HP, the number of requests and the stream it leaves.
fn model_ragnarok(c:&Combat,hits:usize,per:i32)->(Vec<i32>,usize,rng::StreamState){
    let mut hp=hp(c);let mut stream=c.state.card_rng.clone().unwrap();let mut draws=0;
    let mut order=(0..c.state.enemies.len()).collect::<Vec<_>>();
    order.sort_by_key(|i|c.state.actor_slots.get(&c.state.enemies[*i].instance_id).copied().unwrap_or(u8::MAX));
    for _ in 0..hits {
        let live=order.iter().copied().filter(|i|c.state.enemies[*i].life==ActorLife::Active&&hp[*i]>0).collect::<Vec<_>>();
        if live.is_empty(){break}
        let index=if live.len()==1{stream=rng::draw_u64(&stream).unwrap().1;0}
            else{let (index,next)=rng::uniform_below(&stream,live.len() as u128).unwrap();stream=next;index as usize};
        draws+=1;hp[live[index]]-=per;
    }
    (hp.into_iter().map(|v|v.max(0)).collect(),draws,stream)
}

#[test]
fn m25_direct_ragnarok_random_targets(){
    for up in [0,1] {
        let (hits,per)=(face(up,5,6) as usize,face(up,5,6)); // design §5.1 / research:150
        // Face: cost 3, untargeted, no Retain/Exhaust.
        let mut c=watcher(entry(EncounterId::OrbWalker),&[(Ragnarok,up)]);
        assert!(c.step_full(&CombatAction::PlayCard{instance:1,target:Some(TargetId(1))}).is_err());
        let before=c.state.enemies[0].hp;
        let (_,draws,singleton)=model_ragnarok(&c,hits,per);assert_eq!(draws,hits);
        play(&mut c,1,None);
        assert_eq!((c.state.player.energy,in_pile(&c.state.piles.discard,1)),(6,true));
        assert_eq!(before-c.state.enemies[0].hp,(hits as i32)*per,"Ragnarok+{up}: {hits} separate hits of {per}");
        assert_eq!(c.state.card_rng.as_ref(),Some(&singleton),"one card-random request per hit, including n=1");

        // Three Darklings, one at constructed low HP so it is downed mid-sequence and the
        // remaining hits re-choose among the living (research:525-534).
        let mut c=watcher(entry(EncounterId::ThreeDarklings),&[(Ragnarok,up)]);
        let n=c.state.enemies[1].hp-1;hit(&mut c,1,n);assert_eq!(c.state.enemies[1].hp,1);
        let (expected,draws,stream)=model_ragnarok(&c,hits,per);
        play(&mut c,1,None);
        assert_eq!(draws,hits);
        assert_eq!(hp(&c),expected,"Ragnarok+{up} per-hit targets");
        assert_eq!(c.state.card_rng.as_ref(),Some(&stream));

        // Victory stops the remaining hits: a lone enemy at 2*per-1 HP dies on hit 2;
        // no further target request is made (research:528, empty group draws nothing).
        let mut c=watcher(entry(EncounterId::OrbWalker),&[(Ragnarok,up)]);
        let n=c.state.enemies[0].hp-(2*per-1);hit(&mut c,0,n);
        let (_,draws,stream)=model_ragnarok(&c,hits,per);assert_eq!(draws,2);
        play(&mut c,1,None);
        assert_eq!((c.state.enemies[0].hp,c.state.outcome),(0,Some(Outcome::Victory)));
        assert_eq!(c.state.card_rng.as_ref(),Some(&stream),"exactly two requests before Victory");
    }
}
