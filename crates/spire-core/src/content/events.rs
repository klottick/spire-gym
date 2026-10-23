use super::EventId;

#[derive(Clone, Copy, Debug)]
pub struct EventDefinition { pub id: EventId, pub act: u8, pub definition_only: bool }
pub(crate) const EVENTS: &[EventDefinition] = &[EventDefinition { id: EventId::Neow, act: 0, definition_only: true }];
