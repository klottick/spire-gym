#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EventId { Neow, UpgradeShrine, Purifier, Unknown(&'static str) }
impl EventId { pub const fn as_str(self) -> &'static str { match self { Self::Neow => "neow", Self::UpgradeShrine=>"upgrade_shrine",Self::Purifier=>"purifier",Self::Unknown(s) => s } } }
