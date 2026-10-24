use super::super::*;
use crate::content::{CardId,EncounterId};
use crate::combat::model::OwnedCard;

fn apply(c:&mut Combat,id:&'static str,n:i32){c.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::ApplyFullPower{id:FullPowerId(id),amount:n}}],None).unwrap();}
fn create(c:&mut Combat,id:CardId,destination:CardDestination){c.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::CreateCard{id,destination,upgrades:0}}],None).unwrap();}
fn dazed(c:&Combat)->usize{c.state.piles.all(c.state.in_play.as_ref()).filter(|c|c.full().is_some_and(|c|c.id==CardId::Dazed)).count()}
fn chosen(artifact:bool)->Combat {
    let mut e=super::m6_ordinary::entry(EncounterId::ChosenAndByrds);e.setup.enemies.remove(0);
    e.cards=vec![permanent_live_card(CardId::TwinStrike,CardInstanceId(1),0),permanent_live_card(CardId::Defend,CardInstanceId(2),0)];e.opening_draw=2;e.setup.player.draw_per_turn=2;
    let mut c=Combat::enter(e).unwrap();if artifact{apply(&mut c,"artifact",1)}
    c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.state.player.hp,70);c.step_full(&CombatAction::EndTurn).unwrap();c
}

#[test]
fn m6_chosen_hex_adds_one_dazed_per_non_attack_not_hit() {
    for artifact in [false,true] {
        let mut c=chosen(artifact);assert_eq!(c.state.player_full_powers.iter().find(|p|p.id.0=="hex").map(|p|p.state.amount),if artifact{None}else{Some(1)});
        c.step_full(&CombatAction::PlayCard{instance:1,target:Some(TargetId(1))}).unwrap();assert_eq!(dazed(&c),0);
        let card_stream=c.state.card_rng.clone();let shuffle_stream=c.state.rng.shuffle.clone();
        c.step_full(&CombatAction::PlayCard{instance:2,target:None}).unwrap();assert_eq!(dazed(&c),usize::from(!artifact));
        if !artifact{assert_eq!(c.state.card_rng,card_stream,"empty Hex insertion is deterministic and consumes no scoped card RNG");}
        assert_eq!(c.state.rng.shuffle,shuffle_stream,"Hex status insertion is not opening/combat shuffle");
        if !artifact {let new=c.state.piles.draw[0].full().unwrap();assert_eq!((new.id,new.origin.clone()),(CardId::Dazed,CombatCardRef::CombatLocal(3)));assert_eq!(c.state.piles.exhaust.len(),0);}
    }
}

#[test]
fn m10_hex_power_producer_uses_nonempty_n_position_insertion_and_card_stream(){
    let mut c=chosen(false);
    for instance in [100,101]{
        let handle=CardHandle::parse(&instance.to_string()).unwrap();c.state.card_ids_issued.insert(handle);
        c.state.piles.draw.push(OwnedCard::Full(permanent_live_card(CardId::Strike,CardInstanceId(instance),0)));
    }
    let stream=c.state.card_rng.as_ref().unwrap().clone();let shuffle=c.state.rng.shuffle.clone();
    let (expected,next)=rng::uniform_below(&stream,2).unwrap();
    assert!(engine::validate_runtime(&c.state).is_ok());
    c.step_full(&CombatAction::PlayCard{instance:2,target:None}).unwrap();
    let actual=c.state.piles.draw.iter().position(|owned|owned.full().is_some_and(|card|card.id==CardId::Dazed)).unwrap();
    assert_eq!(actual,expected as usize,"actual Hex power-created Dazed uses n existing positions");
    assert_eq!(c.state.card_rng.as_ref().unwrap(),&next);
    assert_eq!(c.state.rng.shuffle,shuffle,"Hex insertion is distinct from opening/combat shuffle");
    assert!(engine::validate_runtime(&c.state).is_ok());
}

#[test]
fn m6_confused_randomizes_each_actual_draw_once() {
    let e=super::m6_ordinary::entry(EncounterId::Snecko);let mut c=Combat::enter(e.clone()).unwrap();c.step_full(&CombatAction::EndTurn).unwrap();
    assert!(c.state.player_full_powers.iter().any(|p|p.id.0=="confused"));let previous=c.state.piles.hand[0].full().unwrap().clone();
    let stream=c.state.card_rng.as_ref().unwrap();c.state.card_rng=Some(rng::StreamState::new(stream.id().clone(),[0;32],0).unwrap());
    create(&mut c,CardId::Shiv,CardDestination::Draw);create(&mut c,CardId::Slimed,CardDestination::Draw);create(&mut c,CardId::Dazed,CardDestination::Draw);
    c.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::Draw(3)}],None).unwrap();
    let view=c.public_full().unwrap();let cost=|id|view.hand.iter().find(|c|c.id==id).unwrap().effective_cost;
    assert_eq!((cost(CardId::Dazed),cost(CardId::Slimed),cost(CardId::Shiv)),(None,Some(2),Some(3)));assert_eq!(c.state.card_rng.as_ref().unwrap().word_index(),2);
    assert!(c.state.piles.hand.iter().any(|card|card.full()==Some(&previous)),"an unrelated held card isn't rerandomized");
    let status=view.hand.iter().find(|card|card.id==CardId::Dazed).unwrap().instance.0;let before=c.snapshot();assert!(c.step_full(&CombatAction::PlayCard{instance:status,target:None}).is_err());assert_eq!(c.snapshot(),before);
    let slimed=view.hand.iter().find(|card|card.id==CardId::Slimed).unwrap().instance.0;c.step_full(&CombatAction::PlayCard{instance:slimed,target:None}).unwrap();assert_eq!(c.state.player.energy,1,"payment uses randomized2, not native base1");
    while c.state.piles.hand.len()<10{create(&mut c,CardId::Shiv,CardDestination::Hand)}let before=c.state.card_rng.clone();c.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::Draw(5)}],None).unwrap();assert_eq!(c.state.card_rng,before);
    let fresh=Combat::enter(e).unwrap();assert_eq!(fresh.public_full().unwrap().hand[0].effective_cost,Some(1));assert_eq!(fresh.state.piles.hand[0].full().unwrap().local.randomized_cost,None);
}

#[test]
fn m6_wound_is_unplayable_non_ethereal_and_combat_local() {
    let mut e=super::m5_elites::entry(EncounterId::Cultist);e.cards=(1..=5).map(|id|permanent_live_card(CardId::Strike,CardInstanceId(id),0)).collect();
    let mut c=Combat::enter(e).unwrap();create(&mut c,CardId::Wound,CardDestination::Hand);
    let status=c.public_full().unwrap().hand.into_iter().find(|c|c.id==CardId::Wound).unwrap();assert_eq!(status.origin,CombatCardRef::CombatLocal(6));assert_eq!(status.effective_cost,None);
    assert!(!c.legal_actions_full().unwrap().iter().any(|a|matches!(a,CombatAction::PlayCard{instance:6,..})));
    c.step_full(&CombatAction::EndTurn).unwrap();assert!(c.state.piles.discard.iter().any(|card|card.full().is_some_and(|c|c.id==CardId::Wound&&c.instance==CardInstanceId(6))));assert!(c.state.piles.exhaust.is_empty());
}

#[test]
fn m6_confused_redraw_overwrites_an_older_discount() {
    let e=super::m6_ordinary::entry(EncounterId::Snecko);let mut c=Combat::enter(e).unwrap();c.step_full(&CombatAction::EndTurn).unwrap();
    let mut card=c.state.piles.hand.pop().unwrap();card.full_mut().unwrap().local.temporary_cost=-1;c.state.piles.draw.push(card);
    let stream=c.state.card_rng.as_ref().unwrap();c.state.card_rng=Some(rng::StreamState::new(stream.id().clone(),[0;32],0).unwrap());
    c.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::Draw(1)}],None).unwrap();
    assert_eq!(c.public_full().unwrap().hand[0].effective_cost,Some(2),"redraw negates the previous discount rather than adding it to the new cost2");
}

#[test]
fn m10_blood_for_blood_confused_draw_is_exact_turn_cost_and_hp_loss_updates_it_once(){
    let mut e=super::m6_ordinary::entry(EncounterId::Snecko);
    e.cards[0]=permanent_live_card(CardId::BloodForBlood,CardInstanceId(1),0);
    let mut c=Combat::enter(e).unwrap();c.step_full(&CombatAction::EndTurn).unwrap();
    assert!(c.state.player_full_powers.iter().any(|power|power.id.0=="confused"&&power.state.amount>0));
    c.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::Damage(DamagePayload::HpLoss(1))}],None).unwrap();
    let position=c.state.piles.hand.iter().position(|owned|owned.handle()==CardHandle::parse("1").unwrap()).unwrap();
    let mut bfb=c.state.piles.hand.remove(position);c.state.piles.draw.push(bfb.clone());
    let stream=c.state.card_rng.as_ref().unwrap();c.state.card_rng=Some(rng::StreamState::new(stream.id().clone(),[0;32],0).unwrap());
    c.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::Draw(1)}],None).unwrap();
    let drawn=c.state.piles.hand.iter().find_map(OwnedCard::full).filter(|card|card.id==CardId::BloodForBlood).unwrap();
    assert_eq!(drawn.local.randomized_cost,Some(2),"existing Confused draw stream selects the literal fresh turn cost2");
    assert_eq!(drawn.local.combat_cost_reduction,1);
    assert_eq!(card_state::effective_cost(drawn,4).unwrap(),2,
        "prior combat reduction must not be subtracted a second time from a fresh Confused setCostForTurn value");
    c.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::Damage(DamagePayload::HpLoss(1))}],None).unwrap();
    let after_loss=c.state.piles.hand.iter().find_map(OwnedCard::full).filter(|card|card.id==CardId::BloodForBlood).unwrap();
    assert_eq!(after_loss.local.randomized_cost,Some(1));
    assert_eq!(after_loss.local.combat_cost_reduction,2);
    assert_eq!(card_state::effective_cost(after_loss,4).unwrap(),1,
        "HP loss after Confused assignment reduces the current turn price exactly once");
    let position=c.state.piles.hand.iter().position(|owned|owned.handle()==CardHandle::parse("1").unwrap()).unwrap();
    bfb=c.state.piles.hand.remove(position);c.state.piles.draw.push(bfb);
    let stream=c.state.card_rng.as_ref().unwrap();c.state.card_rng=Some(rng::StreamState::new(stream.id().clone(),[0;32],0).unwrap());
    c.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::Draw(1)}],None).unwrap();
    let recycled=c.state.piles.hand.iter().find_map(OwnedCard::full).filter(|card|card.id==CardId::BloodForBlood).unwrap();
    assert_eq!(recycled.local.randomized_cost,Some(2));
    assert_eq!(recycled.local.combat_cost_reduction,2);
    assert_eq!(card_state::effective_cost(recycled,4).unwrap(),2,
        "recycling establishes a fresh turn cost without losing persistent combat history");
    assert!(engine::validate_runtime(&c.state).is_ok());
}

#[test]
fn m6_hex_inserts_among_existing_draw_cards_without_reordering_them() {
    let mut c=chosen(false);create(&mut c,CardId::Burn,CardDestination::Draw);create(&mut c,CardId::Burn,CardDestination::Draw);
    let stream=c.state.card_rng.as_ref().unwrap();c.state.card_rng=Some(rng::StreamState::new(stream.id().clone(),[0;32],0).unwrap());
    c.step_full(&CombatAction::PlayCard{instance:2,target:None}).unwrap();
    // Rules-v2 follows the producer's n positions. The prior local n+1
    // interpretation put generated ID5 at slot1; the corrected n=2 law puts
    // it at slot0 without changing the order of existing IDs3 and4.
    assert_eq!(c.state.piles.draw.iter().map(|c|c.full().unwrap().instance.0).collect::<Vec<_>>(),vec![5,3,4]);assert_eq!(c.state.card_rng.as_ref().unwrap().word_index(),1);
}

#[test]
fn m6_draw_rng_fault_keeps_both_transfers_and_only_completed_randomization() {
    let e=super::m6_ordinary::entry(EncounterId::Snecko);let mut c=Combat::enter(e).unwrap();c.step_full(&CombatAction::EndTurn).unwrap();
    create(&mut c,CardId::Shiv,CardDestination::Draw);create(&mut c,CardId::Shiv,CardDestination::Draw);
    let stream=c.state.card_rng.as_ref().unwrap();c.state.card_rng=Some(rng::StreamState::new(stream.id().clone(),[0;32],(1_u128<<64)-1).unwrap());
    assert!(c.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::Draw(2)}],None).is_err());
    assert_eq!(c.state.card_rng.as_ref().unwrap().word_index(),1_u128<<64);assert_eq!(c.state.piles.hand.len(),3);assert!(c.state.piles.draw.is_empty());
    let local=|id|c.state.piles.hand.iter().find_map(|c|c.full().filter(|c|c.instance.0==id)).unwrap();
    assert!(local(3).local.randomized_cost.is_some());assert_eq!(local(2).local.randomized_cost,None);
    assert_eq!(c.state.current.as_ref().unwrap().kind,EffectKind::AfterCardDrawn{card:super::super::card_state::handle(CardInstanceId(2))});assert_eq!(c.state.fault.as_ref().unwrap().kind,FaultKind::RngExhausted);
    let before=c.snapshot();assert!(c.step_full(&CombatAction::EndTurn).is_err());assert_eq!(c.snapshot(),before);
}

#[test]
fn m6_hex_rng_fault_keeps_paid_card_block_and_unissued_status_frontier() {
    let mut c=chosen(false);create(&mut c,CardId::Burn,CardDestination::Draw);create(&mut c,CardId::Burn,CardDestination::Draw);
    let stream=c.state.card_rng.as_ref().unwrap();c.state.card_rng=Some(rng::StreamState::new(stream.id().clone(),[0;32],1_u128<<64).unwrap());
    assert!(c.step_full(&CombatAction::PlayCard{instance:2,target:None}).is_err());assert_eq!((c.state.player.energy,c.state.player.block,c.state.card_ids_issued.len(),dazed(&c)),(2,5,4,0));
    assert!(c.state.piles.discard.iter().any(|card|card.full().is_some_and(|c|c.instance==CardInstanceId(2))));assert_eq!(c.state.fault.as_ref().unwrap().kind,FaultKind::RngExhausted);
    assert!(matches!(c.state.current.as_ref().unwrap().kind,EffectKind::CreateCard{id:CardId::Dazed,destination:CardDestination::ShuffledDraw,..}));
    let before=c.snapshot();assert!(c.step_full(&CombatAction::EndTurn).is_err());assert_eq!(c.snapshot(),before);
}

#[test]
fn m6_snecko_glare_is_artifact_blocked_and_free_or_x_cards_keep_payment_rules() {
    let mut e=super::m6_ordinary::entry(EncounterId::Snecko);e.setup.player_powers=vec![FullPowerEntry{id:FullPowerId("artifact"),state:PowerState{amount:1,skip_first:false,monster_source_grace:false}}];
    let mut blocked=Combat::enter(e).unwrap();blocked.step_full(&CombatAction::EndTurn).unwrap();assert!(!blocked.state.player_full_powers.iter().any(|p|matches!(p.id.0,"artifact"|"confused")));assert_eq!(blocked.state.card_rng.as_ref().unwrap().word_index(),0);assert_eq!(blocked.public_full().unwrap().hand[0].effective_cost,Some(1));
    let mut c=Combat::enter(super::m6_ordinary::entry(EncounterId::Snecko)).unwrap();c.step_full(&CombatAction::EndTurn).unwrap();
    create(&mut c,CardId::Shiv,CardDestination::Draw);c.state.piles.draw.last_mut().unwrap().full_mut().unwrap().local.free_play=true;
    create(&mut c,CardId::Shiv,CardDestination::Draw);c.state.piles.draw.last_mut().unwrap().full_mut().unwrap().local.x_cost=true;
    let stream=c.state.card_rng.as_ref().unwrap();c.state.card_rng=Some(rng::StreamState::new(stream.id().clone(),[0;32],0).unwrap());
    c.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::Draw(2)}],None).unwrap();assert_eq!(c.state.card_rng.as_ref().unwrap().word_index(),1,"X is unchanged, free card still has its underlying cost rolled");
    c.step_full(&CombatAction::PlayCard{instance:2,target:Some(TargetId(1))}).unwrap();assert_eq!(c.state.player.energy,1,"Confused clears a drawn card's free-to-play flag, so the randomized cost is paid");
    c.step_full(&CombatAction::PlayCard{instance:3,target:Some(TargetId(1))}).unwrap();assert_eq!(c.state.player.energy,0);
}

#[test]
fn m6_chosen_byrd_formation_acts_before_its_partners_vulnerable_application() {
    let mut e=super::m6_ordinary::entry(EncounterId::ChosenAndByrds);e.setup.player.hp=160;e.setup.player.max_hp=160;
    assert_eq!((e.setup.enemies[0].enemy_id.as_str(),e.setup.enemies[1].enemy_id.as_str()),("byrd","chosen"));
    let mut c=Combat::enter(e).unwrap();c.step_full(&CombatAction::EndTurn).unwrap();
    for (slot,cursor) in [(0,5),(1,0)]{let stream=c.state.enemies[slot].full_ai_rng.as_ref().unwrap();c.state.enemies[slot].full_ai_rng=Some(rng::StreamState::new(stream.id().clone(),[0;32],cursor).unwrap());}
    c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.state.enemies[1].intent_id.as_str(),"chosen_debilitate");
    let byrd=Actor::Enemy(c.state.enemies[0].instance_id.clone());c.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(byrd),kind:EffectKind::ApplyFullPower{id:FullPowerId("strength"),amount:2}}],None).unwrap();
    let byrd=&c.state.enemies[0];let hits=match byrd.intent_id.as_str(){"byrd_peck"=>5,"byrd_swoop"=>1,_=>panic!("literal fixture should select an attack")};let before=c.state.player.hp;let bird_loss=byrd.intent_damage.unwrap()*hits;
    c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.state.player.hp,before-bird_loss-10,"Byrd's earlier attack must not receive Chosen's later Vulnerable");assert!(c.state.player_full_powers.iter().any(|p|p.id.0=="vulnerable"&&p.state.amount==2));
}
