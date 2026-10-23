use super::*;
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct RunRewardLedger { pub total:i32, highest:[u32;4], keys:[bool;3], terminal:bool }
pub(super) fn key_index(key:Key)->usize {match key {Key::Ruby=>0,Key::Sapphire=>1,Key::Emerald=>2}}
fn calculate(state:&RunState,events:&[CommittedEvent])->Result<RunRewardLedger,RunError>{
    let mut ledger=state.reward_ledger.clone();
    for event in events {
        let amount=match *event {
            CommittedEvent::FloorEntered{act,floor}=>{
                let i=act.checked_sub(1).filter(|i|*i<4).ok_or(RunError::InvalidAction)? as usize;
                if floor>ledger.highest[i] {ledger.highest[i]=floor;if act==4 {10}else{1}}else{0}
            }
            CommittedEvent::KeyAcquired(key)=>{let i=key_index(key);if ledger.keys[i]{0}else{ledger.keys[i]=true;5}}
            CommittedEvent::Ended(terminal)=>if ledger.terminal{0}else{ledger.terminal=true;match terminal{
                RunTerminal::HeartVictory=>100,RunTerminal::Death|RunTerminal::EmptyDeck=>-1,RunTerminal::Act3NoKeys=>0}},
            _=>0,
        };
        ledger.total=ledger.total.checked_add(amount).ok_or(RunError::Unsupported("reward total overflow"))?;
    }Ok(ledger)
}
pub(super) fn preflight(state:&RunState,events:&[CommittedEvent])->Result<(),RunError>{calculate(state,events).map(|_|())}
pub(super) fn commit(state:&mut RunState,events:&[CommittedEvent])->Result<i32,RunError>{
    let ledger=calculate(state,events)?;
    let delta=ledger.total.checked_sub(state.reward_ledger.total).ok_or(RunError::Unsupported("reward delta overflow"))?;
    state.reward_ledger=ledger;state.history.events.extend_from_slice(events);Ok(delta)
}
