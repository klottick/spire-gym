# M6 — Act2 mechanics and an early same-run prefix

**Accepted at the declared M6 scope after one isolated independent milestone review.**
Ironclad/A0 only, with explicitly captured content. This is not complete Act2,
standard pools, all-character/Ascension acceptance, or original-runtime validation.
M7 retains the full Act2 boss/reward/Act3 path; the final four-character A0–20,
Acts1–4/Heart target is unchanged.

## Run it

```sh
cargo run -p spire-core --example m6_act2 --locked --offline -- --scenario all --seed 7
cargo run -p spire-core --example m6_act2 --locked --offline -- --scenario prefix --seed 11
cargo test -p spire-core --test m6_act2 --locked --offline
```

Scenarios: `prefix`, `encounters`, `death`, `all`. Budget defaults to4096 and cannot
exceed it; budgets0/1 return unfinished nonzero exits without invented termination.
The caller uses only public observations, legal actions and generated-map topology.
It does not clone hidden simulators, train, search for easier seeds or alter game
values. Supplied-resource diagnostics are explicitly separate from normal runs.

## Executed results — 2026-10-23

| Seed / Act2 icon | Accepted actions | Final HP | Reward | Result |
|---|---:|---:|---:|---|
|7 Champ / Leader elite|399|49|26|floor26, alive/nonterminal/no actions|
|7 Automaton / Slavers elite|386|87|26|same cutoff|
|7 Collector / Book elite|378|95|26|same cutoff|
|11 Champ|418|76|26|same cutoff|
|11 Automaton|417|99|26|same cutoff|
|11 Collector|416|105|26|same cutoff|

Each starts at ordinary80/80HP,99gold, five Strikes/four Defends/Bash, Burning
Blood, three empty potion slots/no keys. Independently supplied short-Neow and
valid prior boss-seen lists select the icons; no post-start resource/outcome
injection. Each seed7 path includes at least two actual Act2 ordinary combats,
the named elite, Ancient Writing, and actual floor26 chest resolution. Accepted
actions, reward deltas, maps and public decisions replay exactly. The caller takes
the offered Slaver's Collar: its real elite energy, not extra injected resources.

The deliberately nonattacking normal-start path dies to Cultist at floor1 in10
actions, reward0 (floor credit1 plus Death−1). Six separately supplied300HP,
upgraded12-card diagnostics all win through actual native actions/exit: Leader23,
Slavers24, Book24, Champ58, Collector62, Automaton47 actions at seed7. They are
not map progression, Act2 boss-room/reward completion or authored victories.

Earlier fixedseed7 Leader policy losses and the real nonfinal-TwinStrike queue
defect are preserved in the execution ledger. The latter has a native RED→GREEN
regression; caller survival priority has a real-entry regression. Policy losses
were not replaced with a near-end fixture or hidden by seed/value changes.

## Captured profile and boundaries

`Run::new_room_scenario(..., RoomScenario::M6Act2Prefix)` shares accepted M5 Act1
content/topology. Its actual Act1→Act2 source transition initializes/heals/maps
once, keeping earned identities, inventory/counters, paid-purge cost and rewards.
Only M6 continues; **M5 remains nonterminal/no-action Act2Ready**. Act2 retains
all generated edges through floor26. Actual chest Proceed, including an explicit
optional-loot skip, gives `Act2PrefixComplete`: no actions/potions/bonus/terminal.
Unresolved children block source progression; mere floor26 credit is not completion.

- Ordinary visits: Sphere, Three Byrds, Centurion/Mystic, then Snecko,
  Chosen/Byrd, Three Cultists, Centurion/Mystic cycle with last-two exclusions.
- Elites: Leader/Slavers/Book cycle rotated by the selected icon, never immediate
  repeats. Their HP, AI, summons and damage are genuine native mechanics.
- Questions: Ancient Writing, then Joust when gold≥50 (otherwise Purifier), then
  Monster. These are named captures, not a standard event distribution.
- Card/potion captures reuse M5 triples and Fire/Strength/Weak by **Act2-local
  settled visits**. Gold is10–20 ordinary/25–35 elite. Actual room origin controls
  Collar/burning/Emerald/relics, not enemy identity. Elite offers start at the first
  unowned captured relic and advance at preparation even when skipped.
- Shops use later-M5 stock/prices and carried purge cost. Ordinary chest captures
  first-unowned Strawberry/Anchor/Blood Vial or exclusive Sapphire. Pool exhaustion
  remains explicit, not a silently missing drop.

No Act2 boss map room, boss chest, Act3 transition/readiness, or full-act success
is emitted. Unlisted enemy/event/card/relic pools remain incomplete, not filtered
into a supposedly standard random pool. Reaper itself remains unsupported.

## Mechanics and ownership

Flight counts actual NORMAL hits, including blocked/zero damage, with queued
ground/reset reactions. Hex inserts Dazed into shuffled draw; Confused changes
eligible actual-draw costs locally, not permanent cost or Stasis return. Centurion/
Mystic and bosses select from live partner/HP/group context at the real checkpoint.
Slavers retain execution-spent Entangle/Artifact/manual-Attack restriction/expiry;
Book uses growing finite multihit work and qualifying owner-attributed Wounds.

Summons keep append-only target IDs, logical turn slots, fresh constructor/AI
streams and staged partial-fault custody. Leader's living adds actually escape
with HP retained. Collector and Automaton independently qualify for queued zero-
gold suicides with real death callbacks, **not** a predicate ignoring live Minions.
Accepted player healing survives cleanup; new adds cannot join an old turn snapshot.

Stasis owns exactly one live card per Orb outside ordinary piles. Capture selects
highest rarity in nonempty draw, otherwise discard; ties use owned native RNG.
Same-ID return preserves explicit copied costs/free-play/upgrades, uses hand space
or discard overflow, and never invokes draw/Confused or issues a second card.
Known constructor-backed exhaust is restored, not arbitrary authored overrides.
Death exports surviving held cards into the Run permanent-membership census;
boss cleanup resolves actual returns before settlement. Public views show only
already captured cards, not remaining order, future rolls/births/loot.

Ancient Writing uses actual starter Strike/Defend tags and can-upgrade behavior,
not Bash/nonbasic names. Elegance captures/removes one identity, with no-eligible
source completion and settled EmptyDeck handling. Unrepresented bottled/curse
exclusions are not claimed implemented. Joust's qualified special caller admits
gold≥50; wager pays50, Continue samples one shared winner, result button pays gross
100/250, completion opens map. Native tickets29/30 test both sides independently.

All qualification is bounded static/reference evidence, **not original execution**.
Native HMAC streams/canonical tie/slot traversal are deterministic project contracts,
not original Java seed/cursor/comparator/HashMap traversal equivalence. Operational
faults retain source owners/cursors/accepted work; rejected retries cannot reroll,
duplicate cards/payments or masquerade as gameplay losses.

## Verification status

Final bounded matrix executed: native M6 90/0, public M1–M6 42/0 (M6 6/0), other
core integrations21/0, consumers43/0, all six examples and diffcheck pass. The
single independent GPT-6 Luna review found no Critical/Important/Minor issues and
recommended acceptance of this exact profile. Its four declined categories remain
explicit limits: original runtime/Java parity, later roadmap scope, unsupported
handlers/flags, and historical workspace failures. Controller verified all372
reviewed public files unchanged in both worktrees and independently reran the
matrix/examples before acceptance. Only administrative acceptance text changed
after that verification; no product fix or second review was needed.
The prior core baseline contains seven unrelated Poison/Catalyst/Artifact and
old Slaver/Fungi draft failures. Compare names/messages, not counts alone; only
the documented absent optional spawn diagnostic is normalized. No whole-workspace-
green, commit, installation, original execution or cleanup claim is made.
