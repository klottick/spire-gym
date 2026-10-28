use spire_core::{combat::*, content::{CardId, Character, EncounterId, PotionId, RelicId}, run::*};
use spire_core::run::CardInstance;
#[path = "../examples/m3_support/mod.rs"]
#[allow(dead_code)]
mod support;

#[test]
fn authored_checkpoint_preserves_explicit_resources() {
    let run = support::run(7);
    let before = run.clone();
    let public = run.decision().unwrap();
    assert_eq!((public.act, public.floor, public.player.hp, public.player.max_hp, public.player.gold), (1, 0, 40, 80, 123));
    assert_eq!(public.objects.iter().map(|c| (c.instance, c.id, c.upgrades)).collect::<Vec<_>>(),
        vec![(1,CardId::Strike,1),(2,CardId::Bash,1),(3,CardId::Anger,0),(4,CardId::Bloodletting,0),(5,CardId::Disarm,0)]);
    assert!(public.offers.is_empty() && public.candidates.is_empty());
    assert_eq!(public.player.potions, support::start().potions);
    assert_eq!(run, before);
}

#[test]
fn invalid_authored_resources_and_foreign_profiles_reject_before_entry() {
    let invalid: Vec<Box<dyn Fn(&mut AuthoredRunStart)>> = vec![
        Box::new(|s| s.hp=0), Box::new(|s| s.hp=81), Box::new(|s| s.max_hp=1000),
        Box::new(|s| s.deck.clear()), Box::new(|s| s.deck[0].instance=0),
        Box::new(|s| s.deck[0].instance=2), Box::new(|s| s.deck[0].instance=u64::MAX),
        Box::new(|s| s.deck[0].upgrades=2), Box::new(|s| s.deck[0].id=CardId::Unknown("unsupported")),
        Box::new(|s| s.deck=(1..=1000).map(|instance| CardInstance { misc:0, instance,id:CardId::Strike,upgrades:0 }).collect()),
        Box::new(|s| s.relics.push(RelicId::Unknown("unsupported"))),
        Box::new(|s| s.relics.push(RelicId::BurningBlood)),
        Box::new(|s| s.potions[0]=Some(PotionId::Unknown("unsupported"))), Box::new(|s| { s.potions.pop(); }),
    ];
    for alter in invalid {
        let mut start=support::start(); alter(&mut start);
        assert!(Run::from_authored_combat(support::config(),start,support::rng(7)).is_err());
    }
    for config in [RunConfig {character:Character::Silent,ascension:0}, RunConfig {character:Character::Ironclad,ascension:1}] {
        assert!(Run::from_authored_combat(config,support::start(),support::rng(7)).is_err());
    }
}

#[test]
fn rejected_entry_preserves_checkpoint_and_success_records_one_visit() {
    let mut run=support::run(7);
    let before=run.clone();
    assert!(run.enter_combat(EncounterId::SlimeBoss).is_err());
    assert_eq!(run,before);
    assert_eq!(run.enter_combat(EncounterId::JawWorm).unwrap().committed_events,
        vec![CommittedEvent::CombatEntered {visit:0,encounter:EncounterId::JawWorm}]);
    let accepted=run.clone();
    assert_eq!(run.enter_combat(EncounterId::Cultist),Err(RunError::InvalidAction));
    assert_eq!(run,accepted);
}

fn take(run:&mut Run, action:CombatAction) -> RunTransition {
    assert!(run.decision().unwrap().candidates.iter().any(|c| c.action==GameAction::Combat(action.clone())));
    run.step(&GameAction::Combat(action)).unwrap()
}
fn play(run:&mut Run, instance:u64) -> RunTransition {
    let target=if instance==4 {None} else {Some(TargetId(1))};
    take(run,CombatAction::PlayCard {instance,target})
}
fn potion(run:&mut Run, slot:usize, target:Option<TargetId>) -> RunTransition {
    take(run,CombatAction::UsePotion {slot,target})
}
fn prepare_first_kill(run:&mut Run) {
    play(run,5); play(run,4); potion(run,0,None);
    play(run,3); play(run,2); play(run,1);
}

#[test]
fn active_projection_and_invalid_actions_are_atomic() {
    let mut run=support::run(7);
    run.enter_combat(EncounterId::JawWorm).unwrap();
    play(&mut run,4);
    potion(&mut run,0,None);
    let before=run.clone();
    let public=run.decision().unwrap();
    let combat=public.combat.unwrap();
    assert_eq!((public.player.hp,combat.player.hp),(39,39));
    assert_eq!(public.player.potions,combat.inventory.unwrap().potions);
    assert_eq!(public.player.potions[0],None);
    assert_eq!(public.combat_progress,CombatProgress {entered:1,settled:0});
    for action in [GameAction::Choose(ChoiceId(0)),GameAction::Move(NodeId(0)),GameAction::Confirm,
        GameAction::UsePotion {slot:1,target:Some(TargetId(1))},
        GameAction::Combat(CombatAction::UsePotion {slot:0,target:None}),
        GameAction::Combat(CombatAction::PlayCard {instance:999,target:None})] {
        assert_eq!(run.step(&action),Err(RunError::InvalidAction));
        assert_eq!(run,before);
    }
    assert_eq!(run.remove_card(1),Err(RunError::InvalidAction));
    assert_eq!(run,before);
    assert_eq!(run.decision(),run.decision());
    assert_eq!(run,before);
    let mut direct=run.clone();
    assert_eq!(direct.choose_combat(CombatAction::EndTurn),run.step(&GameAction::Combat(CombatAction::EndTurn)));
    assert_eq!(direct,run);
}

#[test]
fn terminal_action_settles_once_without_refunding_or_double_healing() {
    let mut run=support::run(7);
    run.enter_combat(EncounterId::JawWorm).unwrap();
    prepare_first_kill(&mut run);
    let public=run.decision().unwrap();
    let combat=public.combat.unwrap();
    assert_eq!((combat.player.hp,combat.player.block,combat.player.energy),(39,10,2));
    assert_eq!(combat.exhaust[0].instance,CardInstanceId(5));
    assert!(combat.discard.iter().any(|c| matches!(c.origin,CombatCardRef::CombatLocal(_))));
    let transition=potion(&mut run,1,Some(TargetId(1)));
    let record=CombatRecord {visit:0,encounter:EncounterId::JawWorm,outcome:Outcome::Victory};
    assert_eq!(transition.committed_events,vec![CommittedEvent::CombatFinished(record.clone())]);
    let public=run.decision().unwrap();
    assert_eq!((public.player.hp,public.player.max_hp,public.player.gold),(45,80,123));
    assert_eq!(public.player.potions,vec![None,None,Some(PotionId::BloodPotion)]);
    assert_eq!(public.combat_progress,CombatProgress {entered:1,settled:1});
    assert_eq!(public.last_combat,Some(record));
    assert!(public.combat.is_none() && public.terminal.is_none());
    assert_eq!(public.objects.len(),5);
    let before=run.clone();
    assert_eq!(run.choose_combat(CombatAction::UsePotion {slot:1,target:Some(TargetId(1))}),Err(RunError::InvalidAction));
    assert_eq!(run,before);
}

#[test]
fn two_real_fights_preserve_permanent_resources_and_reset_combat_locals() {
    let mut run=support::run(7);
    let permanent=run.decision().unwrap().objects;
    run.enter_combat(EncounterId::JawWorm).unwrap();
    prepare_first_kill(&mut run); potion(&mut run,1,Some(TargetId(1)));
    assert_eq!(run.decision().unwrap().player.hp,45);
    let entry=run.enter_combat(EncounterId::Cultist).unwrap();
    assert_eq!(entry.committed_events,vec![CommittedEvent::CombatEntered {visit:1,encounter:EncounterId::Cultist}]);
    let view=run.decision().unwrap(); let combat=view.combat.unwrap();
    assert_eq!((view.player.hp,combat.player.energy,combat.player.block),(47,4,10));
    assert!(combat.player.powers.is_empty() && combat.exhaust.is_empty() && combat.discard.is_empty());
    assert_eq!(combat.hand.len(),5);
    assert!(combat.hand.iter().all(|c| matches!(c.origin,CombatCardRef::Permanent(_))));
    assert_eq!(view.objects,permanent);
    potion(&mut run,2,None);
    assert_eq!(run.decision().unwrap().player.hp,63);
    for instance in [5,4,3,2,1] { play(&mut run,instance); }
    assert_eq!(run.decision().unwrap().player.hp,60);
    let combat=run.decision().unwrap().combat.unwrap();
    assert_eq!(combat.exhaust[0].instance,CardInstanceId(5),"innate Disarm exhaust was rebuilt");
    assert!(combat.enemies[0].hp>0);
    take(&mut run,CombatAction::EndTurn);
    let combat=run.decision().unwrap().combat.unwrap();
    assert_eq!((combat.player.hp,combat.player.energy,combat.enemies[0].intent),(60,3,FullIntentId::Attack));
    let copy=combat.hand.iter().find(|c| c.id==CardId::Anger && matches!(c.origin,CombatCardRef::CombatLocal(_))).unwrap().instance.0;
    play(&mut run,3);
    take(&mut run,CombatAction::PlayCard {instance:copy,target:Some(TargetId(1))});
    let exit=play(&mut run,2); // Fatal Bash remains in_play when the terminal export happens.
    assert_eq!(exit.committed_events,vec![CommittedEvent::CombatFinished(CombatRecord {
        visit:1,encounter:EncounterId::Cultist,outcome:Outcome::Victory})]);
    let view=run.decision().unwrap();
    assert_eq!((view.player.hp,view.player.max_hp,view.player.gold),(66,80,123));
    assert_eq!(view.player.potions,vec![None,None,None]);
    assert_eq!(view.objects,permanent);
    assert_eq!(view.combat_progress,CombatProgress {entered:2,settled:2});
    assert!(view.combat.is_none() && view.terminal.is_none());
}

#[test]
fn real_enemy_death_is_one_terminal_without_burning_blood_heal() {
    let mut start=support::start(); start.hp=11;
    start.relics=vec![RelicId::BurningBlood]; start.potions=vec![None,None,None];
    let mut run=Run::from_authored_combat(support::config(),start,support::rng(7)).unwrap();
    run.enter_combat(EncounterId::JawWorm).unwrap();
    let transition=take(&mut run,CombatAction::EndTurn);
    assert_eq!(transition.committed_events,vec![CommittedEvent::CombatFinished(CombatRecord {
        visit:0,encounter:EncounterId::JawWorm,outcome:Outcome::Death}),CommittedEvent::Ended(RunTerminal::Death)]);
    let view=run.decision().unwrap();
    assert_eq!((view.player.hp,view.terminal),(0,Some(RunTerminal::Death)));
    assert_eq!(view.combat_progress,CombatProgress {entered:1,settled:1});
    assert!(view.candidates.is_empty() && view.combat.is_none());
    let before=run.clone();
    assert_eq!(run.enter_combat(EncounterId::Cultist),Err(RunError::Terminal));
    assert_eq!(run.choose_combat(CombatAction::EndTurn),Err(RunError::Terminal));
    assert_eq!(run.remove_card(1),Err(RunError::Terminal));
    assert_eq!(run,before);
}

#[test]
fn same_seed_actions_replay_and_rejections_do_not_perturb_next_fight() {
    let mut a=support::run(7); let mut b=support::run(7);
    for encounter in [EncounterId::JawWorm,EncounterId::Cultist] {
        assert_eq!(a.enter_combat(encounter),b.enter_combat(encounter));
        let mut apply=|action:CombatAction| {
            assert_eq!(a.decision(),b.decision());
            let before=b.clone();
            assert_eq!(b.choose_combat(CombatAction::PlayCard {instance:999,target:None}),Err(RunError::InvalidAction));
            assert_eq!(b,before);
            assert_eq!(take(&mut a,action.clone()),take(&mut b,action));
            assert_eq!(a.decision(),b.decision());
        };
        if encounter==EncounterId::Cultist { apply(CombatAction::UsePotion {slot:2,target:None}); }
        for (i,target) in [(5,Some(TargetId(1))),(4,None)] {apply(CombatAction::PlayCard {instance:i,target});}
        if encounter==EncounterId::JawWorm {apply(CombatAction::UsePotion {slot:0,target:None});}
        for i in [3,2,1] {apply(CombatAction::PlayCard {instance:i,target:Some(TargetId(1))});}
        if encounter==EncounterId::JawWorm {apply(CombatAction::UsePotion {slot:1,target:Some(TargetId(1))});}
        else {
            apply(CombatAction::EndTurn);
            // Five canonical cards have been issued; the one turn1 copy is6.
            for i in [3,6,2] {apply(CombatAction::PlayCard {instance:i,target:Some(TargetId(1))});}
        }
    }
    assert_eq!(a.decision().unwrap().player.hp,66);
    assert_eq!(a,b);
}

#[test]
fn max_hp_and_potion_consumption_persist_through_settlement_and_next_entry() {
    let mut start=support::start(); start.deck.truncate(1); start.relics=vec![RelicId::BurningBlood];
    start.potions=vec![Some(PotionId::FruitJuice),Some(PotionId::FirePotion),Some(PotionId::FirePotion)];
    let mut run=Run::from_authored_combat(support::config(),start,support::rng(7)).unwrap();
    run.enter_combat(EncounterId::JawWorm).unwrap();
    potion(&mut run,0,None);
    let view=run.decision().unwrap(); assert_eq!((view.player.hp,view.player.max_hp),(45,85));
    play(&mut run,1); potion(&mut run,1,Some(TargetId(1))); potion(&mut run,2,Some(TargetId(1)));
    let view=run.decision().unwrap(); assert_eq!((view.player.hp,view.player.max_hp),(51,85));
    run.enter_combat(EncounterId::Cultist).unwrap();
    let view=run.decision().unwrap(); assert_eq!((view.player.hp,view.player.max_hp),(51,85));
    assert_eq!(view.player.potions,vec![None,None,None]);
}

#[test]
fn burning_pact_selection_resumes_one_parent_through_the_run_action_menu() {
    let mut start=support::start(); start.relics.clear();
    start.deck=[CardId::Strike,CardId::Strike,CardId::Defend,CardId::Defend,CardId::BurningPact,CardId::PommelStrike]
        .into_iter().enumerate().map(|(i,id)|CardInstance { misc:0,instance:i as u64+1,id,upgrades:0}).collect();
    start.potions=vec![Some(PotionId::SwiftPotion),None,None];
    let mut run=Run::from_authored_combat(support::config(),start,support::rng(7)).unwrap();
    run.enter_combat(EncounterId::JawWorm).unwrap(); potion(&mut run,0,None); // Real draw of sixth card.
    for (instance,target) in [(3,None),(1,Some(TargetId(1))),(5,None)] {
        take(&mut run,CombatAction::PlayCard {instance,target});
    }
    let view=run.decision().unwrap(); let selection=view.combat.unwrap().selection.unwrap();
    assert_eq!((selection.required,selection.disposition),(1,SelectionDisposition::Exhaust));
    assert!(view.candidates.iter().all(|c|matches!(c.action,GameAction::Combat(CombatAction::SelectCard {..}))));
    take(&mut run,CombatAction::SelectCard {instance:4});
    let view=run.decision().unwrap().combat.unwrap();
    assert_eq!((view.hand.len(),view.exhaust.len(),view.discard.len(),view.player.energy),(4,1,1,0));
    assert_eq!(view.exhaust[0].instance,CardInstanceId(4));
    let before=run.clone();
    assert_eq!(run.choose_combat(CombatAction::SelectCard {instance:4}),Err(RunError::InvalidAction));
    assert_eq!(run,before);
}

#[test]
fn caller_budget_never_advances_an_extra_action_or_fabricates_an_outcome() {
    let mut run=support::run(7); run.enter_combat(EncounterId::JawWorm).unwrap();
    let before=run.clone();
    assert!(matches!(support::Caller::new(0).step(&mut run,CombatAction::EndTurn),
        Err(support::DemoError::BudgetExceeded {accepted:0})));
    assert_eq!(run,before);
    let mut caller=support::Caller::new(1);
    caller.step(&mut run,CombatAction::EndTurn).unwrap();
    let after=run.clone();
    assert!(matches!(caller.step(&mut run,CombatAction::EndTurn),Err(support::DemoError::BudgetExceeded {accepted:1})));
    assert_eq!(run,after);
    assert!(run.decision().unwrap().terminal.is_none());
}
