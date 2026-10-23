# Run-content registry foundation

`spire_core::content::Catalog::standard()` is the static typed registry
foundation. The four standard characters have exact fresh, upgrade-0 base decks
and starter relic identities. `CardId` includes distinct character-color starter
identities while preserving the legacy red `strike`, `defend`, and `bash` IDs.
The A10+ Ascender's Bane definition and Iron Wave Note default are also present;
the run-start ascension rule is owned by the run component.

Card definitions separate `CardColor`, `CardRarity`, and `CardType`, and are
marked `definition_only`; that marker explicitly does not claim executable
full-run mechanics. Named producer eligibility is queried with
`Catalog::eligibility(card, producer, character)`: the current foundation names
ordinary rewards, Shard-widened ordinary rewards, Note for Yourself, and Pure
Water. Basics are excluded from both ordinary-reward routes; Note's removable
permanent-card domain is distinct from event blanket eligibility and is encoded
as required identity-level `NoteStorage` metadata, not inferred from color or
rarity. This permits supported off-color basics, colorless Madness, and removable
curse Regret, while rejecting temporary Miracle, Status Wound, and nonremovable
Ascender's Bane. Pure Water generates Miracle rather than an arbitrary card. An
unknown card or producer returns `RegistryError`. `starter_deck` returns a
`Result` with `MissingStarterDeck` for a missing mapping, never an empty fallback.
Full selectable pools return
`RegistryError::IncompletePool` until their complete family tables are populated,
so partial registration cannot become a smaller valid random pool.

Current definition rows include the four starts, Anger, Iron Wave, Ascender's
Bane, Miracle, Wound, Madness, Regret, four starter relic identities, Fire Potion, Neow, and the
existing supplied-entry Cultist/Jaw Worm and encounter identities. Relic,
potion, enemy, encounter, and event records are definition-only. The supplied
combat engine's narrow behavior is not promoted to full-run support. The
registry is extensible, not a complete playable catalog: later card/item/
encounter/event owners extend their own families alongside mechanics. Normally
reachable full-game coverage remains required at final integration; this
foundation is not a reduced-profile release.

Bounded source-grounded facts reused: existing public authored Act 1 research
for the four basic-card identities, red start, Anger/Iron Wave, Miracle, and
producer distinctions; the four-character starting counts and starter relics,
plus A10 Ascender's Bane, were checked against the pinned public reference pages
listed in the task context. These are reference-informed facts, not original
runtime validation. No source census or original-game execution was performed.
The existing `combat::CONTENT_ID`, string vocabulary, and combat behavior are
unchanged.
