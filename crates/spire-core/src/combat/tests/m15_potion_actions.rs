use crate::combat::{Actor, Combat, CombatAction, CombatEntry, CombatCardRef,
    DamagePayload, EffectKind, EnemyHandle, FullCombatSetup, FullEnemySetup, FullIntentId,
    FullPowerEntry, Hook, LiveCard, PlayerState, PowerState, Source};
use crate::combat::rng::{Domain, StreamId, StreamState};
use crate::content::{CardId, Character, EnemyId, PotionId, RelicId};
use crate::items::CombatItemContext;

fn power(id: &'static str, amount: i32) -> FullPowerEntry {
    FullPowerEntry { id: crate::combat::FullPowerId(id), state: PowerState {
        amount, skip_first: false, monster_source_grace: false,
    } }
}

fn fight(cards: &[CardId], opening_draw: usize, potion: PotionId, relics: Vec<RelicId>,
    enemy_count: usize, enemy_powers: &[FullPowerEntry], player_powers: &[FullPowerEntry]) -> Combat {
    let scope = format!("m15-potion-actions-{potion:?}-{opening_draw}-{enemy_count}");
    let shuffle_id = StreamId::new(Domain::RunCombatShuffle, vec![scope.clone()]).unwrap();
    let card_id = StreamId::new(Domain::RunCombatCardRandom, vec![scope.clone()]).unwrap();
    let enemies = (0..enemy_count).map(|index| {
        let owner = EnemyHandle::parse(&format!("enemy-{}", index + 1)).unwrap();
        let ai_id = StreamId::new(Domain::RunCombatEnemyAi,
            vec![scope.clone(), owner.as_str().to_owned()]).unwrap();
        FullEnemySetup { initial_memory: None, instance_id: owner, ascension: 0,
            profile: crate::combat::enemies::EnemyProfile::Standard, enemy_id: EnemyId::Cultist,
            hp: 48, max_hp: 48, block: 0, intent: FullIntentId::Incantation,
            powers: enemy_powers.to_vec(), ai_rng: StreamState::new(ai_id, [0x53; 32], 0).unwrap() }
    }).collect();
    let live_cards = cards.iter().enumerate().map(|(index, id)| {
        let instance = crate::combat::CardInstanceId(index as u64 + 1);
        let live = crate::combat::card_state::permanent(*id, instance, 0);
        LiveCard { misc:0, instance, id: *id, origin: CombatCardRef::Permanent(instance), upgrades: 0,
            local: live.local, copy_family: instance.0,
            copy_provenance: crate::combat::card_state::CopyProvenance::Original }
    }).collect();
    let entry = CombatEntry { setup: FullCombatSetup { enemy_damage_bases: Default::default(),
        player: PlayerState { hp: 35, max_hp: 80, block: 0, energy: 3, max_energy: 3,
            draw_per_turn: 5, powers: vec![] }, player_powers: player_powers.to_vec(),
        character: Character::Silent,
        shuffle_rng: StreamState::new(shuffle_id, [0x51; 32], 0).unwrap(),
        card_rng: StreamState::new(card_id, [0x52; 32], 0).unwrap(), enemies },
        opening_draw, cards: live_cards };
    let mut owner = Some(CombatItemContext::new(relics, vec![Some(potion)]).with_m15_profile());
    let combat = Combat::enter_with_item_context(entry, &mut owner).unwrap();
    assert!(owner.is_none(), "combat takes the inventory owner once");
    combat
}

fn draw_to(combat: &mut Combat, instance: u64, destination: &str) {
    let index = combat.state.piles.draw.iter().position(|card|
        card.full().is_some_and(|live| live.instance.0 == instance)).unwrap();
    let card = combat.state.piles.draw.remove(index);
    match destination {
        "hand" => combat.state.piles.hand.push(card),
        "discard" => combat.state.piles.discard.push(card),
        _ => unreachable!(),
    }
}

fn place_on_top(combat:&mut Combat,instance:u64){
    let index=combat.state.piles.draw.iter().position(|card|
        card.full().is_some_and(|live|live.instance.0==instance)).unwrap();
    let card=combat.state.piles.draw.remove(index);combat.state.piles.draw.push(card);
}

fn use_potion(combat: &mut Combat) {
    combat.step_full(&CombatAction::UsePotion { slot: 0, target: None }).unwrap();
    assert_eq!(combat.item_context().unwrap().potions()[0],None,"a successful use consumes its actual belt slot");
}

#[test]
fn m15_explosive_potion_is_null_source_normal_aoe_with_enemy_receive_modifiers() {
    let mut combat = fight(&[CardId::StrikeGreen], 0, PotionId::ExplosivePotion, vec![], 2,
        &[power("vulnerable", 1)], &[power("strength", 5), power("weak", 2)]);
    let prepared = crate::items::prepare_combat_potion(&combat, 0, None, false).unwrap();
    assert_eq!(prepared.effects.len(), 2);
    for effect in &prepared.effects {
        assert_eq!(effect.source, Source::None);
        assert!(matches!(effect.kind, EffectKind::Damage(DamagePayload::Fixed(15))));
    }
    use_potion(&mut combat);
    assert_eq!(combat.state.enemies.iter().map(|enemy| enemy.hp).collect::<Vec<_>>(), vec![33, 33]);
    assert_eq!(combat.state.player_full_powers.iter().find(|entry| entry.id.0 == "strength").unwrap().state.amount, 5);
    assert_eq!(combat.state.player_full_powers.iter().find(|entry| entry.id.0 == "weak").unwrap().state.amount, 2);

    let mut bark=fight(&[CardId::StrikeGreen],0,PotionId::ExplosivePotion,vec![RelicId::SacredBark],2,
        &[power("vulnerable",1)],&[power("strength",5),power("weak",2)]);
    let prepared=crate::items::prepare_combat_potion(&bark,0,None,false).unwrap();
    assert!(prepared.effects.iter().all(|effect|matches!(effect.kind,EffectKind::Damage(DamagePayload::Fixed(30)))));
    use_potion(&mut bark);
    assert_eq!(bark.state.enemies.iter().map(|enemy|enemy.hp).collect::<Vec<_>>(),vec![18,18]);
}

#[test]
fn m15_gamblers_brew_optional_manual_discard_draws_only_the_chosen_count() {
    let cards = [CardId::StrikeGreen, CardId::DefendGreen, CardId::DaggerThrow,
        CardId::StrikeGreen, CardId::DefendGreen, CardId::StrikeGreen];
    let mut zero = fight(&cards, 4, PotionId::GamblersBrew, vec![RelicId::SacredBark], 1, &[], &[]);
    let before_hand = zero.state.piles.hand.iter().map(|card| card.handle()).collect::<Vec<_>>();
    use_potion(&mut zero);
    let selection = zero.state.pending_selection.as_ref().unwrap();
    assert_eq!(selection.parent, crate::combat::SelectionParent::Potion { slot: 0, id: PotionId::GamblersBrew });
    assert_eq!((selection.minimum, selection.maximum), (0, 4));
    zero.step_full(&CombatAction::ConfirmSelection).unwrap();
    assert_eq!(zero.state.piles.hand.iter().map(|card| card.handle()).collect::<Vec<_>>(), before_hand);
    assert!(zero.state.piles.discard.is_empty());
    assert_eq!(zero.state.silent.discarded_this_turn, 0);

    let mut selected = fight(&cards, 4, PotionId::GamblersBrew, vec![], 1, &[], &[]);
    let candidates = selected.state.piles.hand.iter().take(2).map(|card| card.handle()).collect::<Vec<_>>();
    use_potion(&mut selected);
    for candidate in candidates {
        selected.step_full(&CombatAction::SelectCard { instance: candidate.as_str().parse().unwrap() }).unwrap();
    }
    assert_eq!(selected.state.piles.discard.len(), 0, "selection is staged until confirmation");
    selected.step_full(&CombatAction::ConfirmSelection).unwrap();
    assert_eq!(selected.state.piles.discard.len(), 2);
    assert_eq!(selected.state.piles.hand.len(), 4, "the two discarded instances are replaced by two ordinary draws");
    assert_eq!(selected.state.silent.discarded_this_turn, 2);
}

#[test]
fn m15_liquid_memories_moves_selected_discard_cards_and_keeps_x_cost_negative() {
    let cards = [CardId::Whirlwind, CardId::StrikeGreen, CardId::DefendGreen];
    let mut combat = fight(&cards, 0, PotionId::LiquidMemories, vec![], 1, &[], &[]);
    draw_to(&mut combat, 1, "discard");
    draw_to(&mut combat, 2, "discard");
    use_potion(&mut combat);
    let selection = combat.state.pending_selection.as_ref().unwrap();
    assert_eq!(selection.parent, crate::combat::SelectionParent::Potion { slot: 0, id: PotionId::LiquidMemories });
    assert_eq!((selection.minimum, selection.maximum), (1, 1));
    combat.step_full(&CombatAction::SelectCard { instance: 1 }).unwrap();
    let x_card = combat.state.piles.hand.iter().find_map(|owned| owned.full())
        .filter(|card| card.id == CardId::Whirlwind).unwrap();
    assert!(x_card.local.x_cost);
    assert!(x_card.local.turn_cost.unwrap_or(-1) < 0, "guarded setCostForTurn(0) leaves X/unplayable price alone");
    assert!(combat.state.piles.discard.iter().any(|owned| owned.full().is_some_and(|card| card.instance.0 == 2)));

    let mut priced=fight(&[CardId::StrikeGreen],0,PotionId::LiquidMemories,vec![],1,&[],&[]);
    draw_to(&mut priced,1,"discard");
    use_potion(&mut priced);
    let retrieved=priced.state.piles.hand.iter().find_map(|owned|owned.full()).unwrap();
    assert_eq!(retrieved.local.turn_cost,Some(0),"ordinary nonnegative costs are set to zero for this turn");
}

#[test]
fn m15_liquid_memories_bark_retrieves_two_and_preserves_unplayable_turn_cost() {
    let cards=[CardId::Whirlwind,CardId::StrikeGreen,CardId::DefendGreen,CardId::StrikeGreen];
    let mut combat=fight(&cards,0,PotionId::LiquidMemories,vec![RelicId::SacredBark],1,&[],&[]);
    if let Some(card)=combat.state.piles.draw.iter_mut().find_map(|owned|owned.full_mut().filter(|card|card.instance.0==3)){
        card.local.turn_cost=Some(-2);
    }
    for instance in 1..=3{draw_to(&mut combat,instance,"discard");}
    use_potion(&mut combat);
    let selection=combat.state.pending_selection.as_ref().unwrap();
    assert_eq!((selection.minimum,selection.maximum),(2,2));
    combat.step_full(&CombatAction::SelectCard{instance:1}).unwrap();
    combat.step_full(&CombatAction::SelectCard{instance:3}).unwrap();
    let whirlwind=combat.state.piles.hand.iter().find_map(|owned|owned.full()).filter(|card|card.instance.0==1).unwrap();
    let unplayable=combat.state.piles.hand.iter().filter_map(|owned|owned.full()).find(|card|card.instance.0==3).unwrap();
    assert!(whirlwind.local.x_cost&&whirlwind.local.turn_cost.unwrap_or(-1)<0);
    assert_eq!(unplayable.local.turn_cost,Some(-2),"setCostForTurn(0) leaves the negative unplayable sentinel unchanged");
}

#[test]
fn m15_liquid_memories_keeps_selected_card_in_discard_when_hand_is_full() {
    let cards=[CardId::StrikeGreen;12];
    let mut combat=fight(&cards,10,PotionId::LiquidMemories,vec![],1,&[],&[]);
    let draw_instances=combat.state.piles.draw.iter().map(|owned|owned.handle().as_str().parse::<u64>().unwrap()).collect::<Vec<_>>();
    assert_eq!(draw_instances.len(),2);
    for instance in &draw_instances{draw_to(&mut combat,*instance,"discard");}
    use_potion(&mut combat);
    combat.step_full(&CombatAction::SelectCard{instance:draw_instances[0]}).unwrap();
    assert_eq!(combat.state.piles.hand.len(),10);
    assert!(combat.state.piles.discard.iter().any(|owned|owned.full().is_some_and(|card|card.instance.0==draw_instances[0])));
}

#[test]
fn m15_duplication_repeats_nonpurge_cards_by_charge_and_preserves_card_family() {
    let cards = [CardId::DefendGreen, CardId::StrikeGreen];
    for (bark, expected_repeats) in [(false, 1usize), (true, 2usize)] {
        let relics = if bark { vec![RelicId::SacredBark] } else { vec![] };
        let mut combat = fight(&cards, 2, PotionId::DuplicationPotion, relics, 1, &[], &[]);
        use_potion(&mut combat);
        assert_eq!(combat.state.player_full_powers.iter().find(|entry| entry.id.0 == "duplication").unwrap().state.amount,
            if bark { 2 } else { 1 });
        for instance in 1..=2 {
            combat.step_full(&CombatAction::PlayCard { instance,
                target: (instance == 2).then_some(crate::combat::TargetId(1)) }).unwrap();
        }
        assert_eq!(combat.state.inactive_cards.len(), expected_repeats);
        assert!(combat.state.inactive_cards.iter().all(|copy| copy.full().unwrap().local.purge));
        for copy in &combat.state.inactive_cards {
            assert!(combat.state.piles.discard.iter().any(|original|
                original.full().is_some_and(|card| card.copy_family == copy.full().unwrap().copy_family)));
        }
        assert!(!combat.state.player_full_powers.iter().any(|entry| entry.id.0 == "duplication"),
            "each real card consumes one charge and the last charge is removed through the shared power owner");
    }

    let mut duration=fight(&cards,0,PotionId::DuplicationPotion,vec![RelicId::SacredBark],1,&[],&[]);
    use_potion(&mut duration);
    for expected in [1,0]{
        duration.enqueue_native_effects(vec![crate::combat::Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),
            kind:EffectKind::Hook(Hook::TurnEnded)}],None).unwrap();
        assert_eq!(duration.state.player_full_powers.iter().find(|power|power.id.0=="duplication").map(|power|power.state.amount),
            (expected>0).then_some(expected),"unused Duplication charges decay at the shared TurnEnded hook");
    }
}

#[test]
fn m15_duplication_repeats_x_card_with_captured_energy_and_no_child_payment() {
    let cards=[CardId::Whirlwind,CardId::DefendGreen];
    let mut combat=fight(&cards,2,PotionId::DuplicationPotion,vec![],1,&[],&[]);
    use_potion(&mut combat);
    let enemy_before=combat.state.enemies[0].hp;
    combat.step_full(&CombatAction::PlayCard{instance:1,target:None}).unwrap();
    assert_eq!(combat.state.enemies[0].hp,enemy_before-30,"original and free copy both use captured X=3");
    assert_eq!(combat.state.player.energy,0,"the copied X card receives no second payment");
    assert_eq!(combat.state.inactive_cards.len(),1);
    assert_eq!(combat.state.inactive_cards[0].full().unwrap().copy_family,1);
}

#[test]
fn m15_distilled_chaos_holds_potion_parent_through_child_selection_and_resumes_queue() {
    let cards = [CardId::DefendGreen, CardId::StrikeGreen, CardId::DefendGreen, CardId::SecretTechnique];
    let mut combat = fight(&cards, 0, PotionId::DistilledChaos, vec![], 2, &[], &[]);
    place_on_top(&mut combat,4);
    use_potion(&mut combat);
    assert_eq!(combat.state.phase, crate::combat::Phase::Selection);
    let selection = combat.state.pending_selection.as_ref().unwrap();
    assert_eq!(selection.parent, crate::combat::SelectionParent::Card(crate::combat::CardHandle::parse("4").unwrap()));
    let frame = combat.state.play_frames.last().unwrap();
    assert!(matches!(frame.parent_owner, crate::combat::model::PlayFrameParentOwner::Potion { slot: 0, id: PotionId::DistilledChaos }));
    assert!(frame.parent.is_none() && frame.parent_context.is_none());
    assert_eq!(combat.state.pending.iter().filter(|effect| matches!(effect.kind,
        EffectKind::Potion(crate::combat::PotionEffect::DistilledChaosTopCard { target: Some(_) }))).count(), 2,
        "remaining use-time target proposals are already captured while the first child is suspended");
    combat.validate_runtime_for_test().unwrap();
    combat.step_full(&CombatAction::SelectCard { instance: 1 }).unwrap();
    assert_eq!(combat.state.phase, crate::combat::Phase::Decision);
    assert!(combat.state.play_frames.is_empty());
    assert!(combat.state.in_play.is_none());
    assert_eq!(combat.state.player.energy, 3, "autoplay child cards do not pay hand energy");
    assert!(combat.state.piles.hand.iter().any(|owned| owned.full().is_some_and(|card| card.instance.0 == 1)));
    assert_eq!(combat.state.pending.iter().filter(|effect| matches!(effect.kind,
        EffectKind::Potion(crate::combat::PotionEffect::DistilledChaosTopCard { .. }))).count(), 0);
}

#[test]
fn m15_distilled_chaos_bark_captures_all_six_targets_before_first_child_resolves() {
    let cards=[CardId::DefendGreen,CardId::StrikeGreen,CardId::DefendGreen,CardId::SecretTechnique];
    let mut combat=fight(&cards,0,PotionId::DistilledChaos,vec![RelicId::SacredBark],2,&[],&[]);
    place_on_top(&mut combat,4);
    use_potion(&mut combat);
    assert_eq!(combat.state.phase,crate::combat::Phase::Selection);
    assert_eq!(combat.state.pending.iter().filter(|effect|matches!(effect.kind,
        EffectKind::Potion(crate::combat::PotionEffect::DistilledChaosTopCard{target:Some(_)}))).count(),5,
        "all five remaining proposals are captured before the first child's selection suspends");
    combat.step_full(&CombatAction::SelectCard{instance:1}).unwrap();
    assert_eq!(combat.state.phase,crate::combat::Phase::Decision);
    assert!(combat.state.play_frames.is_empty());
    assert!(!combat.state.pending.iter().any(|effect|matches!(effect.kind,
        EffectKind::Potion(crate::combat::PotionEffect::DistilledChaosTopCard{..}))));
}

#[test]
fn m15_distilled_chaos_first_child_fault_retains_typed_potion_frame_and_source() {
    let cards = [CardId::DefendGreen, CardId::StrikeGreen, CardId::DefendGreen, CardId::SecretTechnique];
    let mut combat = fight(&cards, 0, PotionId::DistilledChaos, vec![], 1, &[], &[]);
    place_on_top(&mut combat,4);
    use_potion(&mut combat);
    assert_eq!(combat.state.phase,crate::combat::Phase::Selection);
    combat.state.pending_selection=None;
    combat.state.phase=crate::combat::Phase::PlayerAction;
    combat.state.pending.push_front(crate::combat::Effect {
        source: Source::Potion { slot: 0, id: PotionId::DistilledChaos }, target: Some(Actor::Player),
        kind: EffectKind::Hook(Hook::CardAfterUse),
    });
    let transition = super::super::engine::resolve(&mut combat.state);
    assert_eq!(transition.status, crate::combat::TransitionStatus::Fault);
    let fault = combat.state.fault.as_ref().unwrap();
    assert_eq!(fault.kind, crate::combat::FaultKind::Invariant);
    assert_eq!(combat.state.current.as_ref().unwrap().source,
        Source::Potion { slot: 0, id: PotionId::DistilledChaos });
    assert!(matches!(combat.state.play_frames.last().unwrap().parent_owner,
        crate::combat::model::PlayFrameParentOwner::Potion { slot: 0, id: PotionId::DistilledChaos }));
    assert!(combat.state.in_play.is_some(), "first fault keeps actual child ownership for diagnostics");
    combat.validate_runtime_for_test().unwrap();
}

#[test]
fn m15_distilled_chaos_queue_cancellation_preserves_unmaterialized_top_actions() {
    let cards=[CardId::StrikeGreen,CardId::DefendGreen,CardId::StrikeGreen,CardId::DefendGreen];
    let mut combat=fight(&cards,0,PotionId::DistilledChaos,vec![],1,&[],&[]);
    use_potion(&mut combat);
    let target=combat.state.enemies[0].instance_id.clone();
    let mut child=combat.state.piles.discard.pop().unwrap();
    let handle=child.handle();
    if let Some(card)=child.full_mut(){card.local.free_play=true;}
    combat.state.queued_autoplay.insert(handle.clone(),child);
    combat.state.pending.push_back(crate::combat::Effect{source:Source::Potion{slot:0,id:PotionId::DistilledChaos},target:None,
        kind:EffectKind::Potion(crate::combat::PotionEffect::DistilledChaosTopCard{target:Some(target.clone())})});
    combat.state.pending.push_back(crate::combat::Effect{source:Source::Potion{slot:0,id:PotionId::DistilledChaos},target:None,
        kind:EffectKind::Potion(crate::combat::PotionEffect::DistilledChaosQueuedChild{child:handle.clone(),target:Some(target),
            energy_on_use:-1,cancelled:false})});
    combat.state.phase=crate::combat::Phase::PlayerAction;
    crate::combat::cards::ironclad::autoplay::cancel_pending_card_queue(&mut combat.state);
    assert!(matches!(combat.state.pending.front().map(|effect|&effect.kind),Some(EffectKind::Potion(
        crate::combat::PotionEffect::DistilledChaosTopCard{target:Some(_)}))),"Awakened/Time Eater cancellation retains captured but unmaterialized top-card actions");
    assert!(matches!(combat.state.pending.back().map(|effect|&effect.kind),Some(EffectKind::Potion(
        crate::combat::PotionEffect::DistilledChaosQueuedChild{child:queued,cancelled:true,..})) if queued==&handle));
    combat.validate_runtime_for_test().unwrap();
    let transition=super::super::engine::resolve(&mut combat.state);
    assert_eq!(transition.status,crate::combat::TransitionStatus::Decision);
    assert!(!combat.state.queued_autoplay.contains_key(&handle));
    assert!(combat.state.piles.discard.iter().any(|owned|owned.handle()==handle),"canceled detached child is disposed through the queued-autoplay owner");
}

#[test]
fn m15_typed_play_frame_preserves_legacy_card_debug_shape() {
    use crate::combat::model::{FinishDisposition,OwnedCard,PlayContext,PlayFrame,PlayFrameParentOwner};
    let parent=OwnedCard::Full(crate::combat::card_state::permanent(CardId::Havoc,crate::combat::CardInstanceId(90),0));
    let parent_handle=parent.handle();
    let child=crate::combat::card_state::handle(crate::combat::CardInstanceId(91));
    let context=PlayContext{card:parent_handle.clone(),target:None,energy_on_use:-1,paid:0,disposition:FinishDisposition::Natural};
    let frame=PlayFrame{parent:Some(parent),parent_owner:PlayFrameParentOwner::Card(parent_handle.clone()),
        parent_context:Some(context),child:child.clone(),target:None,captured_energy:-1,paid:0,
        disposition:FinishDisposition::Natural,parent_finish_checkpoint_done:false};
    let debug=format!("{frame:?}");
    let expected=format!("PlayFrame {{ parent: {:?}, parent_handle: {:?}, parent_context: {:?}, child: {:?}, target: None, captured_energy: -1, paid: 0, disposition: Natural, parent_finish_checkpoint_done: false }}",
        frame.parent,parent_handle,frame.parent_context,child);
    assert_eq!(debug,expected,"Card-owned frames retain the prior Debug field names, order, and bytes");

    let potion=PlayFrame{parent:None,parent_owner:PlayFrameParentOwner::Potion{slot:0,id:PotionId::DistilledChaos},
        parent_context:None,child,target:None,captured_energy:-1,paid:0,disposition:FinishDisposition::Natural,
        parent_finish_checkpoint_done:false};
    assert_eq!(format!("{potion:?}"),"PlayFrame { parent: None, parent_handle: Potion { slot: 0, id: DistilledChaos }, parent_context: None, child: CardHandle(\"91\"), target: None, captured_energy: -1, paid: 0, disposition: Natural, parent_finish_checkpoint_done: false }");
}

#[test]
fn independent_review_duplication_power_disposition() {
    let mut c=fight(&[CardId::Footwork,CardId::DefendGreen],2,PotionId::DuplicationPotion,vec![],1,&[],&[]);
    use_potion(&mut c);
    c.step_full(&CombatAction::PlayCard{instance:1,target:None}).unwrap();
    assert_eq!(c.state.player_full_powers.iter().find(|p|p.id.0=="dexterity").unwrap().state.amount,4);
    println!("discard={:?}, played_powers={:?}, inactive={:?}",c.state.piles.discard,c.state.played_powers,c.state.inactive_cards);
    assert!(!c.state.piles.discard.iter().any(|card|card.full().is_some_and(|card|card.id==CardId::Footwork)),"played Power and purge repeat must not return to drawable discard");
}


#[test]
fn m15_review_opening_draw_callbacks_respect_innate_relics_and_capacity() {
    let mut cards=vec![CardId::Backstab];cards.extend([CardId::EndlessAgony;11]);
    let c=fight(&cards,5,PotionId::DuplicationPotion,vec![RelicId::RingOfTheSnake,RelicId::BagOfPreparation,RelicId::NinjaScroll],1,&[],&[]);
    let view=c.public_full().unwrap();
    assert_eq!(view.hand.len(),10);
    assert_eq!(view.hand.iter().filter(|c|c.id==CardId::Shiv).count(),3);
    assert!(view.hand.iter().any(|c|c.id==CardId::Backstab));
    assert_eq!(view.discard.iter().filter(|c|c.id==CardId::EndlessAgony).count(),2,"opening copies fill the cap before the later Ring/Bag draw requests");
    assert_eq!(view.draw_count,7);
    let generated=view.hand.iter().chain(&view.discard).filter(|card|card.id==CardId::EndlessAgony&&matches!(card.origin,CombatCardRef::CombatLocal(_))).count();
    assert_eq!(generated,4,"one generated child per actual opening Agony; creation never draws itself");
    assert_eq!(c.state.card_ids_issued.len(),19,"12permanent+3pre-drawShivs+4copies; no physical identity reuse");
}

#[test]
fn m15_review_repeated_power_retires_original_and_purges_child_with_callbacks() {
    for repeated in [false,true] {
        let mut c=fight(&[CardId::NoxiousFumes,CardId::DefendGreen],2,PotionId::DuplicationPotion,vec![],1,&[],&[]);
        if repeated{use_potion(&mut c);}
        c.step_full(&CombatAction::PlayCard{instance:1,target:None}).unwrap();
        assert!(c.state.played_powers.iter().any(|c|c.full().is_some_and(|c|c.instance.0==1)));
        assert!(!c.state.piles.discard.iter().any(|c|c.full().is_some_and(|c|c.id==CardId::NoxiousFumes)));
        assert_eq!(c.state.inactive_cards.iter().filter(|c|c.full().is_some_and(|c|c.id==CardId::NoxiousFumes && c.local.purge)).count(),usize::from(repeated));
        c.step_full(&CombatAction::EndTurn).unwrap();
        assert_eq!(c.state.enemies[0].full_powers.iter().find(|p|p.id.0=="poison").unwrap().state.amount,if repeated{4}else{2},"retained Power callback still executes");
    }
}


#[test]
fn m15_review_grand_finale_snecko_oil_randomizes_known_price_while_unplayable() {
    let mut cards=vec![CardId::GrandFinale];cards.extend([CardId::DefendGreen;8]);
    let mut c=fight(&cards,0,PotionId::SneckoOil,vec![],1,&[],&[]);draw_to(&mut c,1,"hand");
    let initial=c.state.card_rng.as_ref().unwrap().clone();let (first,mut expected)=crate::combat::rng::uniform_below(&initial,4).unwrap();
    for _ in 1..6 {expected=crate::combat::rng::uniform_below(&expected,4).unwrap().1;}
    use_potion(&mut c);
    assert_eq!(c.state.card_rng.as_ref(),Some(&expected),"all six resulting hand cards receive the ordinary fixed-cost proposal");
    let view=c.public_full().unwrap();assert_eq!(view.draw_count,3);
    assert_eq!(view.hand.iter().find(|c|c.id==CardId::GrandFinale).unwrap().effective_cost,Some(first as i32));
    assert!(!c.legal_actions_full().unwrap().contains(&CombatAction::PlayCard{instance:1,target:None}));
    c.step_full(&CombatAction::EndTurn).unwrap();
    let grand=c.state.piles.all(c.state.in_play.as_ref()).filter_map(|c|c.full()).find(|card|card.id==CardId::GrandFinale).unwrap();
    assert_eq!(grand.local.turn_cost,Some(first as i32),"Snecko Oil's combat price survives turn reset");
}
