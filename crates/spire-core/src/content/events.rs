use super::EventId;

#[derive(Clone, Copy, Debug)]
pub struct EventDefinition { pub id: EventId, pub act: u8, pub definition_only: bool }
pub(crate) const EVENTS: &[EventDefinition] = &[
    EventDefinition { id: EventId::Neow, act: 0, definition_only: true },
    EventDefinition {id:EventId::UpgradeShrine,act:1,definition_only:false},
    EventDefinition {id:EventId::Purifier,act:1,definition_only:false},
    EventDefinition {id:EventId::AncientWriting,act:2,definition_only:false},
    EventDefinition {id:EventId::Joust,act:2,definition_only:false},
];
