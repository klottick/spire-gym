use super::super::*;
use crate::content::{CardId, Catalog, Character};
use super::full_runtime::entered_combat;
use model::OwnedCard;

fn id(token:&str)->CardId {
    Catalog::standard().cards().iter().find(|r|r.id.as_str()==token)
        .unwrap_or_else(||panic!("missing executable history card {token}")).id
}
fn fight(token:&str,up:u32)->Combat {
    let mut c=entered_combat(9,9,|card|*card=permanent_live_card(id(token),card.instance,up));
    c.state.character=characters::CharacterState::for_character(Character::Silent);
    c.state.enemies[0].hp=500;c.state.enemies[0].max_hp=500;c
}
fn add(c:&mut Combat,id:CardId,instance:u64,draw:bool){
    let card=permanent_live_card(id,CardInstanceId(instance),0);
    assert!(c.state.card_ids_issued.insert(card_state::handle(card.instance)));
    if draw{c.state.piles.draw.push(OwnedCard::Full(card))}else{c.state.piles.hand.push(OwnedCard::Full(card))}
}
fn play(c:&mut Combat,target:bool){c.step_full(&CombatAction::PlayCard{instance:3,target:target.then_some(TargetId(1))}).unwrap();}

#[test]
fn m15_silent_history_draw_then_selected_discard_resumes_once(){
    for (token,draws,discards,cost) in [("acrobatics",[3,4],[1,1],1),("prepared",[1,2],[1,2],0)]{
        for up in 0..=1 {
            let mut c=fight(token,up);
            for n in 100..108{add(&mut c,CardId::DefendGreen,n,true)}
            play(&mut c,false);
            let view=c.public_full().unwrap();
            assert_eq!(view.hand.len(),2+draws[up as usize]);
            assert_eq!(view.player.energy,9-cost);
            assert_eq!(view.selection.as_ref().unwrap().minimum,discards[up as usize]);
            for n in 0..discards[up as usize] {
                let instance=c.public_full().unwrap().hand[n].instance.0;
                c.step_full(&CombatAction::SelectCard{instance}).unwrap();
            }
            assert!(c.public_full().unwrap().selection.is_none());
            assert_eq!(c.state.piles.hand.len(),2+draws[up as usize]-discards[up as usize]);
            assert_eq!(c.state.piles.discard.iter().filter(|x|x.handle().as_str()=="3").count(),1);
            assert_eq!(c.state.player.energy,9-cost);
        }
    }
}

#[test]
fn m15_silent_history_concentrate_undersized_keeps_energy_and_callbacks(){
    for up in 0..=1 {
        for count in 0..=1 {
            let mut c=fight("concentrate",up);
            c.state.piles.hand.retain(|x|x.handle().as_str()=="3");
            // Removed fixture cards are retained as inactive owned resources.
            for n in [1,2]{c.state.inactive_cards.push(OwnedCard::Full(permanent_live_card(CardId::Defend,CardInstanceId(n),0)))}
            if count==1{add(&mut c,CardId::Tactician,100,false)}
            play(&mut c,false);
            assert_eq!(c.state.player.energy,11+count);
            assert!(c.state.piles.hand.is_empty());
            assert!(c.state.pending_selection.is_none());
        }
    }
}

#[test]
fn m15_silent_history_unload_discards_snapshot_nonattacks_only(){
    for up in 0..=1 {
        let mut c=fight("unload",up);
        for card in &mut c.state.piles.hand {
            if card.handle().as_str()=="1"{*card=OwnedCard::Full(permanent_live_card(CardId::Reflex,CardInstanceId(1),0))}
            if card.handle().as_str()=="2"{*card=OwnedCard::Full(permanent_live_card(CardId::NoxiousFumes,CardInstanceId(2),0))}
        }
        add(&mut c,CardId::Tactician,100,false);add(&mut c,CardId::Wound,101,false);
        for n in 200..205{add(&mut c,CardId::DefendGreen,n,true)}
        play(&mut c,true);
        assert_eq!(c.state.enemies[0].hp,if up==0{486}else{482});
        assert_eq!(c.state.player.energy,9);
        assert_eq!(c.state.piles.hand.len(),2,"Reflex children stay outside original snapshot");
        for n in [1,2,3,100,101]{assert!(c.state.piles.discard.iter().any(|x|x.handle().as_str()==n.to_string()))}
    }
}

#[test]
fn m15_silent_history_counts_cards_not_hits_and_unplayable_skills(){
    for up in 0..=1 {
        let mut c=fight("finisher",up);
        *c.state.piles.hand.iter_mut().find(|x|x.handle().as_str()=="1").unwrap()=OwnedCard::Full(permanent_live_card(id("riddle_with_holes"),CardInstanceId(1),0));
        c.step_full(&CombatAction::PlayCard{instance:1,target:Some(TargetId(1))}).unwrap();
        play(&mut c,true);
        assert_eq!(c.state.enemies[0].hp,if up==0{479}else{477},"five Riddle hits count as one Attack");
        let mut c=fight("flechettes",up);
        *c.state.piles.hand.iter_mut().find(|x|x.handle().as_str()=="1").unwrap()=OwnedCard::Full(permanent_live_card(CardId::Reflex,CardInstanceId(1),0));
        add(&mut c,CardId::Tactician,100,false);
        play(&mut c,true);
        assert_eq!(c.state.enemies[0].hp,if up==0{488}else{482},"three Skills including two unplayable reactions");
    }
}

#[test]
fn m15_silent_history_expertise_escape_and_grand_finale_use_action_state(){
    for up in 0..=1 {
        let mut c=fight("expertise",up);
        for n in 100..110{add(&mut c,CardId::DefendGreen,n,true)}
        play(&mut c,false);
        assert_eq!(c.state.piles.hand.len(),6+up as usize);
        assert_eq!(c.state.player.energy,8);
        let mut c=fight("escape_plan",up);add(&mut c,CardId::Tactician,100,true);
        play(&mut c,false);assert_eq!(c.state.player.block,3+2*up as i32);
        let mut c=fight("grand_finale",up);add(&mut c,CardId::DefendGreen,100,true);
        let before=c.state.clone();
        assert_eq!(c.step_full(&CombatAction::PlayCard{instance:3,target:None}),Err(crate::run::RunError::InvalidAction));
        assert_eq!(c.state,before);
        c.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::Draw(1)}],None).unwrap();
        play(&mut c,false);assert_eq!(c.state.enemies[0].hp,450-10*up as i32);
    }
}

#[test]
fn m15_silent_history_absent_draw_differs_from_draw_zero(){
    for up in 0..=1 {
        let mut gamble=fight("calculated_gamble",up);
        while let Some(index)=gamble.state.piles.hand.iter().position(|c|c.handle().as_str()!="3"){
            let card=gamble.state.piles.hand.remove(index);gamble.state.inactive_cards.push(card);
        }
        gamble.state.silent.drawn_cards=vec![CardId::DefendGreen];
        play(&mut gamble,false);
        assert_eq!(gamble.state.silent.drawn_cards,[CardId::DefendGreen]);
        assert_eq!(gamble.state.piles.exhaust.len(),usize::from(up==0));
        gamble.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:None,kind:EffectKind::Draw(0)}],None).unwrap();
        assert!(gamble.state.silent.drawn_cards.is_empty());

        let mut expertise=fight("expertise",up);
        for n in 100..106{add(&mut expertise,CardId::DefendGreen,n,false)}
        expertise.state.silent.drawn_cards=vec![CardId::Tactician];
        play(&mut expertise,false);
        assert_eq!(expertise.state.silent.drawn_cards,[CardId::Tactician]);
    }
}

#[test]
fn m15_silent_history_random_all_and_later_energy_keep_real_owners(){
    for up in 0..=1 {
        let mut c=fight("all_out_attack",up);
        play(&mut c,false);
        assert_eq!(c.state.enemies[0].hp,if up==0{490}else{486});
        assert_eq!(c.state.piles.hand.len(),1);
        assert_eq!(c.state.silent.discarded_this_turn,1);
        assert_eq!(c.state.player.energy,8);
        let mut c=fight("calculated_gamble",up);
        for n in 100..106{add(&mut c,CardId::DefendGreen,n,true)}
        play(&mut c,false);
        assert_eq!(c.state.piles.hand.len(),2);
        assert_eq!(c.state.silent.discarded_this_turn,2);
        assert_eq!(c.state.silent.drawn_cards.len(),2);
        let mut c=fight("sneaky_strike",up);
        play(&mut c,true);assert_eq!(c.state.player.energy,7);
        let mut c=fight("sneaky_strike",up);
        let discarded=c.state.piles.hand.remove(0);
        piles::discard_selected_card(&mut c.state,discarded,true).unwrap();
        play(&mut c,true);assert_eq!(c.state.player.energy,9);
        assert_eq!(c.state.enemies[0].hp,if up==0{488}else{484});
    }
}

#[test]
fn m15_silent_history_escape_followup_precedes_bottom_evolve_draw(){
    for (requested,shortage,expected) in [(1,false,0),(2,false,3),(2,true,3)] {
        let mut c=fight("escape_plan",0);
        c.state.player_full_powers.push(FullPowerEntry{id:FullPowerId("evolve"),state:PowerState{amount:1,skip_first:false,monster_source_grace:false}});
        if shortage {
            add(&mut c,CardId::Tactician,100,false);
            let index=c.state.piles.hand.iter().position(|c|c.handle().as_str()=="100").unwrap();
            let card=c.state.piles.hand.remove(index);c.state.piles.discard.push(card);
        }else{add(&mut c,CardId::Tactician,100,true)}
        add(&mut c,CardId::Wound,101,true);
        if requested==1 {play(&mut c,false)}else{
            // Supplied action-composition fixture: actual issued parent and
            // shared resolver; two-piece draw has the same follow-up owner.
            let index=c.state.piles.hand.iter().position(|c|c.handle().as_str()=="3").unwrap();
            c.state.in_play=Some(c.state.piles.hand.remove(index));
            c.state.phase=Phase::PlayerAction;
            let source=Source::Card(CardHandle::parse("3").unwrap());
            for kind in [EffectKind::Draw(2),EffectKind::Silent(cards::SilentEffect::EscapePlanBlock{block:3}),EffectKind::FinishCard,EffectKind::Decision]{
                c.state.pending.push_back(Effect{source:if kind==EffectKind::Decision{Source::None}else{source.clone()},target:None,kind});
            }
            engine::resolve(&mut c.state);
        }
        assert!(c.fault_record().is_none(),"{:?}",c.fault_record());
        assert_eq!(c.state.player.block,expected);
    }
}
