# M5 — full generated Act1, explicitly captured content

**Accepted at the declared M5 scope** after one isolated independent milestone review.
This is Ironclad/A0 on the entire native generated Act1 map, not complete standard
random content pools, other characters/ascensions, a full run or original-runtime
validation. The final target remains four characters, A0–20, Acts1–4/Heart.

## Run it

```sh
cargo run -p spire-core --example m5_act1 --locked --offline -- --scenario all --seed 7
cargo run -p spire-core --example m5_act1 --locked --offline -- --scenario guardian --seed 11
```

Scenarios: `guardian`, `hexaghost`, `slime`, `death`, or `all`. `--budget` defaults
to4096 and is capped there. Exhaustion is an unfinished nonzero exit, never a
fabricated win, loss or truncation. The example reads only public maps, current
observations and legal menus; it does not clone hidden simulator state, inject
resources, search for easier seeds, train a model or alter game values.

Executed public results (2026-10-23):

| Seed/scenario | Actions | Real elites | Final HP | Gold | Cards | Reward | Result |
|---|---:|---:|---:|---:|---:|---:|---|
|7 Guardian|212|1 Nob|95/95|246|18|17|Act2Ready, nonterminal|
|7 Hexaghost|171|1 Lagavulin|87/87|246|16|17|Act2Ready, nonterminal|
|7 Slime Boss|223|1 Three Sentries|80/80|336|18|32|Act2Ready, all three keys|
|7 death|10|0|0/88|99|10|0|Native Cultist Death at floor1|
|11 Guardian|250|1 Nob|95/95|253|19|17|Act2Ready, nonterminal|

These outcomes come from accepted actions, not authored combat victories. Both
short Neow choices execute across these paths. Guardian takes Black Blood,
Hexaghost takes Empty Cage with two actual captured removals, Slime takes Collar.
Focused tests also exercise deliberate no-relic skip/warning/cancel/confirmation.
Each full public path replays the same actions, observations and reward deltas.

## Constructor, privacy and captured profile

`Run::new_room_scenario(config, profile, rng, RoomScenario::M5Act1Captured)` calls
ordinary `Run::new`:80/80HP,99gold, five Strikes/four Defends/Bash, Burning Blood,
three empty potion slots and no keys. Require Ironclad/A0, fully unlocked,
Act4-enabled and an independently supplied `neow_eligible=false`. The example
provides valid prior seen lists leaving the named Act1 boss unseen; a seed does
not imply Neow eligibility or boss history.

`PublicDecision.content_profile` declares this scenario. The generated map keeps
all legal edges, six starts, floor9 treasure, floor15 rest and floor16 boss.
First-unseen boss priority is Guardian→Hexaghost→SlimeBoss; all-seen uses a
separate unbiased native shuffle. Only the current act's selected boss icon is
public. Discovery commits at actual prebattle, not map creation. Unresolved
question payloads, future encounters/offer children, boss order and RNG remain
private. Published decisions/offers are detached values.

Captured content is deliberate input, **not registered-subset random sampling**:

- Ordinary encounters: Cultist/Jaw Worm/Two Louse, then repeat Large Slime/
  Three Louse/Lots of Slimes. Elites cycle Nob/Lagavulin/Three Sentries, rotated
  by the selected boss. They run actual native HP/AI/status/split mechanics.
- First two questions capture Upgrade Shrine/Purifier; later ones capture Monster.
- First merchant retains M4 stock; later stock is Clothesline50, Strength Potion50,
  Anchor150, suppressing already-owned relics. Paid removal75 then +25 persists.
- Ordinary treasure captures Strawberry, otherwise Anchor, otherwise Blood Vial,
  first unowned; Sapphire is the exclusive alternative. Exhaustion is explicit.
- Ordinary/elite card triples rotate Iron Wave/Shrug/Anger, Clothesline/Pommel/
  Disarm, Cleave/Twin Strike/Thunderclap. Potions rotate Fire/Strength/Weak.
- Room-source gold is genuinely sampled10–20 ordinary,25–35 elite,95–105 boss.
  Elite relic offers consume the next unowned identity from Lantern/Pear/Mango/
  Magic Flower/Toy Ornithopter/Strawberry/Anchor/Blood Vial at offer preparation,
  not claim. Skipping never restores the cursor or silently removes a drop.
- Boss cards are captured Rare Bludgeon/Impervious/Offering. Floor17 boss chest
  reveals Black Blood/Empty Cage/Slaver's Collar only when opened; no Sapphire.

`Catalog::selectable_pool` remains incomplete. Default full-Neow/random-content
boundaries are not redirected to this profile. The native map is representative,
not an original-Java seed/cursor replica.

## Real effects and ownership

Lagavulin's three sleeping turns/natural wake, damage stun, Attack18 and Siphon
Dex−1 then Str−1 use the existing queue/Artifact owner. Three Sentries retain
slot phases and Artifact1, Bolt Dazed2/Beam9. Dazed is unplayable/Ethereal; held
Burn/Burn+ uses THORNS2/4 through Block without CardPlayed. Burning elites roll
one formation-wide Strength2/rounded25%maxHP/Metallicize4/monster Regen3 buff
before battle-start callbacks/Lament; Emerald remains an independent reward claim.

Guardian's surviving HP loss latches30, then queues defensive change/Block20;
quota rises40/50/60 without excess rollover. Sharp Hide is one THORNS3 per Attack
play, not per hit. Twin Slam's two hits precede queued reset. Hexaghost captures
Divider once, runs its real seven-step cycle and six-hit Inferno, then upgrades
only draw/discard Burns before a later insertion gate. Its future Sear flag is
actor-owned, not a global Burn upgrade. Native Slime Boss replacement is not
victory: actual descendants must die before settlement/loot.

Terminal Combat retains inventory/counters while loot and token frontiers are
prepared; Run then receives the post-hook result once. Full belts retain offers.
Claims/child resumes preflight source/status/identity/resources. Black Blood
replaces Burning Blood at source slot0 and heals12, not18. Cage0 completes its
source while retaining room acknowledgement;1/2 auto-remove all;>2 requires two
captured picks, first removes nothing. Mandatory children cannot be bypassed.
Only settled empty decks lose. Collar temporarily changes master energy before
energy preparation on typed elite/boss sources, not onEquip or a per-turn grant;
activated living victory restores the base master. Existing heal modifiers remain
active when their actual relics are acquired.

Proceed after resolved boss children/confirmed skip resets only act-local state,
requests ordinary heal(maxHP) once, and initializes the same Run's Act2 map and
private boss order. Permanent IDs/upgrades/resources/counters/keys/purge cost/
history/frontiers carry over. `ActAdvanced` credits0, current floor stays17,
next map rooms are18–33. **Act2Ready is nonterminal, has no extra success reward
and exposes zero actions**, including no potion actions or first Act2 encounter.
Full earned totals are17 without keys or32 with all keys. Retained administrative
faults are not Death/readiness and cannot rerun accepted work.

## Verification and limits

```sh
cargo test -p spire-core --test m1_combat --test m2_combat --test m3_run --test m4_rooms --test m5_act1 --locked --offline
cargo test -p spire-core --lib m5_ --locked --offline
cargo test --workspace --locked --offline
cargo test -p spire-env -p spire-python --locked --offline
```

Final controller verification: M1–M5 public36/0, M5-focused54/0, consumers43/0, other
core integrations21/0 and all five milestone examples pass. Warm workspace wall
.161s (build.01s, core execution.14s). Core428/7 retains exactly the original
seven draft names/messages; there is **no whole-workspace-green claim**:

- `catalyst_permanent_card_state_carries_its_exhaust_flag`
- `artifact_prevents_one_poison_application_without_erasing_existing_poison`
- `catalyst_adds_twice_the_current_poison_and_exhausts_but_absence_is_noop`
- `poison_ticks_at_afflicted_enemy_start_ignoring_block_then_decrements_and_can_kill`
- `silent_deadly_poison_applies_base_and_upgraded_amount_for_one_energy`
- `silent_poisoned_stab_resolves_attack_before_base_or_upgraded_poison`
- `blue_slaver_and_two_fungi_encounters_construct_real_actors_across_ascensions`

No complete ordinary/event/Neow/reward/relic pools, other characters/non-A0 runs,
Act2 gameplay, whole-family acceptance, general full snapshot restoration,
Python-extension refresh, dependency installs or original-game execution is
delivered here. This is source-grounded/reference-informed, not original-runtime
validated. Existing supplied-entry batching/Gymnasium interfaces remain separate;
ML models/optimizers/training stay outside this repository. Execution evidence:
`.superpowers/sdd/2026-10-23-m5-ironclad-act1/progress.md`.

One fresh GPT-6 Luna review (`ses_e98dcd1beffernDEPv7qzL3I2H`) in its own
explicitly materialized worktree/unique target returned no findings or declined
judgments and accepted this scope. Controller verified all346 reviewed public
files unchanged in BOTH worktrees before acceptance-status documentation edits,
then reran the checks above. `.27.2`/`.27` closed normally; M6 `.28` and planning-
only `.28.1` are open/unassigned. No M6 product start or new commit is implied.
