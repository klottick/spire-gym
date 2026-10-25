use super::super::*;
use super::support::base_scenario;

#[test]
fn invalid_targets_pay_draw_nothing() {
    let mut combat = Combat::new(&base_scenario()).unwrap();
    let before = combat.state.clone();
    let result = combat.step(&Action::PlayCard { card_instance_id: CardHandle::parse("1").unwrap(),
        target_id: Some(EnemyHandle::parse("missing").unwrap()) });
    assert_eq!(result, Err(StepError::InvalidAction));
    assert_eq!(combat.state, before);
}

#[test]
fn ordered_multi_pick_resolves_distinct_same_kind_instances_and_disposition() {
    let mut combat = super::full_runtime::entered_combat(3, 3, |_| {});
    let parent = combat.state.piles.hand.remove(0);
    let parent_id = parent.handle();
    let first = combat.state.piles.hand[0].handle();
    let second = combat.state.piles.hand[1].handle();
    combat.state.in_play = Some(parent);
    combat.state.phase = Phase::PlayerAction;
    combat.state.pending.push_back(Effect { source: Source::Card(parent_id.clone()), target: None,
        kind: EffectKind::SelectCards { candidates: vec![first.clone(), second.clone()], required: 2,
            disposition: SelectionDisposition::Discard, empty: EmptySelection::Continue } });
    combat.state.pending.push_back(Effect { source: Source::Card(parent_id.clone()), target: None,
        kind: EffectKind::FinishCard });
    combat.state.pending.push_back(Effect { source: Source::None, target: None, kind: EffectKind::Decision });
    assert_eq!(super::super::engine::resolve(&mut combat.state).status, TransitionStatus::Selection);
    let a: u64 = first.as_str().parse().unwrap();
    let b: u64 = second.as_str().parse().unwrap();
    assert_eq!(combat.legal_actions_full().unwrap().len(), 2);
    combat.step_full(&CombatAction::SelectCard { instance: b }).unwrap();
    assert_eq!(combat.state.phase, Phase::Selection, "first pick suspends incrementally");
    assert_eq!(combat.legal_actions_full().unwrap(), vec![CombatAction::SelectCard { instance: a }]);
    assert_eq!(combat.public_full().unwrap().selection.unwrap().selected, vec![CardInstanceId(b)]);
    let after_first = combat.state.clone();
    assert_eq!(combat.step_full(&CombatAction::SelectCard { instance: b }),
        Err(crate::run::RunError::InvalidAction));
    assert_eq!(combat.state, after_first, "repeated picks are rejected without changing the capture");
    combat.step_full(&CombatAction::SelectCard { instance: a }).unwrap();
    assert_eq!(combat.state.phase, Phase::Decision);
    assert_eq!(combat.state.piles.discard.iter().map(|c| c.handle()).collect::<Vec<_>>(), vec![second, first, parent_id]);
    assert_eq!(combat.state.player.energy, 3, "resume does not repay the parent action");
}

#[test]
fn terminal_full_exit_exports_real_owned_cards_and_resources() {
    let mut combat = super::full_runtime::entered_combat(3, 3, |_| {});
    combat.state.player.hp = 27;
    combat.state.enemies[0].hp = 8;
    combat.step_full(&CombatAction::PlayCard { instance: 3, target: Some(TargetId(1)) }).unwrap();
    let exit = combat.exit().unwrap();
    assert_eq!(exit.outcome, Outcome::Victory);
    assert_eq!((exit.hp, exit.max_hp, exit.energy), (27, 50, 1));
    assert!(exit.item_context.is_none(), "the fixture does not own Burning Blood");
    let cards = exit.hand.into_iter().chain(exit.draw).chain(exit.discard).chain(exit.exhaust)
        .chain(exit.in_play.iter().cloned()).collect::<Vec<_>>();
    assert_eq!(cards.iter().filter(|card| card.instance == CardInstanceId(3)
        && card.id == crate::content::CardId::Bash && card.upgrades == 0
        && card.origin == CombatCardRef::Permanent(CardInstanceId(3))).count(), 1);
    assert!(exit.in_play.is_some(), "terminal handoff preserves the lethal card before FinishCard");
}

#[test]
fn fault_while_resuming_selection_retains_parent_capture_and_current_queue() {
    let mut combat = super::full_runtime::entered_combat(3, 3, |_| {});
    let parent = combat.state.piles.hand.remove(0);
    let parent_id = parent.handle();
    let candidate = combat.state.piles.hand[0].handle();
    combat.state.in_play = Some(parent);
    combat.state.phase = Phase::PlayerAction;
    combat.state.pending.extend([
        Effect { source: Source::Card(parent_id.clone()), target: None,
            kind: EffectKind::SelectCards { candidates: vec![candidate.clone()], required: 1,
                disposition: SelectionDisposition::Discard, empty: EmptySelection::Continue } },
        Effect { source: Source::Card(parent_id.clone()), target: None, kind: EffectKind::FinishCard },
        Effect { source: Source::None, target: None, kind: EffectKind::Decision },
    ]);
    assert_eq!(super::super::engine::resolve(&mut combat.state).status, TransitionStatus::Selection);
    combat.state.resolved_effects = 99_999;
    let result = combat.step_full(&CombatAction::SelectCard { instance: candidate.as_str().parse().unwrap() });
    let crate::run::RunError::Faulted(packet) = result.unwrap_err() else { panic!("fault expected") };
    let crate::run::RunFaultKind::Combat(native) = packet.kind else { panic!("combat payload expected") };
    assert_eq!(native.current.as_ref().unwrap().kind, EffectKind::FinishCard);
    assert_eq!(native.pending.front().unwrap().kind, EffectKind::Decision);
    let selection = native.selection.as_ref().unwrap();
    assert_eq!(selection.parent, SelectionParent::Card(parent_id));
    assert_eq!(selection.candidates, vec![candidate.clone()]);
    assert_eq!(selection.selected, vec![candidate.clone()]);
    assert_eq!(native.attempted_action, Some(CombatAction::SelectCard {
        instance: candidate.as_str().parse().unwrap() }));
}

#[test]
fn selected_exact_owned_copy_resumes_parent_and_finish_once() {
    let mut combat = super::full_runtime::entered_combat(3, 3, |_| {});
    let parent = combat.state.piles.hand.remove(0);
    let selected = combat.state.piles.hand.pop().unwrap();
    let parent_id = parent.handle();
    let mut selected = selected;
    let parent_card_id = parent.full().unwrap().id;
    selected.full_mut().unwrap().id = parent_card_id;
    let selected_id = selected.handle();
    combat.state.piles.draw.push(selected.clone());
    combat.state.in_play = Some(parent);
    combat.state.phase = Phase::PlayerAction;
    combat.state.pending.push_back(Effect { source: Source::Card(parent_id.clone()), target: None,
        kind: EffectKind::SelectCard { candidates: vec![selected_id.clone()], empty: EmptySelection::Continue } });
    combat.state.pending.push_back(Effect { source: Source::Card(parent_id.clone()), target: None, kind: EffectKind::FinishCard });
    combat.state.pending.push_back(Effect { source: Source::None, target: None, kind: EffectKind::Decision });
    let transition = super::super::engine::resolve(&mut combat.state);
    assert_eq!(transition.status, TransitionStatus::Selection);
    let view = combat.public_full().unwrap().selection.unwrap();
    assert_eq!(view.candidates.iter().map(|c| c.instance).collect::<Vec<_>>(),
        vec![CardInstanceId(selected_id.as_str().parse().unwrap())]);

    let before = combat.state.clone();
    assert_eq!(combat.step_full(&CombatAction::SelectCard { instance: 999 }),
        Err(crate::run::RunError::InvalidAction));
    assert_eq!(combat.state, before, "stale selection preserves parent, queue and payment");

    combat.step_full(&CombatAction::SelectCard { instance: selected_id.as_str().parse().unwrap() }).unwrap();
    assert_eq!(combat.state.phase, Phase::Decision);
    assert_eq!(combat.state.in_play, None);
    assert_eq!(combat.state.piles.hand.iter().filter(|c| c.handle() == selected_id).count(), 1);
    assert_eq!(combat.state.piles.discard.iter().filter(|c| c.handle() == parent_id).count(), 1);
    assert_eq!(combat.state.player.energy, 3, "selection did not repay or redraw");
}

#[test]
fn empty_selection_uses_automatic_branch_and_does_not_deadlock() {
    let mut combat = super::full_runtime::entered_combat(3, 3, |_| {});
    let parent = combat.state.piles.hand.remove(0);
    let parent_id = parent.handle();
    combat.state.in_play = Some(parent);
    combat.state.phase = Phase::PlayerAction;
    combat.state.pending.push_back(Effect { source: Source::Card(parent_id.clone()), target: None,
        kind: EffectKind::SelectCard { candidates: vec![], empty: EmptySelection::Continue } });
    combat.state.pending.push_back(Effect { source: Source::Card(parent_id.clone()), target: None, kind: EffectKind::FinishCard });
    combat.state.pending.push_back(Effect { source: Source::None, target: None, kind: EffectKind::Decision });
    let transition = super::super::engine::resolve(&mut combat.state);
    assert_eq!(transition.status, TransitionStatus::Decision);
    assert_eq!(combat.state.phase, Phase::Decision);
    assert_eq!(combat.state.piles.discard.iter().filter(|c| c.handle() == parent_id).count(), 1);
}

#[test]
fn typed_combat_hooks_follow_card_damage_and_finish_order() {
    let mut combat = super::full_runtime::entered_combat(3, 3, |_| {});
    combat.step_full(&CombatAction::PlayCard { instance: 3, target: Some(TargetId(1)) }).unwrap();
    assert_eq!(combat.state.hook_events.iter().map(|event| event.hook).collect::<Vec<_>>(),
        vec![Hook::CombatStarted, Hook::TurnStartedPreDraw, Hook::TurnStarted, Hook::CardPlayed, Hook::Damage, Hook::CardAfterUse, Hook::CardDiscarded]);
    assert_eq!(combat.state.hook_events[3].source, Source::Card(CardHandle::parse("3").unwrap()));
    assert_eq!(combat.state.hook_events[4].target, Some(Actor::Enemy(EnemyHandle::parse("enemy-1").unwrap())));
}

#[test]
fn lethal_damage_runs_death_and_settlement_hooks_before_earned_healing_cleanup() {
    let mut combat = super::full_runtime::entered_combat(3, 3, |_| {});
    combat.state.player.hp = 31;
    combat.state.enemies[0].hp = 8;
    combat.step_full(&CombatAction::PlayCard { instance: 3, target: Some(TargetId(1)) }).unwrap();
    assert_eq!(combat.state.hook_events.iter().map(|event| event.hook).collect::<Vec<_>>(),
        vec![Hook::CombatStarted, Hook::TurnStartedPreDraw, Hook::TurnStarted, Hook::CardPlayed, Hook::Damage, Hook::Death, Hook::CombatSettled]);
    assert_eq!(combat.state.hp_before_healing, Some(31));
    assert_eq!(combat.state.hp_after_cleanup, Some(31), "no owned Burning Blood means no full-mode heal");
    assert_eq!(combat.state.player.hp, 31);
}
