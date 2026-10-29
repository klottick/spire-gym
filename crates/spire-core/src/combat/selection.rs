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
    /// One optional card choice suspended under the live player EndTurn queue.
    EndTurn { turn:u32 },
    Power { turn:u32, id:super::FullPowerId },
}

/// A nonmoving upgrade is not an exhaust/discard with a cosmetic label.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum SelectionOperation { Scry, MeditateReturn, HologramRetrieve, SeekRetrieve, Recycle, Move(SelectionDisposition), Upgrade, Topdeck, ReturnFromExhaust, CopyToHand{copies:u8}, Retain, NightmareCapture, Setup, Bottomdeck, RetrieveMemories }
