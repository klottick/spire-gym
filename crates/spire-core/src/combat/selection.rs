//! Owned card-selection suspension on the current resolver queue.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EmptySelection { Continue }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectionDisposition { Hand, Discard, Exhaust, DrawTop }

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum SelectionSourceZone { Hand, Draw, Discard, Exhaust }

#[derive(Clone,Debug,PartialEq,Eq)]
pub enum SelectionParent { Card(super::CardHandle), Potion { slot:usize,id:crate::content::PotionId } }

/// A nonmoving upgrade is not an exhaust/discard with a cosmetic label.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum SelectionOperation { Move(SelectionDisposition), Upgrade, Topdeck, ReturnFromExhaust, CopyToHand{copies:u8} }
