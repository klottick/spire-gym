//! Owned card-selection suspension on the current resolver queue.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EmptySelection { Continue }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectionDisposition { Hand, Discard, Exhaust, DrawTop }

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum SelectionSourceZone { Hand, Draw, Discard, Exhaust }

#[derive(Clone,Debug,PartialEq,Eq)]
pub enum SelectionParent {
    Card(super::CardHandle),
    Potion { slot:usize,id:crate::content::PotionId },
    Relic(crate::content::RelicId),
    /// One optional card choice suspended under the live player EndTurn queue.
    EndTurn { turn:u32 },
    Power { turn:u32, id:super::FullPowerId },
}

/// A nonmoving upgrade is not an exhaust/discard with a cosmetic label.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum SelectionOperation { Scry, MeditateReturn, HologramRetrieve, SeekRetrieve, Recycle, Move(SelectionDisposition), Upgrade, Topdeck, ReturnFromExhaust, CopyToHand{copies:u8}, Retain, NightmareCapture, Setup, Bottomdeck, RetrieveMemories,
    /// Omniscience's draw-pile choice (identity only until its behaviour lands).
    Omniscience }

/// Record the continuation that a selection opening now interrupts. The
/// resolver stamps the once-per-resolution marks it had already spent.
pub(super) fn suspend(state:&super::model::State)->super::model::SelectionResume{
    super::model::SelectionResume{phase:state.phase,in_play:state.in_play.as_ref().map(super::model::OwnedCard::handle),
        frame:state.play_frames.last().map(|frame|(frame.child.clone(),frame.parent_owner.clone())),
        marks:super::model::ResolveMarks::default()}
}

/// One rule for every suspended selection: its record still names the live
/// in-play card and top play frame, and the interrupted phase still satisfies
/// the busy-continuation invariant `validate_runtime` applies to that phase.
pub(super) fn validate_resume(state:&super::model::State,resume:&super::model::SelectionResume)->Result<(),super::model::InputError>{
    use super::model::{EffectKind,OwnedCard,Phase};
    let live_frame=state.play_frames.last().map(|frame|(&frame.child,&frame.parent_owner));
    let valid=!matches!(resume.phase,Phase::Selection|Phase::Terminal)
        &&resume.in_play==state.in_play.as_ref().map(OwnedCard::handle)
        &&resume.frame.as_ref().map(|(child,owner)|(child,owner))==live_frame
        &&(resume.phase==Phase::Decision||!state.pending.is_empty())
        &&(resume.in_play.is_none()||resume.phase==Phase::PlayerAction
            &&state.pending.iter().any(|work|matches!(work.kind,EffectKind::FinishCard|EffectKind::Terminal(_))))
        &&(!resume.marks.hand_emptied||resume.phase==Phase::Decision);
    if valid{Ok(())}else{Err(super::model::InputError::Invariant("selection lost its suspended continuation".into()))}
}
