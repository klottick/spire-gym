# M4 — authored Act1 room loop

Accepted M4 connects standard Ironclad A0 starting resources to an explicitly
authored nine-floor prefix. It is **not normal random map/offer generation,
Act1/boss completion, a full run, or original-runtime validation**. The final
target remains four characters, A0–20, Acts1–4 and the Heart.

## Run it

```sh
cargo run -p spire-core --example m4_rooms --locked --offline -- --scenario all --seed 7
```

From any directory (the subshell leaves your current directory unchanged):

```sh
(cd /home/klottick/.local/share/opencode/worktree/503d4a/full-run-gym-poc-20261022 && cargo run -p spire-core --example m4_rooms --locked --offline -- --scenario all --seed 7)
```

Choose `route`, `purge`, `keys`, or `death` instead of `all`. Seed7 observed:

| Scenario | Accepted actions | Fights | Final HP | Gold | Cards | Reward | Result |
|---|---:|---:|---:|---:|---:|---:|---|
| route | 55 | 3 | 95/95 | 67 | 13 | 9 | CheckpointComplete, no terminal |
| purge | 43 | 3 | 87/87 | 67 | 11 | 9 | CheckpointComplete, no terminal |
| keys | 39 | 3 | 80/80 | 142 | 12 | 19 | Ruby+Sapphire, no terminal |
| death | 12 | 1 | 0/88 | 99 | 10 | 0 | Native first-Jaw Death |

These are executed public-action results, not fixed combat outcomes. The caller
uses only current public choices/cards/targets; it never injects resources after
construction. Its512-action budget reports an error if unfinished, never a
fabricated win/loss/truncation. The native simulation has no new gameplay clock.

## Public interface and precise profile

`Run::new_room_scenario(config, profile, rng, RoomScenario::M4Act1Prefix)` requires
Ironclad/A0, fully unlocked/Act4-enabled and independently specified
`profile.neow_eligible=false`. It calls ordinary `Run::new`:80/80HP,99gold,
five Strikes/four Defends/Bash, Burning Blood and three empty potion slots.
A project replay seed does not imply original custom-seeded Neow history.

Both actual short blessings work: captured +8maxHP with ordinary heal8, or
Neow's Lament with three battles. Lament sets every initial monster's currentHP1
without changing maxHP/dealing damage; counters3→2→1→−2 persist through combat,
potion use and settlement. It spends one charge per formation, not per monster.

The authoritative existing map stores a two-start choice (Cultist/Jaw), Upgrade
Shrine, merchant, the other weak encounter, Purifier, campfire, Two Louse,
second merchant and treasure. Enemies use native setup/HP/AI and the shared
combat resolver. Unrevealed questions and unopened chest/card payloads remain
private; retained public decisions are detached values.

Combat victories materialize gold10–20 using an independent semantic RNG scope,
captured `[IronWave,ShrugItOff,Anger]` card choices and captured Fire Potion.
Merchant stock is explicitly captured Iron Wave25, Fire Potion50 and Blood
Vial150. These are **authored offers, not reduced ordinary random pools**.
`Catalog::selectable_pool` remains explicitly incomplete. Default/full-Neow and
generated-world unsupported boundaries are not silently redirected here.

`Run::decision()` adds typed `room` choices/status/prices and `reward_total`;
`candidates` is the legal-action list. `GameAction::Choose`, `Move`, captured
`Select`/source-legal `Cancel`, `Combat`, and outside-combat potion use/discard
drive progression. `RunTransition.reward` is the credited delta, not gold earned.
Only visible Lament counters are exposed; opaque stored counters remain private.

## Ownership and consequences

- The existing Run owns permanent identities/deck/resources. Its single Combat
  temporarily owns inventory **and counters**, then returns post-hook values once.
  Burning Blood is already applied by Combat; Run settlement never heals again.
- Loot is prepared before releasing terminal Combat. Offers/read/open are not
  acquisitions. Claims cannot duplicate cards/items; a full potion belt leaves
  the reward available until a slot is freed or the caller Proceeds.
- Purge costs75 then +25 per successful run-wide paid removal, once per merchant.
  Opening/cancelling the grid spends nothing. Accepted selection removes the
  exact captured permanent instance and returns to the same merchant.
- Upgrade Shrine and Purifier have real initial Leave choices; committed shrine
  grids are not arbitrarily cancellable. Smith Cancel returns to the campfire;
  accepted Smith upgrades the original card. Tokens never restart per grid.
- Rest heals floor(binary32 maxHP×0.3), capped at maxHP, before `Rested` dispatch.
  Recall grants Ruby instead of Rest/Smith. Chest Open reveals Strawberry versus
  the exclusive Sapphire alternative; Strawberry runs native equip +7maxHP/heal7.
- New highest Act1 floors credit+1; first actual key acquisitions+5; native
  Death/settled EmptyDeck−1. Other purchases, loot gold, actions and reads add0.
  Prefix completion has no win bonus and is not a RunTerminal.
- Invalid input/preflight failures preserve state/RNG. Accepted startup,
  settlement and reward-preparation faults retain the first fault/actual owner;
  they cannot rerun entry/heal/acquisition and are not scored as gameplay deaths.

## Verification and limits

```sh
cargo test -p spire-core --lib run::tests --locked --offline
cargo test -p spire-core --lib combat::tests::shared_mechanics --locked --offline
cargo test -p spire-core --test m1_combat --test m2_combat --test m3_run --test m4_rooms --locked --offline
```

The public M4 cases exercise the entire prefix, real loss, key/relic alternatives,
paid-purge replay and caller budgets. Small native fixtures supplement these with
exact arithmetic/identity, source-specific cancellation, full belts, hidden views,
counter ownership/expiry, real upgraded-Bash consumption and retained faults.
Prior M1–M3 consumers remain required. Final core374/7 retains exactly the seven
pre-M4 Poison/Catalyst/Artifact and Slaver/Fungi failure names/panic messages;
no workspace-green claim. Run58/0, shared10/0, public M1–M4 32/0, accepted
consumers43/0, other integrations21/0 and all four examples pass.

No full offer pools, Act1 boss, general snapshot restoration, Python-extension
refresh or original-game execution is delivered here. Source-grounded/
reference-informed only. One independent GPT-6 Luna milestone review in its own
materialized worktree found no findings or declined judgments. All328 reviewed
files were unchanged in both worktrees and fresh focused checks/diffcheck passed
before normal `.26.2`/`.26` closure. Subsequent acceptance-status edits are docs
only. M5 `.27`/planning `.27.1` is next, not started product work. Execution ledger:
`.superpowers/sdd/2026-10-23-m4-act1-room-loop/progress.md`.
