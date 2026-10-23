#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EventId { Neow, UpgradeShrine, Purifier, AncientWriting, Joust, Unknown(&'static str) }
impl EventId { pub const fn as_str(self) -> &'static str { match self { Self::Neow => "neow", Self::UpgradeShrine=>"upgrade_shrine",Self::Purifier=>"purifier",Self::AncientWriting=>"ancient_writing",Self::Joust=>"joust",Self::Unknown(s) => s } } }
