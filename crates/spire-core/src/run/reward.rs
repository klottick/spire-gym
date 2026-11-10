use super::*;
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct RunRewardLedger { pub total:i32, highest:[u32;4], keys:[bool;3], terminal:bool }
pub(super) fn key_index(key:Key)->usize {match key {Key::Ruby=>0,Key::Sapphire=>1,Key::Emerald=>2}}
/// M34b: one itemised reward credit (PoC §7). `amount` is the credit this event added.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum RewardItemKind { NewFloor{act:u8,floor:u32}, NewAct4Floor{floor:u32}, FirstKey(Key), HeartVictory, Death, EmptyDeck }
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct RewardItem { pub kind:RewardItemKind, pub amount:i32 }
/// M34b: an opaque copy of the reward ledger, taken before a step so its committed events can be itemised.
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct RewardLedgerSnapshot(RunRewardLedger);
/// The only reward law: each committed event either adds one item or nothing. Revisits, repeated keys,
/// a second terminal and Act3NoKeys add nothing.
fn itemise_into(ledger:&mut RunRewardLedger,events:&[CommittedEvent],items:&mut Vec<RewardItem>)->Result<(),RunError>{
    for event in events {
        let kind=match *event {
            CommittedEvent::FloorEntered{act,floor}=>{
                let i=act.checked_sub(1).filter(|i|*i<4).ok_or(RunError::InvalidAction)? as usize;
                if floor>ledger.highest[i] {ledger.highest[i]=floor;Some(if act==4 {RewardItemKind::NewAct4Floor{floor}}else{RewardItemKind::NewFloor{act,floor}})}else{None}
            }
            CommittedEvent::KeyAcquired(key)=>{let i=key_index(key);if ledger.keys[i]{None}else{ledger.keys[i]=true;Some(RewardItemKind::FirstKey(key))}}
            CommittedEvent::Ended(terminal)=>if ledger.terminal{None}else{ledger.terminal=true;match terminal{
                RunTerminal::HeartVictory=>Some(RewardItemKind::HeartVictory),RunTerminal::Death=>Some(RewardItemKind::Death),
                RunTerminal::EmptyDeck=>Some(RewardItemKind::EmptyDeck),RunTerminal::Act3NoKeys=>None}},
            _=>None,
        };
        if let Some(kind)=kind{
            let amount=match kind{RewardItemKind::NewFloor{..}=>1,RewardItemKind::NewAct4Floor{..}=>10,RewardItemKind::FirstKey(_)=>5,
                RewardItemKind::HeartVictory=>100,RewardItemKind::Death|RewardItemKind::EmptyDeck=>-1};
            ledger.total=ledger.total.checked_add(amount).ok_or(RunError::Unsupported("reward total overflow"))?;
            items.push(RewardItem{kind,amount});
        }
    }Ok(())
}
fn calculate(state:&RunState,events:&[CommittedEvent])->Result<RunRewardLedger,RunError>{
    let mut ledger=state.reward_ledger.clone();itemise_into(&mut ledger,events,&mut Vec::new())?;Ok(ledger)
}
/// M34b: itemise the events one transition committed against the ledger as it was before that transition.
pub fn itemise_rewards(before:&RewardLedgerSnapshot,events:&[CommittedEvent])->Result<Vec<RewardItem>,RunError>{
    let mut ledger=before.0.clone();let mut items=Vec::new();itemise_into(&mut ledger,events,&mut items)?;Ok(items)
}
impl Run{
    /// M34b: snapshot the reward ledger (reads only; consumes no RNG).
    pub fn reward_snapshot(&self)->RewardLedgerSnapshot{RewardLedgerSnapshot(self.state.reward_ledger.clone())}
}
pub(super) fn preflight(state:&RunState,events:&[CommittedEvent])->Result<(),RunError>{calculate(state,events).map(|_|())}
pub(super) fn commit(state:&mut RunState,events:&[CommittedEvent])->Result<i32,RunError>{
    let ledger=calculate(state,events)?;
    let delta=ledger.total.checked_sub(state.reward_ledger.total).ok_or(RunError::Unsupported("reward delta overflow"))?;
    state.reward_ledger=ledger;state.history.events.extend_from_slice(events);Ok(delta)
}
