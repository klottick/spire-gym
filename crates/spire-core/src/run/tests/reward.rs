use crate::{combat::rng::Partition, content::{CardId, Character}, run::*};
use super::super::{mutations, reward};

fn run() -> Run { Run::new(RunConfig { character:Character::Ironclad, ascension:0 }, CreationProfile::experienced(), RunRng::new([7;32],Partition::Test,0)).unwrap() }

#[test]
fn selected_events_credit_once_and_only_selected_outcomes() {
    let mut r=run();
    let floor=CommittedEvent::FloorEntered{act:1,floor:1};
    assert_eq!(reward::commit(&mut r.state,&[floor.clone()]).unwrap(),1);
    assert_eq!(reward::commit(&mut r.state,&[floor]).unwrap(),0);
    assert_eq!(reward::commit(&mut r.state,&[CommittedEvent::FloorEntered{act:1,floor:2}]).unwrap(),1);
    assert_eq!(reward::commit(&mut r.state,&[CommittedEvent::FloorEntered{act:4,floor:1}]).unwrap(),10);
    for key in [Key::Ruby,Key::Sapphire] {
        assert_eq!(reward::commit(&mut r.state,&[CommittedEvent::KeyAcquired(key)]).unwrap(),5);
        assert_eq!(reward::commit(&mut r.state,&[CommittedEvent::KeyAcquired(key)]).unwrap(),0);
    }
    assert_eq!(r.decision().unwrap().reward_total,22);
    for (terminal,expected) in [(RunTerminal::Death,-1),(RunTerminal::EmptyDeck,-1),(RunTerminal::HeartVictory,100),(RunTerminal::Act3NoKeys,0)] {
        let mut r=run();let event=CommittedEvent::Ended(terminal);
        assert_eq!(reward::commit(&mut r.state,&[event.clone()]).unwrap(),expected);
        assert_eq!(reward::commit(&mut r.state,&[event]).unwrap(),0);
    }
}

#[test]
fn finite_mutations_preserve_identity_and_preflight_errors() {
    let mut r=run();
    mutations::gain_gold(&mut r.state,15).unwrap();assert_eq!(r.state.gold,114);
    let id=mutations::obtain_card(&mut r.state,CardId::IronWave,0).unwrap();
    assert_eq!(r.state.deck.iter().filter(|c|c.instance==id).count(),1);
    let strike=r.state.deck.iter().find(|c|c.id==CardId::Strike).unwrap().instance;
    mutations::upgrade_card(&mut r.state,strike).unwrap();
    let before=r.clone();assert!(mutations::upgrade_card(&mut r.state,strike).is_err());assert!(r==before);
    let removed=mutations::remove_card(&mut r.state,strike).unwrap();assert_eq!((removed.instance,removed.upgrades),(strike,1));
    assert_eq!(r.state.deck.iter().filter(|c|c.id==CardId::Strike).count(),4);
    r.state.hp=40;r.state.max_hp=88;mutations::heal(&mut r.state,26).unwrap();assert_eq!(r.state.hp,66);
    r.state.hp=80;mutations::heal(&mut r.state,26).unwrap();assert_eq!(r.state.hp,88);
    r.state.hp=40;r.state.max_hp=80;mutations::increase_max_hp(&mut r.state,8).unwrap();assert_eq!((r.state.hp,r.state.max_hp),(48,88));
    r.state.next_instance=u64::MAX;let before=r.clone();assert!(mutations::obtain_card(&mut r.state,CardId::IronWave,0).is_err());assert!(r==before);
}

#[test]
fn ledger_overflow_preflights_without_history_or_key_mutation() {
    let mut r=run();r.state.reward_ledger.total=i32::MAX;
    let before=r.clone();assert!(mutations::acquire_key(&mut r.state,Key::Ruby).is_err());assert!(r==before);
    assert!(reward::commit(&mut r.state,&[CommittedEvent::FloorEntered{act:1,floor:1}]).is_err());assert!(r==before);
}

#[test]
fn terminal_source_credits_transition_and_retains_total() {
    let mut r=run();let transition=r.commit_terminal(RunTerminal::Death).unwrap();
    assert_eq!(transition.reward,-1);assert_eq!(r.decision().unwrap().reward_total,-1);
    assert_eq!(r.state.history.events,vec![CommittedEvent::Ended(RunTerminal::Death)]);
}
