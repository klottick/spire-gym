//! Owned card-selection suspension on the current resolver queue.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EmptySelection { Continue }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectionDisposition { Hand, Discard, Exhaust }
