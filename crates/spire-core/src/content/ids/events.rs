#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EventId { Neow, Unknown(&'static str) }
impl EventId { pub const fn as_str(self) -> &'static str { match self { Self::Neow => "neow", Self::Unknown(s) => s } } }
