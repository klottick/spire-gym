use crate::{content::Character, run::*};

fn run() -> Run { Run::new(RunConfig { character: Character::Ironclad, ascension: 0 }, CreationProfile::experienced(),
    RunRng::new([1; 32], crate::combat::rng::Partition::Test, 0)).unwrap() }

#[test]
fn invalid_action_preserves_rng() {
    let mut run = run();
    let before = run.clone();
    assert!(run.step(&GameAction::Confirm).is_err());
    assert_eq!(run, before);
}

#[test]
fn selected_offer_commits_typed_payload_then_faults_without_retry() {
    let mut run = run();
    run.record_event_for_test(CommittedEvent::FloorEntered{act:1,floor:0});
    let decision=run.decision().unwrap();
    assert_eq!(decision.offers.iter().map(|o|o.category).collect::<Vec<_>>(),vec![0,1,2,3]);
    assert!(decision.offers[2].drawback.is_some());
    let candidate=decision.candidates[0].clone();
    let offer=candidate.neow_offer.clone().unwrap();
    let action=candidate.action.clone();
    assert!(matches!(run.step(&action),Err(RunError::Faulted(_))));
    assert_eq!(run.state().selected_neow.as_ref(),Some(&offer));
    let fault=run.state().fault.clone().unwrap();
    assert_eq!(fault.committed_events,vec![CommittedEvent::FloorEntered{act:1,floor:0}]);
    let committed=run.clone();
    assert_eq!(run.step(&action),Err(RunError::Faulted(fault)));
    assert_eq!(run.step(&GameAction::Confirm),Err(RunError::Faulted(run.state().fault.clone().unwrap())));
    let faulted=run.clone();
    assert!(matches!(run.remove_card(1),Err(RunError::Faulted(_))));
    assert_eq!(run,faulted);
    assert_eq!(run,committed);
}

#[test]
fn nested_selection_resumes_captured_parent_and_exact_instance_once_through_step() {
    let mut run=run();
    let parent=Candidate::internal_test_parent(ChoiceId(77));
    let duplicate_kind=run.state().deck.iter().filter(|c|c.id==crate::content::CardId::Strike).map(|c|c.instance).take(2).collect::<Vec<_>>();
    run.install_pending_selection_for_test(parent.clone(),duplicate_kind.clone(),2,EmptySelectionRule::Fault);
    let selection=run.decision().unwrap();
    assert_eq!(selection.selection.as_ref().unwrap().required,2);
    assert_eq!(selection.selection.as_ref().unwrap().eligible.len(),2);
    let chosen=selection.candidates[1].action.clone();
    assert!(matches!(run.step(&chosen),Ok(_)));
    let intermediate=run.decision().unwrap();
    assert_eq!(intermediate.selection.as_ref().unwrap().selected_count,1);
    assert_eq!(intermediate.selection.as_ref().unwrap().eligible.len(),1);
    assert_eq!(run.state().resume_count,0);
    let final_choice=intermediate.candidates[0].action.clone();
    assert!(matches!(run.step(&final_choice),Ok(_)));
    let resumed=run.decision().unwrap();
    assert_eq!(resumed.candidates,vec![parent.clone()]);
    let committed=run.state().last_selection.as_ref().unwrap();
    assert_eq!(committed.iter().map(|c|c.instance).collect::<Vec<_>>(),vec![duplicate_kind[1],duplicate_kind[0]]);
    assert_eq!(run.state().last_resumed_parent,Some(parent.id));
    assert_eq!(run.state().resume_count,1);
    assert!(matches!(run.step(&GameAction::Select(SelectionId(2))),Err(RunError::InvalidAction)));
    assert_eq!(run.state().resume_count,1);
}

#[test]
fn zero_eligible_selection_auto_resumes_parent_without_deadlock() {
    let mut run=run();
    let parent=Candidate::internal_test_parent(ChoiceId(88));
    run.install_empty_selection_for_test(parent.clone());
    assert_eq!(run.step(&GameAction::Confirm),Ok(RunTransition{reward:0,terminal:None,committed_events:vec![]}));
    assert_eq!(run.decision().unwrap().candidates,vec![parent.clone()]);
    assert_eq!(run.state().last_resumed_parent,Some(parent.id));
    assert_eq!(run.state().resume_count,1);
}

#[test]
fn zero_eligible_fault_rule_retains_first_fault_without_parent_retry() {
    let mut run=run();
    let parent=Candidate::internal_test_parent(ChoiceId(89));
    run.install_empty_fault_selection_for_test(parent);
    let fault=run.step(&GameAction::Confirm).unwrap_err();
    assert!(matches!(fault,RunError::Faulted(_)));
    assert_eq!(run.step(&GameAction::Confirm),Err(fault));
    assert_eq!(run.state().resume_count,0);
}

#[test]
fn terminal_is_readable_and_distinct_from_retained_fault() {
    let mut completed=run();
    completed.install_terminal_for_test(RunTerminal::Death);
    let decision=completed.decision().unwrap();
    assert_eq!(decision.terminal,Some(RunTerminal::Death));
    assert!(decision.candidates.is_empty());
    let mut faulted=run();
    faulted.install_fault_for_test(RunFault{kind:RunFaultKind::UnsupportedBoundary("fixture"),checkpoint:"fixture",committed_events:vec![]});
    assert!(matches!(faulted.decision(),Err(RunError::Faulted(_))));
}

#[test]
fn public_projection_contains_player_objects_and_offers_but_hides_private_profile() {
    let run=run();
    let public=run.decision().unwrap();
    assert_eq!(public.player.hp,80);
    assert_eq!(public.player.max_hp,80);
    assert_eq!(public.player.gold,99);
    assert_eq!(public.player.keys,[false;3]);
    assert_eq!(public.objects.len(),10);
    assert_eq!(public.offers.len(),4);
    assert!(!format!("{public:?}").contains("IronWave"));
}

#[test]
fn stale_selection_is_rejected_without_mutating_pending_capture() {
    let mut run=run();
    let parent=Candidate::internal_test_parent(ChoiceId(90));
    let instance=run.state().deck[0].instance;
    run.install_pending_selection_for_test(parent,vec![instance],1,EmptySelectionRule::Fault);
    let action=run.decision().unwrap().candidates[0].action.clone();
    run.remove_card(instance).unwrap();
    let before=run.clone();
    assert_eq!(run.step(&action),Err(RunError::InvalidAction));
    assert_eq!(run,before);
}

#[test]
fn selected_instance_is_present_in_public_candidate_view() {
    let mut run=run();
    let parent=Candidate::internal_test_parent(ChoiceId(91));
    let instance=run.state().deck[0].instance;
    run.install_pending_selection_for_test(parent,vec![instance],1,EmptySelectionRule::Fault);
    let candidate=&run.decision().unwrap().candidates[0];
    assert_eq!(candidate.object.as_ref().unwrap().instance,instance);
}

#[test]
fn resumed_neow_parent_dispatches_captured_offer_to_unwired_fault_without_rechoosing() {
    let mut run=run();
    let offer=NeowOffer{category:2,reward:NeowReward::Gold250,drawback:Some(NeowDrawback::NoGold)};
    let instance=run.state().deck[0].instance;
    run.install_neow_selection_for_test(offer.clone(),vec![instance]);
    let action=run.decision().unwrap().candidates[0].action.clone();
    assert!(matches!(run.step(&action),Err(RunError::Faulted(_))));
    assert_eq!(run.state().selected_neow,Some(offer));
    assert_eq!(run.state().last_selection.as_ref().unwrap()[0].instance,instance);
    assert!(run.decision().is_err());
}

#[test]
fn experienced_profile_is_owned_and_supplies_note_bosses_and_act4() {
    let mut profile=CreationProfile::experienced();
    let original=profile.clone();
    let run=Run::new(RunConfig{character:Character::Ironclad,ascension:0},profile.clone(),RunRng::new([1;32],crate::combat::rng::Partition::Test,7)).unwrap();
    profile.seen_bosses[0].clear();
    profile.stored_note=(crate::content::CardId::Strike,99);
    assert_eq!(run.state().seen_bosses,original.seen_bosses);
    assert_eq!(run.state().stored_note,(crate::content::CardId::IronWave,0));
    assert!(run.state().act4_enabled);
}
