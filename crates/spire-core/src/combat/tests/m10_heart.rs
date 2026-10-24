use super::super::*;
use crate::content::EnemyId;

#[test]
fn qualified_act4_entry_builds_one_real_heart_with_debilitate_as_initial_intent(){
    use crate::combat::rng::Partition;
    use crate::content::{CardId,Character,EncounterId};
    use crate::run::RunRng;
    let cards=(1..=10).map(|id|super::super::card_state::permanent(CardId::Strike,CardInstanceId(id),0)).collect::<Vec<_>>();
    let context=enemies::SetupContext{player:PlayerState{hp:80,max_hp:80,block:0,energy:3,max_energy:3,draw_per_turn:5,powers:vec![]},
        player_powers:vec![],character:Character::Ironclad,cards,opening_draw:5,encounter_visit:0};
    let entry=enemies::setup(EncounterId::CorruptHeart,4,0,&mut RunRng::new([29;32],Partition::Test,0),context).expect("qualified Heart source entry");
    assert_eq!(entry.setup.enemies.len(),1);
    assert_eq!((entry.setup.enemies[0].enemy_id,entry.setup.enemies[0].hp,entry.setup.enemies[0].max_hp),(EnemyId::CorruptHeart,750,750));
    assert!(entry.setup.enemies[0].powers.iter().any(|p|p.id.0=="beat_of_death"&&p.state.amount==1));
    assert!(entry.setup.enemies[0].powers.iter().any(|p|p.id.0=="invincible"&&p.state.amount==300));
    assert_eq!(entry.setup.enemies[0].intent,FullIntentId::HeartDebilitate);
    let combat=Combat::enter(entry).expect("Heart entry preserves first selected Debilitate without executing it");
    assert_eq!(combat.state.enemies[0].enemy_id,ActorEnemyId::Full(EnemyId::CorruptHeart));
    assert_eq!(combat.state.enemies[0].move_history.first().map(|intent|intent.as_str()),Some("heart_debilitate"));
    assert!(!combat.state.player_full_powers.iter().any(|p|matches!(p.id.0,"weak"|"vulnerable"|"frail")));
    let created=combat.state.piles.hand.iter().chain(&combat.state.piles.draw).chain(&combat.state.piles.discard)
        .filter_map(|owned|owned.full()).filter(|card|matches!(card.id,CardId::Dazed|CardId::Slimed|CardId::Wound|CardId::Burn|CardId::Void)).count();
    assert_eq!(created,0,"debilitate payloads are absent before the first enemy turn");
}

#[test]
fn heart_encounter_setup_is_not_an_ironclad_specific_enemy_guard(){
    use crate::combat::rng::Partition;
    use crate::content::{CardId,Character,EncounterId};
    use crate::run::RunRng;
    let cards=(1..=10).map(|id|super::super::card_state::permanent(CardId::Zap,CardInstanceId(id),0)).collect();
    let context=enemies::SetupContext{player:PlayerState{hp:70,max_hp:70,block:0,energy:3,max_energy:3,draw_per_turn:5,powers:vec![]},
        player_powers:vec![],character:Character::Defect,cards,opening_draw:5,encounter_visit:0};
    let entry=enemies::setup(EncounterId::CorruptHeart,4,0,&mut RunRng::new([31;32],Partition::Test,0),context)
        .expect("the captured Heart enemy behavior has no Ironclad-only requirement");
    assert_eq!(entry.setup.character,Character::Defect);
    assert_eq!(entry.setup.enemies.len(),1);
}

fn heart_fixture()->Combat{
    let mut combat=actual_heart();
    let enemy=&combat.state.enemies[0];
    let history=enemy.move_history.iter().map(|intent|match intent{ActorIntentId::Full(id)=>*id,_=>panic!("actual Heart history is fully typed")}).collect::<Vec<_>>();
    let selection=enemies::select_move(EnemyId::CorruptHeart,&history,
        enemy.full_memory.as_ref().unwrap(),enemy.full_ai_rng.as_ref().unwrap()).unwrap();
    let enemy=&mut combat.state.enemies[0];
    enemy.intent_id=ActorIntentId::Full(selection.intent);
    enemy.selected_move=enemy.intent_id.clone();
    enemy.move_history.push(ActorIntentId::Full(selection.intent));
    enemy.full_memory=Some(selection.memory);
    enemy.full_ai_rng=Some(selection.stream);
    if let Some(entry)=enemy.full_damage_entries.get_mut(&selection.intent){
        let damage=if selection.intent==FullIntentId::HeartBloodShots{2}else{40};entry.base=damage;entry.output=damage;
    }
    super::super::effects::refresh_enemy_values(&mut combat.state).unwrap();
    combat
}

fn set_heart_cursor(c:&mut Combat,selection_count:u32,buff_count:u32){
    let first_attack=if selection_count==0{None}else{
        match c.state.enemies[0].full_memory.as_ref().unwrap(){
            EnemyMemory::CorruptHeart{first_attack:Some(first),..}=>Some(*first),
            _=>Some(true),
        }
    };
    let mut history=vec![FullIntentId::HeartDebilitate];
    for selected in 0..selection_count{
        let phase=selected%3;
        let intent=match phase{
            0=>if first_attack.unwrap_or(true){FullIntentId::HeartBloodShots}else{FullIntentId::HeartEcho},
            1=>if first_attack.unwrap_or(true){FullIntentId::HeartEcho}else{FullIntentId::HeartBloodShots},
            _=>FullIntentId::HeartBuff,
        };
        history.push(intent);
    }
    let enemy=&mut c.state.enemies[0];
    let selected=*history.last().unwrap();
    enemy.intent_id=ActorIntentId::Full(selected);
    enemy.selected_move=ActorIntentId::Full(selected);
    enemy.move_history=history.into_iter().map(ActorIntentId::Full).collect();
    enemy.full_memory=Some(EnemyMemory::CorruptHeart{
        selection_count,first_attack,buff_count,
        last_buff_selection:if buff_count==0{None}else{Some(buff_count*3-1)},
    });
    super::super::effects::refresh_enemy_values(&mut c.state).unwrap();
}

pub(super) fn actual_heart()->Combat{
    actual_heart_with_card(None)
}

fn actual_heart_with_card(override_card:Option<crate::content::CardId>)->Combat{
    actual_heart_with_card_and_relics(override_card,vec![])
}

fn actual_heart_with_card_and_relics(override_card:Option<crate::content::CardId>,relics:Vec<crate::content::RelicId>)->Combat{
    use crate::combat::rng::Partition;
    use crate::content::{CardId,Character,EncounterId};
    use crate::run::RunRng;
    let cards=(1..=10).map(|id|super::super::card_state::permanent(
        if id==3{override_card.unwrap_or(CardId::Strike)}else{CardId::Strike},CardInstanceId(id),0)).collect();
    let context=enemies::SetupContext{player:PlayerState{hp:80,max_hp:80,block:0,energy:3,max_energy:3,draw_per_turn:5,powers:vec![]},
        player_powers:vec![],character:Character::Ironclad,cards,opening_draw:if override_card.is_some(){10}else{5},encounter_visit:0};
    let entry=enemies::setup(EncounterId::CorruptHeart,4,0,&mut RunRng::new([29;32],Partition::Test,0),context).unwrap();
    if relics.is_empty(){Combat::enter(entry).unwrap()}else{
        let mut items=Some(crate::items::CombatItemContext::new(relics,vec![None,None,None]));
        Combat::enter_with_item_context(entry,&mut items).unwrap()
    }
}

fn play_first_legal_card(c:&mut Combat){
    assert!(c.validate_runtime_for_test().is_ok(),"actual Heart state validates before card-use dispatch: {:?}",c.validate_runtime_for_test());
    let action=c.legal_actions_full().unwrap().into_iter().find(|action|matches!(action,CombatAction::PlayCard{..})).expect("actual entry has a playable Strike");
    c.step_full(&action).unwrap();
    assert!(c.validate_runtime_for_test().is_ok(),"actual card-use callback/disposition/Beat result validates at its settled checkpoint");
}

#[test]
fn actual_entry_starts_with_debilitate_unapplied_and_keeps_opening_draw_clean(){
    use crate::content::CardId;
    let mut c=actual_heart();let enemy=&c.state.enemies[0];
    assert_eq!(enemy.intent_id.as_str(),"heart_debilitate");
    assert_eq!(enemy.move_history.iter().map(|move_id|move_id.as_str()).collect::<Vec<_>>(),["heart_debilitate"]);
    assert!(c.state.piles.hand.iter().chain(&c.state.piles.draw).all(|owned|owned.full().is_some_and(|card|
        !matches!(card.id,CardId::Dazed|CardId::Slimed|CardId::Wound|CardId::Burn|CardId::Void))));
    assert!(!c.state.player_full_powers.iter().any(|power|matches!(power.id.0,"weak"|"vulnerable"|"frail")));
    c.step_full(&CombatAction::EndTurn).unwrap();
    assert_eq!((c.state.player_full_powers.iter().find(|power|power.id.0=="vulnerable").unwrap().state.amount,
        c.state.player_full_powers.iter().find(|power|power.id.0=="weak").unwrap().state.amount,
        c.state.player_full_powers.iter().find(|power|power.id.0=="frail").unwrap().state.amount),(2,2,2));
    assert!(matches!(c.state.enemies[0].intent_id.as_str(),"heart_blood_shots"|"heart_echo"));
    assert_eq!(c.state.piles.hand.iter().chain(&c.state.piles.draw).chain(&c.state.piles.discard).filter(|owned|owned.full().is_some_and(|card|
        matches!(card.id,CardId::Dazed|CardId::Slimed|CardId::Wound|CardId::Burn|CardId::Void))).count(),5);
}

#[test]
fn heart_runtime_validation_rejects_malformed_memory_and_history_without_mutation(){
    let mut c=actual_heart();
    assert!(c.validate_runtime_for_test().is_ok());
    c.state.enemies[0].move_history.clear();
    let before=c.state.clone();
    assert!(c.validate_runtime_for_test().is_err(),"the Heart's selected intent and typed history are owned state");
    assert_eq!(c.state,before,"validation rejects malformed restored state without repairing or mutating it");
}

#[test]
fn heart_runtime_validation_rejects_buff_execution_cursor_that_disagrees_with_buff_count(){
    let mut c=heart_fixture();
    set_heart_cursor(&mut c,6,2);
    assert!(c.validate_runtime_for_test().is_ok(),"two executed buffs are possible after two selected buff cycles");
    if let Some(EnemyMemory::CorruptHeart{last_buff_selection,..})=c.state.enemies[0].full_memory.as_mut(){
        *last_buff_selection=Some(2);
    }
    let before=c.state.clone();
    assert!(c.validate_runtime_for_test().is_err(),"buff_count=2 must point to the second selected buff, not the first");
    assert_eq!(c.state,before,"malformed restored buff memory is rejected without mutation");
}

#[test]
fn actual_after_use_callback_runs_before_cleanup_but_beat_damage_runs_after_disposition(){
    let mut c=actual_heart();
    assert!(c.validate_runtime_for_test().is_ok());
    let instance=c.state.piles.hand[0].full().unwrap().instance.0;
    c.step_full(&CombatAction::PlayCard{instance,target:Some(TargetId(1))}).unwrap();
    assert!(c.validate_runtime_for_test().is_ok(),"actual ordinary card and bottom Beat work settle to a valid checkpoint");
    let events=c.state.hook_events.iter().map(|event|event.hook).collect::<Vec<_>>();
    let after_use=events.iter().position(|hook|*hook==Hook::CardAfterUse).unwrap();
    let discarded=events.iter().position(|hook|*hook==Hook::CardDiscarded).unwrap();
    let beat_damage=events.iter().rposition(|hook|*hook==Hook::Damage).unwrap();
    assert!(after_use<discarded,"UseCard after-use notification itself remains before disposition");
    assert!(discarded<beat_damage,"bottom-queued Beat damage must not interrupt card cleanup/disposition");
    assert!(c.state.in_play.is_none());
}

#[test]
fn beat_after_use_covers_a_real_zero_cost_attack_and_a_power_card(){
    use crate::content::CardId;
    for card_id in [CardId::Anger,CardId::Inflame]{
        let mut c=actual_heart_with_card(Some(card_id));
        assert!(c.validate_runtime_for_test().is_ok());
        let action=c.legal_actions_full().unwrap().into_iter().find(|action|matches!(action,CombatAction::PlayCard{instance:3,..})).unwrap();
        c.step_full(&action).expect("the actual card use reaches its normal after-use owner");
        assert!(c.validate_runtime_for_test().is_ok());
        assert_eq!(c.state.hook_events.iter().filter(|event|event.hook==Hook::CardAfterUse).count(),1);
        assert_eq!(c.state.player.hp,79,"one real card use triggers one Beat even for a zero-cost card or Power");
        if card_id==CardId::Inflame{
            assert!(c.state.played_powers.iter().any(|owned|owned.full().is_some_and(|card|card.id==CardId::Inflame)),
                "Power disposition is complete before bottom-queued Beat damage resolves");
        }
    }
}

#[test]
fn m10_heart_initial_ai_uses_a_real_fair_coin_selection_and_owned_memory(){
    let combat=super::full_runtime::entered_combat(1,1, |_|{});
    let stream=combat.state.enemies[0].full_ai_rng.as_ref().unwrap();
    let memory=EnemyMemory::for_enemy(EnemyId::CorruptHeart).unwrap();
    let selected=enemies::select_move(EnemyId::CorruptHeart,&[FullIntentId::HeartDebilitate],&memory,stream)
        .expect("Corrupt Heart's first selected move is one of its source-qualified attacks");
    assert!(matches!(selected.intent.as_str(),"heart_blood_shots"|"heart_echo"));
    assert_eq!(selected.stream.word_index(),stream.word_index()+1,"the fair two-way choice advances only this Heart's AI stream");
    let EnemyMemory::CorruptHeart{selection_count,first_attack:Some(_),buff_count,..}=selected.memory else{panic!("first choice commits only its selected-move state")};
    assert_eq!((selection_count,buff_count),(1,0));
}

#[test]
fn heart_selects_attack_opposite_then_executes_buff_without_consuming_an_extra_coin(){
    let c=heart_fixture();let enemy=&c.state.enemies[0];
    let stream=enemy.full_ai_rng.as_ref().unwrap();
    let mut memory=EnemyMemory::for_enemy(EnemyId::CorruptHeart).unwrap();
    let mut history=vec![FullIntentId::HeartDebilitate];
    let first=enemies::select_move(EnemyId::CorruptHeart,&history,&memory,stream).unwrap();
    let first_name=first.intent.as_str();history.push(first.intent);memory=first.memory;
    let second=enemies::select_move(EnemyId::CorruptHeart,&history,&memory,&first.stream).unwrap();
    assert_ne!(first_name,second.intent.as_str(),"the paired attack is the other member of the selected pair");
    assert_eq!(second.stream.word_index(),first.stream.word_index(),"only the first attack selection consumes the coin");
    history.push(second.intent);memory=second.memory;
    let buff=enemies::select_move(EnemyId::CorruptHeart,&history,&memory,&second.stream).unwrap();
    assert_eq!(buff.intent.as_str(),"heart_buff");
    assert_eq!(buff.memory,EnemyMemory::CorruptHeart{selection_count:3,first_attack:match memory {EnemyMemory::CorruptHeart{first_attack,..}=>first_attack,_=>None},buff_count:0,last_buff_selection:None});
}

#[test]
fn heart_blood_shots_and_echo_publish_the_qualified_hit_counts_and_values(){
    let mut c=heart_fixture();
    c.state.enemies[0].intent_id=ActorIntentId::Full(FullIntentId::HeartBloodShots);
    let shots=enemies::move_work(&c.state.enemies[0]).unwrap();
    assert_eq!(shots.len(),12);
    assert!(shots.iter().all(|effect|effect.kind==EffectKind::Damage(DamagePayload::FullEnemyMove(FullIntentId::HeartBloodShots))));
    assert_eq!(c.state.enemies[0].full_damage_entries[&FullIntentId::HeartBloodShots].output,2);
    assert_eq!(c.state.enemies[0].full_damage_entries[&FullIntentId::HeartEcho].output,40);
    let mut incoming=Vec::new();
    for effect in shots {incoming.push(super::super::effects::apply_effect_with_result(&mut c.state,&effect).unwrap().unwrap().attempted);}
    assert_eq!(incoming,vec![2;12],"each real Blood Shot hit has its own two-damage base");
    assert_eq!(c.state.player.hp,56,"the actual eighty-HP setup loses twelve two-damage hits");
    let mut echo=heart_fixture();echo.state.player.hp=80;
    echo.state.enemies[0].intent_id=ActorIntentId::Full(FullIntentId::HeartEcho);
    let work=enemies::move_work(&echo.state.enemies[0]).unwrap();
    super::super::effects::apply_effect(&mut echo.state,&work[0]).unwrap();
    assert_eq!(echo.state.player.hp,40,"Echo is one actual forty-damage hit");
}

#[test]
fn the_real_monster_turn_dispatches_both_blood_shots_through_the_shared_queue(){
    let mut c=heart_fixture();let owner=c.state.enemies[0].instance_id.clone();
    assert!(c.validate_runtime_for_test().is_ok());
    let hp=c.state.player.hp;
    c.state.pending.push_back(Effect{source:Source::None,target:None,kind:EffectKind::Decision});
    enemies::act(&mut c.state,&owner).expect("the Heart has a real enemy-turn dispatcher");
    let selected=c.state.enemies[0].intent_id.as_str();
    let hits=c.state.pending.iter().filter(|effect|matches!(effect.kind,EffectKind::Damage(DamagePayload::FullEnemyMove(_)))).count();
    assert_eq!(hits,if selected=="heart_blood_shots"{12}else{1});
    super::super::engine::resolve(&mut c.state);
    assert!(c.validate_runtime_for_test().is_ok(),"actual selected attack resolves to the next legal decision");
    let amount=if selected=="heart_blood_shots"{24}else{40};
    assert_eq!(c.state.player.hp,hp-amount,"the selected real Heart attack reaches the shared damage resolver");
}

#[test]
fn invincible_caps_committed_hp_after_block_for_normal_thorns_and_hp_loss(){
    for (payload,block) in [(DamagePayload::Fixed(400),10),(DamagePayload::Thorns(400),0),(DamagePayload::HpLoss(400),99)]{
        let mut c=heart_fixture();c.state.enemies[0].block=block;
        let before=c.state.enemies[0].hp;
        let effect=Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Enemy(c.state.enemies[0].instance_id.clone())),kind:EffectKind::Damage(payload)};
        let receipt=super::super::effects::apply_effect_with_result(&mut c.state,&effect).expect("typed incoming damage resolves");
        assert_eq!(c.state.enemies[0].hp,before-300,"quota caps actual HP loss after Block");
        assert_eq!(c.state.enemies[0].block,if block==99{99}else{0},"HP_LOSS bypasses Block; ordinary damage spends it first");
        assert_eq!(c.state.enemies[0].full_powers.iter().find(|p|p.id.0=="invincible").unwrap().state.amount,0);
        assert_eq!(receipt.unwrap().hp_lost,300,"the committed receipt reports capped loss, not the uncapped request");
    }
}

#[test]
fn invincible_charges_only_unblocked_committed_hp_per_hit(){
    let mut c=heart_fixture();let owner=c.state.enemies[0].instance_id.clone();
    c.state.enemies[0].full_powers.iter_mut().find(|p|p.id.0=="invincible").unwrap().state.amount=10;
    c.state.enemies[0].block=20;
    let hit=|damage|Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Enemy(owner.clone())),kind:EffectKind::Damage(DamagePayload::Fixed(damage))};
    let before=c.state.enemies[0].hp;
    let blocked=super::super::effects::apply_effect_with_result(&mut c.state,&hit(15)).unwrap().unwrap();
    assert_eq!((blocked.hp_lost,blocked.blocked,c.state.enemies[0].full_powers.iter().find(|p|p.id.0=="invincible").unwrap().state.amount),(0,15,10));
    c.state.enemies[0].block=5;
    let first=super::super::effects::apply_effect_with_result(&mut c.state,&hit(9)).unwrap().unwrap();
    assert_eq!((first.hp_lost,first.blocked),(4,5));
    assert_eq!(c.state.enemies[0].full_powers.iter().find(|p|p.id.0=="invincible").unwrap().state.amount,6);
    let second=super::super::effects::apply_effect_with_result(&mut c.state,&hit(50)).unwrap().unwrap();
    assert_eq!((second.hp_lost,second.attempted,c.state.enemies[0].hp),(6,6,before-10));
    let last=super::super::effects::apply_effect_with_result(&mut c.state,&hit(50)).unwrap().unwrap();
    assert_eq!((last.hp_lost,last.attempted,c.state.enemies[0].hp),(0,0,before-10));
}

#[test]
fn invincible_spends_the_capped_post_block_callback_value_before_overkill_hp_clamp(){
    let mut c=heart_fixture();let owner=c.state.enemies[0].instance_id.clone();
    c.state.enemies[0].hp=2;
    c.state.enemies[0].full_powers.iter_mut().find(|power|power.id.0=="invincible").unwrap().state.amount=20;
    let lethal=Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Enemy(owner)),kind:EffectKind::Damage(DamagePayload::Fixed(50))};
    let receipt=super::super::effects::apply_effect_with_result(&mut c.state,&lethal).unwrap().unwrap();
    assert_eq!((receipt.attempted,receipt.hp_lost),(20,2));
    assert_eq!(c.state.enemies[0].full_powers.iter().find(|power|power.id.0=="invincible").unwrap().state.amount,0,
        "Invincible receives and spends the post-Block capped callback amount, not post-clamp HP lost");
}

#[test]
fn actual_reaper_hp_receipt_and_charons_ashes_respect_invincible_and_full_block(){
    use crate::combat::observe::{DamageCategory,ResolvedEffectKind};
    use crate::content::{CardId,RelicId};
    for (quota,block,expected_reaper_loss,expected_ashes_loss,expected_player_hp) in
        [(7,0,4,3,23),(0,7,0,0,19)] {
        let mut c=actual_heart_with_card_and_relics(Some(CardId::Reaper),vec![RelicId::CharonsAshes]);
        c.state.player.hp=20;
        let heart=c.state.enemies[0].instance_id.clone();
        c.state.enemies[0].block=block;
        c.state.enemies[0].full_powers.iter_mut().find(|power|power.id.0=="invincible").unwrap().state.amount=quota;
        assert!(c.validate_runtime_for_test().is_ok(),"the real Reaper/Charon's Ashes setup is valid before the action");
        let action=c.legal_actions_full().unwrap().into_iter().find(|action|matches!(action,CombatAction::PlayCard{instance:3,..}))
            .expect("actual Reaper is available in the Heart opening hand");
        let session=super::super::observe::Session::combat(&c);
        let result=c.step_full(&action);
        let observed=session.finish(result);
        assert!(observed.result.is_ok(),"real Reaper and relic listeners resolve normally: {:?}",observed.result);
        let hits=observed.effects.iter().filter_map(|effect|match (&effect.target,&effect.kind){
            (Some(Actor::Enemy(owner)),ResolvedEffectKind::Damage{hp_lost,category,..}) if owner==&heart=>Some((*hp_lost,*category,effect.source.clone())),
            _=>None,
        }).collect::<Vec<_>>();
        assert_eq!(hits.iter().filter(|(_,category,_)|*category==DamageCategory::Normal).map(|(loss,_,_)|*loss).sum::<i32>(),expected_reaper_loss,
            "the actual Reaper receipt uses post-Block capped HP lost");
        let ashes=hits.iter().find(|(_,category,source)|*category==DamageCategory::Thorns&&*source==Source::None)
            .expect("actual Charon's Ashes listener emits its source-null THORNS hit");
        assert_eq!(ashes.0,expected_ashes_loss,"the real listener's observed hp_lost obeys remaining quota and Block");
        let heals=observed.effects.iter().filter_map(|effect|match effect.kind{ResolvedEffectKind::Heal{amount,..}=>Some(amount),_=>None}).sum::<i32>();
        assert_eq!(heals,expected_reaper_loss,"earned Reaper healing follows actual hp_lost, not attempted/capped damage or overkill");
        assert_eq!(c.state.player.hp,expected_player_hp,"actual Reaper heal and one post-disposition Beat resolve in sequence");
        assert_eq!(c.state.enemies[0].block,0,"Reaper and Charon's Ashes consume the available block in the full-block case");
        assert!(c.validate_runtime_for_test().is_ok(),"the actual listener composition leaves a valid quiescent Heart state");
    }
}

#[test]
fn invincible_resets_at_monster_start_and_only_that_checkpoint(){
    let mut c=heart_fixture();
    c.state.enemies[0].full_powers.iter_mut().find(|p|p.id.0=="invincible").unwrap().state.amount=0;
    let before=c.state.player.hp;
    super::super::effects::apply_effect(&mut c.state,&Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::Hook(Hook::TurnStarted)}).unwrap();
    assert_eq!(c.state.enemies[0].full_powers.iter().find(|p|p.id.0=="invincible").unwrap().state.amount,0,"a player-start checkpoint must not refill the Heart quota");
    assert_eq!(c.state.player.hp,before);
    super::super::effects::apply_effect(&mut c.state,&Effect{source:Source::None,target:None,kind:EffectKind::MonsterStart}).unwrap();
    assert_eq!(c.state.enemies[0].full_powers.iter().find(|p|p.id.0=="invincible").unwrap().state.amount,300);
}

#[test]
fn forged_after_use_hook_cannot_trigger_heart_retaliation(){
    let mut c=heart_fixture();let card=c.state.piles.hand[0].handle();let hp=c.state.player.hp;
    let result=c.enqueue_hook(HookEvent{hook:Hook::CardAfterUse,source:Source::Card(card),target:None});
    assert!(result.is_err(),"only an actual in-play card continuation may reach the private after-use checkpoint");
    assert_eq!(c.state.player.hp,hp);
    assert!(c.state.pending.is_empty());
}

#[test]
fn an_actual_successful_card_use_triggers_beat_after_its_payload(){
    let mut c=heart_fixture();let before=c.state.player.hp;
    assert!(c.validate_runtime_for_test().is_ok());
    let heart_hp=c.state.enemies[0].hp;
    let action=c.legal_actions_full().unwrap().into_iter().find(|action|matches!(action,CombatAction::PlayCard{..})).unwrap();
    let played=match action{CombatAction::PlayCard{instance,..}=>instance,_=>unreachable!()};
    c.step_full(&action).expect("legal Strike resolves fully");
    assert!(c.validate_runtime_for_test().is_ok());
    assert!(c.state.hook_events.iter().any(|event|event.hook==Hook::CardAfterUse),"actual use must reach the post-payload hook");
    assert_eq!(c.state.enemies[0].full_powers.iter().find(|power|power.id.0=="beat_of_death").unwrap().state.amount,1);
    assert_eq!(c.state.player.hp,before-1,"the after-use callback follows the actual attack payload");
    assert_eq!(c.state.enemies[0].hp,heart_hp-6,"the real card damage commits before Beat of Death");
    assert!(c.state.hook_events.iter().any(|event|event.hook==Hook::CardAfterUse&&event.source==Source::Card(CardHandle::parse(&played.to_string()).unwrap())));
}

#[test]
fn beat_waits_for_a_nested_selection_and_runs_once_for_the_completed_use(){
    let mut c=actual_heart_with_card(Some(crate::content::CardId::BurningPact));
    assert!(c.validate_runtime_for_test().is_ok());
    let hp=c.state.player.hp;
    let action=c.legal_actions_full().unwrap().into_iter().find(|action|matches!(action,CombatAction::PlayCard{instance:3,..})).unwrap();
    c.step_full(&action).expect("Burning Pact opens its real card selection");
    assert!(c.validate_runtime_for_test().is_ok(),"the suspended actual selection owner validates before Beat");
    assert_eq!(c.state.phase,Phase::Selection);
    assert_eq!(c.state.player.hp,hp,"the use checkpoint remains after its nested selection");
    c.step_full(&CombatAction::SelectCard{instance:2}).expect("the selected hand card exhausts before after-use");
    assert!(c.validate_runtime_for_test().is_ok(),"selected card disposition and one Beat settle normally");
    assert_eq!(c.state.player.hp,hp-1,"the completed card use triggers one Beat callback");
    assert_eq!(c.state.hook_events.iter().filter(|event|event.hook==Hook::CardAfterUse).count(),1);
}

#[test]
fn fully_blocked_attack_still_triggers_one_beat_for_that_use(){
    let mut c=heart_fixture();c.state.player.block=5;let hp=c.state.player.hp;
    play_first_legal_card(&mut c);
    assert_eq!((c.state.player.hp,c.state.player.block),(hp,4));
    assert_eq!(c.state.hook_events.iter().filter(|event|event.hook==Hook::CardAfterUse).count(),1);
}

#[test]
fn beat_dispatches_once_for_each_actual_double_tap_use_not_once_for_the_outer_request(){
    let mut c=heart_fixture();
    assert!(c.validate_runtime_for_test().is_ok());
    c.state.player_full_powers.push(FullPowerEntry{id:FullPowerId("double_tap"),state:PowerState{amount:1,skip_first:false,monster_source_grace:false}});
    let action=c.legal_actions_full().unwrap().into_iter().find(|action|matches!(action,CombatAction::PlayCard{..})).unwrap();
    c.step_full(&action).expect("the real repeated Strike has two accepted card-use frames");
    assert!(c.validate_runtime_for_test().is_ok());
    assert_eq!(c.state.hook_events.iter().filter(|event|event.hook==Hook::CardAfterUse).count(),2,
        "the outer request does not synthesize an extra callback beyond the two real uses");
    assert_eq!(c.state.player.hp,78,"each actual use triggers exactly one Beat damage from the eighty-HP setup");
}

#[test]
fn beat_composes_with_real_havoc_child_attack_and_whirlwind_multihit(){
    use crate::combat::observe::{DamageCategory,ResolvedEffectKind};
    use crate::content::CardId;
    let mut havoc=actual_heart_with_card(Some(CardId::Havoc));
    let source=havoc.state.piles.hand.iter().find(|owned|owned.full().is_some_and(|card|card.id==CardId::Havoc)).unwrap().handle();
    let child_index=havoc.state.piles.hand.iter().position(|owned|owned.full().is_some_and(|card|card.id==CardId::Strike)).unwrap();
    let child=havoc.state.piles.hand.remove(child_index);
    havoc.state.piles.draw.push(child);
    assert!(havoc.validate_runtime_for_test().is_ok(),"Havoc's valid top-child state has exact owned-card custody");
    let action=havoc.legal_actions_full().unwrap().into_iter().find(|action|matches!(action,CombatAction::PlayCard{instance:3,..})).unwrap();
    let session=super::super::observe::Session::combat(&havoc);
    let result=havoc.step_full(&action);
    let observed=session.finish(result);
    assert!(observed.result.is_ok(),"actual Havoc child and parent complete normally: {:?}",observed.result);
    assert!(havoc.validate_runtime_for_test().is_ok(),"actual child and parent cleanup reach a validated decision");
    let uses=havoc.state.hook_events.iter().filter(|event|event.hook==Hook::CardAfterUse).collect::<Vec<_>>();
    assert_eq!(uses.len(),2,"the parent Havoc and its separately played Strike child each own one callback");
    assert!(uses.iter().any(|event|event.source==Source::Card(source.clone())));
    assert_eq!(havoc.state.player.hp,78,"two actual uses cause two Beats, not per-hit retaliation");
    assert!(havoc.state.piles.discard.iter().any(|owned|owned.handle()==source&&owned.full().is_some_and(|card|card.id==CardId::Havoc)),
        "Havoc's ordinary parent cleanup completes before its bottom-queued Beat work");

    let mut suppressed=actual_heart_with_card(Some(CardId::Havoc));
    let status=super::super::card_state::permanent(CardId::Dazed,CardInstanceId(99),0);
    suppressed.state.card_ids_issued.insert(super::super::card_state::handle(CardInstanceId(99)));
    suppressed.state.piles.draw.push(super::super::model::OwnedCard::Full(status));
    assert!(suppressed.validate_runtime_for_test().is_ok(),"the extracted Status is a valid owned top card before Havoc dispatch");
    let action=suppressed.legal_actions_full().unwrap().into_iter().find(|action|matches!(action,CombatAction::PlayCard{instance:3,..})).unwrap();
    suppressed.step_full(&action).expect("Havoc completes while suppressing the unplayable Status child");
    assert!(suppressed.validate_runtime_for_test().is_ok(),"failed Status child retains valid cleanup and parent callback custody");
    let callbacks=suppressed.state.hook_events.iter().filter(|event|event.hook==Hook::CardAfterUse).collect::<Vec<_>>();
    assert_eq!(callbacks.len(),1,"the failed Status child emits no after-use callback; only its real Havoc parent does");
    assert_eq!(callbacks[0].source,Source::Card(super::super::card_state::handle(CardInstanceId(3))));
    assert_eq!(suppressed.state.player.hp,79,"one completed parent use still causes exactly one Beat");
    assert!(suppressed.state.in_play.is_none()&&suppressed.state.play_frames.is_empty());

    let mut whirlwind=actual_heart_with_card(Some(CardId::Whirlwind));
    let heart=whirlwind.state.enemies[0].instance_id.clone();
    let before=whirlwind.state.enemies[0].hp;
    let action=whirlwind.legal_actions_full().unwrap().into_iter().find(|action|matches!(action,CombatAction::PlayCard{instance:3,..})).unwrap();
    let session=super::super::observe::Session::combat(&whirlwind);
    let result=whirlwind.step_full(&action);
    let observed=session.finish(result);
    assert!(observed.result.is_ok(),"actual X-cost Whirlwind reaches its native multi-hit path: {:?}",observed.result);
    assert!(whirlwind.validate_runtime_for_test().is_ok(),"multi-hit card and single Beat callback settle to a valid decision");
    let hits=observed.effects.iter().filter(|effect|effect.target==Some(Actor::Enemy(heart.clone()))
        &&matches!(effect.kind,ResolvedEffectKind::Damage{category:DamageCategory::Normal,..})).count();
    assert!(hits>=2,"the actual X-cost card produced multiple separately observed damage hits");
    assert_eq!(whirlwind.state.hook_events.iter().filter(|event|event.hook==Hook::CardAfterUse).count(),1,
        "multiple native hits remain one card use and one Beat callback");
    assert_eq!(whirlwind.state.player.hp,79);
    assert!(whirlwind.state.enemies[0].hp<before);
}

#[test]
fn accepted_after_use_fault_keeps_beat_owner_and_unfinished_card_without_retry(){
    let mut c=heart_fixture();c.state.damage_events_this_combat=u32::MAX;
    assert!(c.validate_runtime_for_test().is_ok(),"the retained damage-event overflow is a valid pre-command fault setup");
    let hp=c.state.player.hp;
    let action=c.legal_actions_full().unwrap().into_iter().find(|action|matches!(action,CombatAction::PlayCard{..})).unwrap();
    let result=c.step_full(&action);
    assert!(result.is_err(),"the actual Beat damage reaches the existing retained damage-event fault");
    assert_eq!(c.state.player.hp,hp,"overflow preflight is before Beat's HP commit");
    assert_eq!(c.state.current.as_ref().map(|effect|effect.kind.clone()),Some(EffectKind::Damage(DamagePayload::Thorns(1))));
    assert_eq!(c.state.current.as_ref().unwrap().source,Source::Actor(Actor::Enemy(c.state.enemies[0].instance_id.clone())));
    assert!(c.state.in_play.is_none(),"the after-use callback is earlier than ordinary card disposition");
    assert!(!c.state.pending.iter().any(|effect|effect.kind==EffectKind::FinishCard),"the continuation already completed before bottom-queued Beat faults");
    assert!(c.validate_runtime_for_test().is_ok(),"the accepted Beat fault retains a valid owner/checkpoint state");
}

#[test]
fn already_dispatched_lethal_beat_uses_ordinary_death_and_stops_finish_card(){
    let mut c=heart_fixture();c.state.player.hp=1;
    play_first_legal_card(&mut c);
    assert_eq!(c.state.outcome,Some(Outcome::Death));
    assert!(c.state.pending.is_empty());
    assert!(c.state.in_play.is_none(),"ordinary disposition completed before bottom-queued Beat resolves");
    assert!(c.state.hook_events.iter().any(|event|event.hook==Hook::CardAfterUse));
}

#[test]
fn lethal_heart_damage_immediately_clears_undispatched_beat_but_never_suppresses_its_producer(){
    let mut c=heart_fixture();c.state.enemies[0].hp=1;c.state.enemies[0].full_powers.iter_mut().find(|p|p.id.0=="invincible").unwrap().state.amount=300;
    let heart=c.state.enemies[0].instance_id.clone();let hp_before=c.state.player.hp;
    let queued=Effect{source:Source::Actor(Actor::Enemy(heart.clone())),target:Some(Actor::Player),kind:EffectKind::Damage(DamagePayload::Thorns(1))};
    c.state.pending.push_back(queued);
    let lethal=Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Enemy(heart)),kind:EffectKind::Damage(DamagePayload::Fixed(1))};
    c.state.pending.push_front(lethal);
    super::super::engine::resolve(&mut c.state);
    assert_eq!(c.state.outcome,Some(Outcome::Victory));
    assert!(c.state.pending.is_empty(),"existing headless terminal cutoff drops still-undispatched after-use retaliation");
    assert_eq!(c.state.player.hp,hp_before,"this clockless cutoff is not simulated as a dying-source THORNS callback");
}

#[test]
fn already_owned_beat_payload_remains_valid_for_heart_source_before_terminal_queue_cutoff(){
    let mut c=heart_fixture();let owner=c.state.enemies[0].instance_id.clone();
    c.state.enemies[0].hp=0;
    let beat=Effect{source:Source::Actor(Actor::Enemy(owner)),target:Some(Actor::Player),kind:EffectKind::Damage(DamagePayload::Thorns(1))};
    super::super::engine::validate_effect(&beat,&c.state).expect("the immediate settlement mapping, not a dying-source predicate, clears this undispatched payload");
}

#[test]
fn heart_buff_runs_its_actual_order_and_only_compensates_negative_strength(){
    let mut c=heart_fixture();
    set_heart_cursor(&mut c,3,0);
    let first_attack=match c.state.enemies[0].full_memory.as_ref().unwrap(){EnemyMemory::CorruptHeart{first_attack,..}=>*first_attack,_=>None};
    assert!(c.validate_runtime_for_test().is_ok(),"canonical first selected buff state validates before execution");
    c.state.enemies[0].full_powers.push(FullPowerEntry{id:FullPowerId("strength"),state:PowerState{amount:-4,skip_first:false,monster_source_grace:false}});
    let owner=c.state.enemies[0].instance_id.clone();let actor=Actor::Enemy(owner.clone());
    let work=Effect{source:Source::Actor(actor.clone()),target:Some(actor.clone()),kind:EffectKind::CorruptHeartBuff};
    super::super::effects::apply_effect(&mut c.state,&work).expect("selected Heart buff is valid owner work");
    assert_eq!(c.state.pending.iter().map(|effect|effect.kind.clone()).collect::<Vec<_>>(),vec![
        EffectKind::ApplyFullPower{id:FullPowerId("strength"),amount:6},
        EffectKind::ApplyFullPower{id:FullPowerId("artifact"),amount:2},
    ]);
    for _ in 0..2{let child=c.state.pending.pop_front().unwrap();super::super::effects::apply_effect(&mut c.state,&child).unwrap();}
    assert!(c.validate_runtime_for_test().is_ok(),"completed first buff and its marker validate at the next quiescent checkpoint");
    assert_eq!(c.state.enemies[0].full_powers.iter().find(|p|p.id.0=="strength").unwrap().state.amount,2);
    assert_eq!(c.state.enemies[0].full_powers.iter().find(|p|p.id.0=="artifact").unwrap().state.amount,2);
    assert_eq!(c.state.enemies[0].full_memory,Some(EnemyMemory::CorruptHeart{selection_count:3,first_attack,buff_count:1,last_buff_selection:Some(2)}));

    c.state.enemies[0].full_powers.iter_mut().find(|p|p.id.0=="strength").unwrap().state.amount=7;
    super::super::effects::refresh_enemy_values(&mut c.state).unwrap();
    set_heart_cursor(&mut c,6,1);
    assert!(c.validate_runtime_for_test().is_ok(),"second selected buff follows one actually executed buff: {:?}",c.validate_runtime_for_test());
    let work=Effect{source:Source::Actor(actor.clone()),target:Some(actor),kind:EffectKind::CorruptHeartBuff};
    super::super::effects::apply_effect(&mut c.state,&work).expect("later buff remains selected owner work");
    assert!(!c.state.pending.iter().any(|effect|effect.kind==EffectKind::RemoveFullPower(FullPowerId("strength"))));
    while let Some(child)=c.state.pending.pop_front(){super::super::effects::apply_effect(&mut c.state,&child).unwrap();}
    assert!(c.validate_runtime_for_test().is_ok(),"second buff sequence validates after its queued children resolve");
}

#[test]
fn a_selected_heart_buff_cannot_be_dispatched_twice(){
    let mut c=heart_fixture();let actor=Actor::Enemy(c.state.enemies[0].instance_id.clone());
    set_heart_cursor(&mut c,3,0);
    let first_attack=match c.state.enemies[0].full_memory.as_ref().unwrap(){EnemyMemory::CorruptHeart{first_attack,..}=>*first_attack,_=>None};
    assert!(c.validate_runtime_for_test().is_ok());
    let effect=Effect{source:Source::Actor(actor.clone()),target:Some(actor),kind:EffectKind::CorruptHeartBuff};
    super::super::effects::apply_effect(&mut c.state,&effect).unwrap();
    let before=c.state.pending.len();
    assert!(super::super::effects::apply_effect(&mut c.state,&effect).is_err(),"one installed selection has only one execution owner");
    assert_eq!(c.state.pending.len(),before);
    assert_eq!(c.state.enemies[0].full_memory,Some(EnemyMemory::CorruptHeart{selection_count:3,first_attack,buff_count:1,last_buff_selection:Some(2)}));
    while let Some(child)=c.state.pending.pop_front(){super::super::effects::apply_effect(&mut c.state,&child).unwrap();}
    assert!(c.validate_runtime_for_test().is_ok(),"single selected buff execution is valid after its owned children resolve");
}

#[test]
fn heart_buff_history_adds_the_qualified_cumulative_power_sequence(){
    let mut c=heart_fixture();let owner=c.state.enemies[0].instance_id.clone();let actor=Actor::Enemy(owner.clone());
    for (index,want_strength,want_artifact,want_beat,want_stabs) in [
        (0,2,2,1,0),(1,4,2,2,0),(2,6,2,2,1),(3,18,2,2,1),(4,70,2,2,1),
    ] {
        set_heart_cursor(&mut c,(index+1)*3,index);
        assert!(c.validate_runtime_for_test().is_ok(),"selection/buff history is valid before ordinal {index} executes: {:?}",c.validate_runtime_for_test());
        let effect=Effect{source:Source::Actor(actor.clone()),target:Some(actor.clone()),kind:EffectKind::CorruptHeartBuff};
        super::super::effects::apply_effect(&mut c.state,&effect).expect("selected buff enqueues owned children");
        let mut resolved=0;
        while let Some(child)=c.state.pending.pop_front(){super::super::effects::apply_effect(&mut c.state,&child).expect("qualified Heart power child resolves");resolved+=1;}
        assert!(c.validate_runtime_for_test().is_ok(),"selection and executed ordinal remain valid after ordinal {index}");
        assert!(resolved>=2,"each buff contains its Strength addition and sequential additional power");
        let amount=|id|c.state.enemies[0].full_powers.iter().find(|p|p.id.0==id).map_or(0,|p|p.state.amount);
        assert_eq!((amount("strength"),amount("artifact"),amount("beat_of_death"),amount("painful_stabs")),(want_strength,want_artifact,want_beat,want_stabs));
    }
}
