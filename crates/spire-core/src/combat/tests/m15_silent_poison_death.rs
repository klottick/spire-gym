use super::super::*;
use crate::content::{CardId,Catalog};
use super::full_runtime::entered_combat;

fn id(token:&str)->CardId{Catalog::standard().cards().iter().find(|r|r.id.as_str()==token).unwrap_or_else(||panic!("missing poison/death green card {token}")).id}
fn fight(token:&str,up:u32)->Combat{let mut c=entered_combat(9,9,|card|*card=permanent_live_card(id(token),card.instance,up));c.state.enemies[0].hp=500;c.state.enemies[0].max_hp=500;c}
fn enemy_power(c:&mut Combat,name:&'static str,amount:i32){c.state.enemies[0].full_powers.push(FullPowerEntry{id:FullPowerId(name),state:PowerState{amount,skip_first:false,monster_source_grace:false}})}
fn amount(c:&Combat,slot:usize,name:&str)->i32{c.state.enemies[slot].full_powers.iter().find(|p|p.id.0==name).map_or(0,|p|p.state.amount)}
fn play(c:&mut Combat,target:bool){c.step_full(&CombatAction::PlayCard{instance:3,target:target.then_some(TargetId(1))}).unwrap();}

#[test]
fn m15_silent_poison_death_pairs_use_real_applications_and_capture_conditions(){
    for up in 0..=1 {
        for poisoned in [false,true]{
            let mut c=fight("bane",up);if poisoned{enemy_power(&mut c,"poison",2)}
            play(&mut c,true);assert_eq!(c.state.enemies[0].hp,500-(7+3*up as i32)*if poisoned{2}else{1});
        }
        let mut c=fight("bouncing_flask",up);play(&mut c,false);
        assert_eq!(amount(&c,0,"poison"),9+3*up as i32);
        for weak in [false,true]{
            let mut c=fight("heel_hook",up);if weak{enemy_power(&mut c,"weak",2)}
            play(&mut c,true);assert_eq!(c.state.enemies[0].hp,495-3*up as i32);
            assert_eq!(c.state.player.energy,if weak{9}else{8});
        }
        let mut c=fight("terror",up);play(&mut c,true);
        assert_eq!(amount(&c,0,"vulnerable"),99);assert_eq!(c.state.player.energy,8+up as i32);
        assert!(c.state.piles.exhaust.iter().any(|x|x.handle().as_str()=="3"));
    }
}

#[test]
fn m15_silent_poison_death_debuffs_have_independent_artifact_receipts(){
    for up in 0..=1 {
        for charges in 0..=2 {
            let mut cloud=fight("crippling_cloud",up);if charges>0{enemy_power(&mut cloud,"artifact",charges)}
            play(&mut cloud,false);
            assert_eq!(amount(&cloud,0,"poison"),if charges==0{4+3*up as i32}else{0});
            assert_eq!(amount(&cloud,0,"weak"),if charges<2{2}else{0});
            let mut corpse=fight("corpse_explosion",up);if charges>0{enemy_power(&mut corpse,"artifact",charges)}
            play(&mut corpse,true);
            assert_eq!(amount(&corpse,0,"poison"),if charges==0{6+3*up as i32}else{0});
            assert_eq!(amount(&corpse,0,"corpse_explosion"),i32::from(charges<2));
            if charges==0{
                let owner=corpse.state.enemies[0].instance_id.clone();
                corpse.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Enemy(owner)),kind:EffectKind::RemoveFullPower(FullPowerId("poison"))}],None).unwrap();
                assert_eq!(amount(&corpse,0,"corpse_explosion"),1);
            }
            let mut wail=fight("piercing_wail",up);if charges>0{enemy_power(&mut wail,"artifact",charges)}
            play(&mut wail,false);
            assert_eq!(amount(&wail,0,"strength"),if charges==0{-6-2*up as i32}else{0});
            assert_eq!(amount(&wail,0,"shackled"),if charges==0{6+2*up as i32}else{0});
            wail.step_full(&CombatAction::EndTurn).unwrap();
            assert_eq!(amount(&wail,0,"strength"),0);assert_eq!(amount(&wail,0,"shackled"),0);
        }
    }
}

#[test]
fn m15_silent_poison_death_explosion_uses_callback_max_hp_and_shared_thorns(){
    let mut c=fight("corpse_explosion",0);
    c.state.enemies[0].hp=6;c.state.enemies[0].max_hp=40;
    let mut survivor=c.state.enemies[0].clone();survivor.instance_id=EnemyHandle::parse("enemy-2").unwrap();survivor.hp=200;survivor.max_hp=200;
    c.state.enemies.push(survivor);
    play(&mut c,true);
    c.step_full(&CombatAction::PlayCard{instance:1,target:Some(TargetId(1))}).unwrap();
    assert_eq!(c.state.enemies[1].hp,160,"explosion is maxHP40, not Poison6 or remainingHP");
    assert!(c.fault_record().is_none());
}

#[test]
fn m15_silent_poison_death_half_dead_and_rebirth_keep_callback_owners(){
    use crate::content::EncounterId;
    for (encounter,slot,callback_hp) in [(EncounterId::ThreeDarklings,0,40),(EncounterId::AwakenedOne,2,300)]{
        let mut entry=super::m8_shapes::entry(encounter);
        entry.cards=vec![permanent_live_card(id("corpse_explosion"),CardInstanceId(3),0),permanent_live_card(CardId::StrikeGreen,CardInstanceId(1),0)];
        entry.opening_draw=2;
        let mut c=Combat::enter(entry).unwrap();
        for (index,enemy) in c.state.enemies.iter_mut().enumerate(){enemy.max_hp=if index==slot{callback_hp}else{400};enemy.hp=if index==slot{6}else{400};}
        c.step_full(&CombatAction::PlayCard{instance:3,target:Some(TargetId(slot as u64+1))}).unwrap();
        c.step_full(&CombatAction::PlayCard{instance:1,target:Some(TargetId(slot as u64+1))}).unwrap();
        assert_eq!(c.state.enemies[slot].life,ActorLife::HalfDead);
        assert_eq!(c.state.outcome,None);
        for (index,enemy) in c.state.enemies.iter().enumerate().filter(|(index,_)|*index!=slot){
            assert_eq!(enemy.hp,400-callback_hp,"{encounter:?} survivor{index}");
        }
        if encounter==EncounterId::AwakenedOne{
            assert_eq!(amount(&c,slot,"corpse_explosion"),0,"phase cleanse removes installation after callback capture");
            c.step_full(&CombatAction::EndTurn).unwrap();
            assert_eq!(c.state.enemies[slot].life,ActorLife::Active);
            assert_eq!(c.state.enemies[slot].hp,300);
        }
    }
}

#[test]
fn m15_silent_poison_death_last_enemy_does_not_create_an_explosion(){
    let mut c=fight("corpse_explosion",0);c.state.enemies[0].hp=6;
    play(&mut c,true);
    let observed=c.step_full_observed(&CombatAction::PlayCard{instance:1,target:Some(TargetId(1))});
    observed.result.unwrap();assert_eq!(c.state.outcome,Some(Outcome::Victory));
    assert!(!observed.effects.iter().any(|effect|matches!(effect.kind,observe::ResolvedEffectKind::Damage{category:observe::DamageCategory::Thorns,..})),"last-enemy cleanup is not another nominal AoE hit");
}

#[test]
fn m15_silent_poison_death_split_replacement_does_not_inherit_explosion(){
    let mut entry=enemies::setup(crate::content::EncounterId::SlimeBoss,1,0,
        &mut crate::run::RunRng::new([41;32],rng::Partition::Test,0),
        enemies::SetupContext{player:PlayerState{hp:300,max_hp:300,block:0,energy:3,max_energy:3,draw_per_turn:1,powers:vec![]},
            player_powers:vec![],character:crate::content::Character::Silent,
            cards:vec![permanent_live_card(CardId::DefendGreen,CardInstanceId(1),0)],opening_draw:1,encounter_visit:0}).unwrap();
    entry.cards=vec![permanent_live_card(id("corpse_explosion"),CardInstanceId(3),0),permanent_live_card(CardId::StrikeGreen,CardInstanceId(1),0)];entry.opening_draw=2;
    let mut c=Combat::enter(entry).unwrap();
    let parent=c.state.enemies[0].instance_id.clone();
    c.state.enemies[0].hp=c.state.enemies[0].max_hp/2;
    play(&mut c,true);
    c.step_full(&CombatAction::PlayCard{instance:1,target:Some(TargetId(1))}).unwrap();
    c.step_full(&CombatAction::EndTurn).unwrap();
    assert!(c.state.enemies.iter().any(|e|e.instance_id==parent&&e.life==ActorLife::Replaced));
    let children=c.state.enemies.iter().filter(|e|e.life==ActorLife::Active&&e.hp>0).collect::<Vec<_>>();
    assert_eq!(children.len(),2);
    assert!(children.iter().all(|e|!e.full_powers.iter().any(|p|p.id.0=="corpse_explosion")));
    assert!(!c.state.hook_events.iter().any(|event|event.hook==Hook::Death&&event.target==Some(Actor::Enemy(parent.clone()))));
}

#[test]
fn m15_silent_poison_death_flask_preselects_next_before_current_application(){
    let mut entry=super::m8_shapes::entry(crate::content::EncounterId::DonuAndDeca);
    entry.cards=vec![permanent_live_card(id("bouncing_flask"),CardInstanceId(3),0)];entry.opening_draw=1;
    let mut c=Combat::enter(entry).unwrap();
    for enemy in &mut c.state.enemies{enemy.full_powers.retain(|p|p.id.0!="artifact");}
    let stream=c.state.card_rng.as_ref().unwrap();
    c.state.card_rng=Some(rng::StreamState::new(stream.id().clone(),*stream.key(),(1u128<<64)-2).unwrap());
    let result=c.step_full(&CombatAction::PlayCard{instance:3,target:None});
    let crate::run::RunError::Faulted(first)=result.unwrap_err()else{panic!("expected retained card RNG fault")};
    assert_eq!(c.state.card_rng.as_ref().unwrap().word_index(),1u128<<64);
    assert_eq!(c.state.enemies.iter().flat_map(|e|&e.full_powers).filter(|p|p.id.0=="poison").map(|p|p.state.amount).sum::<i32>(),3,"first application accepted; next target draw faults before second application");
    assert_eq!(c.state.player.energy,1);
    assert_eq!(c.state.in_play.as_ref().unwrap().handle().as_str(),"3");
    assert!(matches!(c.state.current.as_ref().unwrap().kind,EffectKind::Silent(cards::SilentEffect::FlaskBounce{remaining:2,..})));
    let saved=c.snapshot();assert_eq!(c.step_full(&CombatAction::EndTurn),Err(crate::run::RunError::Faulted(first)));assert_eq!(c.snapshot(),saved);
}
