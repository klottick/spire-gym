use super::*;
use crate::{
    combat::{Actor, Combat, CombatAction, CombatCardRef, CombatEntry,
        DamagePayload, Effect, EffectKind, EnemyHandle, FullCombatSetup, FullEnemySetup,
        FullIntentId, LiveCard, PlayerState, Source, ResolvedEffectKind},
    content::{CardId, Character, EnemyId, RelicId},
    run::{CreationProfile, Run, RunConfig, RunRng},
};

fn run_with_relic(relic:RelicId)->Run {
    let mut run=Run::new(RunConfig{character:Character::Ironclad,ascension:0},
        CreationProfile::experienced(),RunRng::new([151;32],crate::combat::rng::Partition::Test,0)).unwrap();
    crate::items::acquire_relic(run.state_mut_for_test(),relic).unwrap();
    run
}

fn entry(cards:usize,opening_draw:usize,enemies:usize,hp:i32)->CombatEntry {
    let suffix=format!("task15-{cards}-{opening_draw}-{enemies}-{hp}");
    let shuffle_id=crate::combat::rng::StreamId::new(crate::combat::rng::Domain::RunCombatShuffle,vec![suffix.clone()]).unwrap();
    let card_id=crate::combat::rng::StreamId::new(crate::combat::rng::Domain::RunCombatCardRandom,vec![suffix.clone()]).unwrap();
    let enemies=(0..enemies).map(|i|{
        let owner=EnemyHandle::parse(&format!("enemy-{}",i+1)).unwrap();
        let ai=crate::combat::rng::StreamId::new(crate::combat::rng::Domain::RunCombatEnemyAi,
            vec![suffix.clone(),owner.as_str().to_owned()]).unwrap();
        FullEnemySetup{initial_memory:None,instance_id:owner,ascension:0,
            profile:crate::combat::enemies::EnemyProfile::Standard,enemy_id:EnemyId::Cultist,
            hp:40,max_hp:40,block:0,intent:FullIntentId::Incantation,powers:vec![],
            ai_rng:crate::combat::rng::StreamState::new(ai,[153;32],0).unwrap()}
    }).collect();
    let setup=FullCombatSetup{enemy_damage_bases:Default::default(),
        player:PlayerState{hp,max_hp:80,block:0,energy:3,max_energy:3,draw_per_turn:5,powers:vec![]},
        player_powers:vec![],character:Character::Ironclad,
        shuffle_rng:crate::combat::rng::StreamState::new(shuffle_id,[152;32],0).unwrap(),
        card_rng:crate::combat::rng::StreamState::new(card_id,[154;32],0).unwrap(),enemies};
    let cards=(1..=cards as u64).map(|instance|LiveCard{instance:crate::combat::CardInstanceId(instance),
        id:CardId::Strike,origin:CombatCardRef::Permanent(crate::combat::CardInstanceId(instance)),
        upgrades:0,local:crate::combat::CombatCardData::default(),copy_family:instance,
        copy_provenance:crate::combat::card_state::CopyProvenance::Original}).collect();
    CombatEntry{setup,opening_draw,cards}
}

fn enter(run:&mut Run,entry:CombatEntry)->Combat{
    let mut owner=Some(CombatItemContext::take_from_run(run.state_mut_for_test()));
    let combat=Combat::enter_with_item_context(entry,&mut owner).unwrap();
    assert!(owner.is_none(),"actual run inventory transfers into the combat owner");
    combat.public_full().expect("actual setup publishes a valid full observation");
    combat.validate_runtime_for_test().expect("actual setup validates canonical actors, every issued card, and owner custody before commands");
    combat
}

fn hp_loss(source:Source,amount:i32)->Effect{
    Effect{source,target:Some(Actor::Player),kind:EffectKind::Damage(DamagePayload::HpLoss(amount))}
}

#[test]
fn m10_ironclad_relic_turns_brimstone_applies_to_current_formation_at_each_player_start(){
    let mut run=run_with_relic(RelicId::Brimstone);
    let mut combat=enter(&mut run,entry(12,5,2,70));
    let first=combat.public_full().unwrap();
    assert_eq!(first.player.powers.iter().find(|power|power.id.0=="strength").map(|power|power.amount),Some(2));
    assert!(first.enemies.iter().all(|enemy|enemy.powers.iter().any(|power|power.id.0=="strength"&&power.amount==1)));

    let turn=combat.step_full_observed(&CombatAction::EndTurn);turn.result.unwrap();
    let second=combat.public_full().unwrap();
    assert_eq!(second.player.powers.iter().find(|power|power.id.0=="strength").map(|power|power.amount),Some(4));
    assert!(second.enemies.iter().all(|enemy|enemy.powers.iter().any(|power|power.id.0=="strength"&&power.amount==2)));
    let ordered=turn.effects.iter().filter_map(|event|match (&event.target,&event.kind){
        (Some(target),ResolvedEffectKind::Power{id:crate::combat::FullPowerId("strength"),before:Some(before),after:Some(after)})
            if (*before,*after)==(1,2)||(*before,*after)==(2,4)=>Some(target.clone()),_=>None,
    }).collect::<Vec<_>>();
    assert_eq!(ordered,vec![Actor::Enemy(EnemyHandle::parse("enemy-2").unwrap()),
        Actor::Enemy(EnemyHandle::parse("enemy-1").unwrap()),Actor::Player],
        "source inserts player then each current monster at top, so actual dispatch reverses all three");
}

#[test]
fn m10_ironclad_relic_turns_mark_of_pain_transfers_permanent_energy_and_inserts_two_temporary_wounds_after_opening_hand(){
    let mut run=run_with_relic(RelicId::MarkOfPain);
    let mut combat=enter(&mut run,entry(5,5,1,70));
    let start=combat.public_full().unwrap();
    assert_eq!(start.player.max_energy,4,"onEquip master energy persists into every actual combat");
    assert_eq!(start.player.energy,4);
    assert_eq!(start.hand.len(),5,"the real nonzero opening hand is fixed before battle-start relic work");
    assert!(start.hand.iter().all(|card|card.id==CardId::Strike),"neither generated Wound can enter the already-dealt opening hand");
    assert_eq!(start.draw_count,2,"an exhausted original draw pile receives exactly two temporary Wounds");
    combat.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::Draw(2)}],None).unwrap();
    let drawn=combat.public_full().unwrap();
    let wounds=drawn.hand.iter().filter(|card|card.id==CardId::Wound).collect::<Vec<_>>();
    assert_eq!(wounds.len(),2,"two distinct Wounds are added to draw, not the hand or permanent deck");
    assert_ne!(wounds[0].instance,wounds[1].instance);
    assert_eq!(combat.item_context().unwrap().relics(),&[RelicId::BurningBlood,RelicId::MarkOfPain]);
    assert!(run.state().relics.is_empty(),"permanent relic ownership is held by the active combat context, not duplicated in RunState");
    combat.step_full(&CombatAction::EndTurn).unwrap();
    let next=combat.public_full().unwrap();
    assert_eq!(next.player.max_energy,4,"Mark's master-energy bonus is one persistent equip change, not a recurring turn gain");
    assert_eq!(next.hand.iter().filter(|card|card.id==CardId::Wound).count(),2,"the two battle-start Wounds are not generated again on later player turns");
}

#[test]
fn m10_ironclad_relic_turns_mark_energy_overflow_rejects_before_inventory_transfer(){
    let mut run=run_with_relic(RelicId::MarkOfPain);
    let mut context=Some(CombatItemContext::take_from_run(run.state_mut_for_test()));
    let before=context.clone();
    let mut e=entry(5,1,1,70);e.setup.player.max_energy=999;e.setup.player.energy=999;
    assert!(matches!(Combat::enter_with_item_context(e,&mut context),Err(crate::combat::CombatEntryFailure::Rejected(_))));
    assert_eq!(context,before,"master-energy overflow is rejected before native transfer mutates the permanent item owner");
    assert!(context.as_ref().unwrap().owns_relic(RelicId::MarkOfPain));
}

#[test]
fn m10_ironclad_relic_turns_runic_cube_draws_once_per_positive_hp_loss_event(){
    let mut run=run_with_relic(RelicId::RunicCube);
    let mut combat=enter(&mut run,entry(12,1,1,70));
    assert!(combat.item_context().unwrap().owns_relic(RelicId::RunicCube));
    let before=combat.public_full().unwrap().hand.len();
    combat.enqueue_native_effects(vec![hp_loss(Source::Actor(Actor::Player),1),
        hp_loss(Source::Actor(Actor::Player),2)],None).unwrap();
    let after=combat.public_full().unwrap();
    assert_eq!(after.hand.len(),before+2,"two positive notifications produce two ordinary draws, not one draw per HP point");
    assert_eq!(after.player.hp,67);
}

#[test]
fn m10_ironclad_relic_turns_runic_cube_runs_on_actual_offering_hp_loss_before_card_draw(){
    let mut run=run_with_relic(RelicId::RunicCube);
    let mut e=entry(12,1,1,70);for card in &mut e.cards{card.id=CardId::Offering;card.local.exhaust_on_finish=true;}
    let mut combat=enter(&mut run,e);
    let offering=combat.public_full().unwrap().hand[0].instance.0;
    let observed=combat.step_full_observed(&CombatAction::PlayCard{instance:offering,target:None});observed.result.unwrap();
    let loss=observed.effects.iter().find(|event|matches!(event.kind,ResolvedEffectKind::Damage{category:crate::combat::DamageCategory::HpLoss,hp_lost:6,..})).unwrap();
    let draws=observed.effects.iter().filter(|event|event.sequence>loss.sequence&&matches!(event.kind,
        ResolvedEffectKind::CardMove{after:crate::combat::PublicZone::Hand,..})).collect::<Vec<_>>();
    assert_eq!(draws.len(),4,"Offering's actual 3-card payload plus one Runic Cube callback draw");
    assert_eq!(observed.effects.iter().find(|event|matches!(event.kind,ResolvedEffectKind::Energy{..})).unwrap().sequence>loss.sequence,true);
    assert_eq!(combat.public_full().unwrap().player.hp,64);
}

#[test]
fn m10_ironclad_relic_turns_runic_cube_does_not_draw_when_block_prevents_hp_loss(){
    let mut run=run_with_relic(RelicId::RunicCube);
    let mut combat=enter(&mut run,entry(12,1,1,70));
    combat.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::GainBlock(4)}],None).unwrap();
    let before=combat.public_full().unwrap();
    combat.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Enemy(before.enemies[0].instance_id.clone())),
        target:Some(Actor::Player),kind:EffectKind::Damage(DamagePayload::Fixed(4))}],None).unwrap();
    let after=combat.public_full().unwrap();
    assert_eq!(after.hand.len(),before.hand.len());
    assert_eq!(after.player.hp,before.player.hp);
    assert_eq!(after.player.block,0);
}

#[test]
fn m10_ironclad_relic_turns_runic_cube_draw_uses_existing_no_draw_and_ten_card_cap(){
    let mut run=run_with_relic(RelicId::RunicCube);
    let mut no_draw_entry=entry(12,1,1,70);
    no_draw_entry.setup.player_powers.push(crate::combat::FullPowerEntry{id:crate::combat::FullPowerId("no_draw"),
        state:crate::combat::PowerState{amount:1,skip_first:false,monster_source_grace:false}});
    let mut no_draw=enter(&mut run,no_draw_entry);
    let hand=no_draw.public_full().unwrap().hand.len();
    no_draw.enqueue_native_effects(vec![hp_loss(Source::Actor(Actor::Player),1)],None).unwrap();
    assert_eq!(no_draw.public_full().unwrap().hand.len(),hand,"the callback enters the ordinary No Draw pipeline");

    let mut capped_run=run_with_relic(RelicId::RunicCube);
    let mut capped=enter(&mut capped_run,entry(12,10,1,70));
    assert_eq!(capped.public_full().unwrap().hand.len(),10);
    let draw_before=capped.public_full().unwrap().draw_count;
    capped.enqueue_native_effects(vec![hp_loss(Source::Actor(Actor::Player),1)],None).unwrap();
    assert_eq!(capped.public_full().unwrap().hand.len(),10,"ordinary draw respects the real hand cap");
    assert_eq!(capped.public_full().unwrap().draw_count,draw_before);
}

#[test]
fn m10_ironclad_relic_turns_runic_cube_precedes_rupture_after_the_same_positive_loss(){
    let mut run=run_with_relic(RelicId::RunicCube);
    let mut e=entry(12,1,1,70);
    e.setup.player_powers.push(crate::combat::FullPowerEntry{id:crate::combat::FullPowerId("brutality"),
        state:crate::combat::PowerState{amount:1,skip_first:false,monster_source_grace:false}});
    e.setup.player_powers.push(crate::combat::FullPowerEntry{id:crate::combat::FullPowerId("rupture"),
        state:crate::combat::PowerState{amount:1,skip_first:false,monster_source_grace:false}});
    let mut combat=enter(&mut run,e);
    let observed=combat.step_full_observed(&CombatAction::EndTurn);observed.result.unwrap();
    let loss=observed.effects.iter().find(|event|matches!(event.kind,ResolvedEffectKind::Damage{category:crate::combat::DamageCategory::HpLoss,hp_lost:1,..})).unwrap();
    let cube_draw=observed.effects.iter().find(|event|event.sequence>loss.sequence&&matches!(event.kind,
        ResolvedEffectKind::CardMove{after:crate::combat::PublicZone::Hand,..})).unwrap();
    let rupture=observed.effects.iter().find(|event|event.sequence>loss.sequence&&matches!(event.kind,
        ResolvedEffectKind::Power{id:crate::combat::FullPowerId("strength"),..})).unwrap();
    assert!(loss.sequence<cube_draw.sequence&&cube_draw.sequence<rupture.sequence,
        "Runic Cube's top-queued ordinary draw resolves ahead of Rupture's top-queued Strength");
}

#[test]
fn m10_ironclad_relic_turns_lethal_self_loss_keeps_runic_cube_inventory_and_terminal_owner(){
    let mut run=run_with_relic(RelicId::RunicCube);
    let mut combat=enter(&mut run,entry(12,1,1,1));
    combat.enqueue_native_effects(vec![hp_loss(Source::Actor(Actor::Player),1)],None).unwrap();
    assert_eq!(combat.public_full().unwrap().outcome,Some(crate::combat::Outcome::Death));
    assert!(combat.item_context().unwrap().owns_relic(RelicId::RunicCube),"lethal callback/settlement cannot consume or duplicate the permanent inventory owner");
    assert_eq!(combat.public_full().unwrap().player.hp,0);
}
