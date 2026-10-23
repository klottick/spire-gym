# Enemy and encounter correctness reference

Compiled 2026-10-23 for **unmodded Slay the Spire 1, PC v2.3**, all four
standard characters, Acts 1–4 and Ascension 0–20. Reference only: **not an
implemented enemy registry, executable AI sampler, or original-runtime proof**.
No game code, binaries, assets, dependencies or private evidence are included.

## Contents

- [`enemies.json`](enemies.json): **65 enemy kinds, 196 moves**, HP bands,
  starting powers/Block, per-hit damage, payloads/card destinations, opening and
  continuation rules, contextual chances, Ascension changes, lifecycle and variants.
- [`encounters.json`](encounters.json): **67 encounter/pool-context records**,
  including event combats, formation/composition rules, relative selection weights,
  queue exclusions, boss discovery, A20 double boss and burning-elite distinctions.
- [`sources.json`](sources.json): revision-pinned community pages, qualified local
  research, bounded secondary-port cross-checks and [M8 task-local static findings](m8-static-findings.md).
- [`test_enemy_reference_dataset.py`](../../tests/python/test_enemy_reference_dataset.py):
  Small offline checks of reference integrity and selected numeric distinctions.

The roster includes bosses, summons, bandits, both Louse colors and all Slime
sizes. Hard Jaw Worm, awake event Lagavulin, center/outer Darkling and
leader-generated gremlins are **variants**, not duplicate species. Apology Slime
is mod content and excluded. Noncombat damage events such as Scrap Ooze are not
enemy kinds. A room's elite/reward status is not the same as each actor's class:
Blue/Red Slavers retain normal-enemy scaling beside Elite-class Taskmaster.

## Reading the records

| Field | Meaning |
|---|---|
| `key`, `id` | Stable reference key and uppercase convenience label; not native enum IDs |
| `game_id`, `identity_status` | 25 primary-catalog IDs, thirteen owned-static IDs with unverified upstream build, and 27 **provisional** constructor labels |
| `acts`, `category` | Reachable act contexts and ordinary actor class; not room/reward classification |
| `hp.bands` | Inclusive Ascension and ordinary inclusive HP ranges; split-created HP overrides normal rolls |
| `starting_state` | Intrinsic ordinary setup; contextual variants and room/relic effects are separate |
| `moves[].damage` | Unmodified damage **per hit**, hit counts, fixed-per-instance ranges or readable formulas |
| `moves[].effect`, `target` | Readable effects, Block, powers, card destinations and recipient distinctions |
| `moves[].wiki_intent` | Wiki display token, **not** an exact game intent enum or kind-local move byte |
| `ai.opening`, `history_and_state_rules` | Opening, cycles, repeat redirects, selection/execution flags and phase/live-state conditions |
| `ai.nominal_choices` | Contextual rational **proposal** weights, explicitly graded nominal/approximate |
| `ai.exact_a0_*` | Retained exact Jaw/Acid local A0 laws, not new all-Ascension evidence |
| `ai.ascension_changes` | Additional moveset/power thresholds; numeric payload changes also appear in moves/setup |
| `lifecycle`, `variants` | Interruptions, phases, splitting, regrowth, summons, escape, reactions and special setups |
| `local_a0`, `citations`, `uncertainties` | Preserved authored findings, evidence grade and concrete unresolved conflicts |

`A<N>` in readable effects means **Ascension N and above**. Ascension modifiers
are cumulative. HP thresholds normally follow A7/A8/A9 for normal/Elite/Boss
classes; Bronze Orb and Torch Head follow their parent boss's A9 family, with
secondary rather than primary corroboration here. Damage bases exclude Strength,
Weak/Vulnerable, Flying, Back Attack and other live modifiers. A range for Louse
Bite or Darkling Nip is rolled once per actor, not each hit or turn.
Exploder's owned-static fixed30 THORNS/self-suicide-before-damage path is distinguished
from ordinary Attack scaling; other prevention interactions and the owned archive's
upstream build/channel remain unverified. Orb Walker growth is end-of-round in
those bytes, not owner-end-turn. Shield Smash's
pre-A18 Block uses calculated attack output rather than actual player HP loss
according to the cited secondary port, not newly established primary evidence.

Effects are readable data, **not a generic effect language**. Important amounts,
durations, card destinations and thresholds remain in those descriptions; they
are not all duplicated as machine-executable fields. A discrepancy note takes
precedence over treating an ambiguous source value as settled.

In encounters, `enemy_ids` is the **support set including possible dependent
actors**, not an initial ordered formation. Read `formation` for counts, slots,
sampling and offspring. Weak/strong entries for the same family are separate
reference records, but encounter-history checks compare the family, not record ID.
Explicitly unestablished mixed-group slot orders must not be inferred from names.

## Chances are conditional, not flat turn frequencies

Opening moves, deterministic continuations, selected-move history, live allies,
HP thresholds, debuffs, once-used flags and interruptions constrain selection.
Nominal percentages alone are **not an exact post-restriction sampler**. Do not
discard forbidden moves and renormalize the remainder unless the cited law says
to do that. Selection history is not always executed-move history.

The retained local Jaw Worm law gives post-Chomp Bellow **189/320 = 59.0625%**,
not a rounded 59%. Its secondary coins and the Acid Slime redirects are retained
explicitly. Red Slaver's Entangle-used flag is execution-set, even when Artifact
prevents the power; its A0 post-Entangle proposal is Stab45%/Scrape55%, not the
conflicting secondary port's 50/50. Higher-Ascension rules do not inherit a primary
evidence grade just because A0 is known.

Encounter selection is a separate process: ordinary last-two-family rejection,
elite no-immediate-repeat, special first-strong exclusions and boss discovery
must not become IID room rates. Gremlin Gang samples remaining **members**;
Shapes have their own composition rules. An event-owned fight does not consume
an ordinary queue just because its actors look familiar.

The approved [RNG-001](../../docs/intentional-deviations.md) policy preserves
mechanically relevant conditional probabilities, history, restrictions and
deliberate dependencies. It does **not** require original Java PRNG, seed mapping,
call consumption or incidental stream coupling. Reference data is not a policy
observation: future queues, hidden history and future draws must remain hidden.

## Evidence, conflicts and limits

The existing [25-kind Act1 report](../../docs/research/2026-10-19-act1-content-enemies.md)
supplies retained static and specifically reviewed source/bytecode findings in
its **Ironclad A0 Act1** scope. Its local rows and remaining composition boundaries
are retained, not promoted to all-character/all-Ascension/runtime acceptance.
The [Acts2/3 report](../../docs/research/2026-10-18-ironclad-a0-acts23-research.md)
is an initial supporting A0 draft establishing selected encounter rules, not
later-act enemy-mechanics closure. Existing 25 public keys/IDs are preserved.
Of 40 additional constructor labels, Darkling/Growth/Transient/Maw/Writhing/Head/
Nemesis/Reptomancer/Dagger/AwakenedOne/TimeEater/Deca/Donu now have named owned-static findings;27 remain provisional. Neither establishes upstream build provenance or native
serialization compatibility merely from species coverage.

Wiki revisions freeze the retrieved community claims, **not the v2.3 executable**.
Several pages are marked under construction. The secondary implementation is a
cross-check, not a correctness oracle; known malformed HP values from it were not
imported. Concrete disagreements remain visible:

- Mystic: Heal priority breakpoint and consecutive-Heal limiter.
- Snake Plant: A17 fixed-cycle claim versus a conflicting secondary repeat rule.
- Spire Growth: Constricted12 at wiki A2 versus secondary A17, plus A17 priority.
- Taskmaster: first Wound increase at wiki A2 versus secondary Elite A3.
- Writhing Mass: attributed wiki opening/later weights and fixedBlock16 differ from
  named owned-static opening/redirects and A0 base-backed Block15. The owned-byte
  law is documented; upstream build/channel and arbitrary callback compositions
  remain unverified, not silently promoted to target-PCv2.3 truth.
- Louses/Acid(S)/Byrd: missing or ambiguous wiki AI has secondary-qualified
  supplements rather than an invented primary claim.
- Nemesis: exact A0 owned-static cooldown/history/fair-coin redirects are now
  documented in local:m8-elites; nominal weights still are not effective rates.
  Reptomancer's A0 integer/history/live-count redirects are also now documented;
  Gremlin Leader's rounded reference chances retain their reference-only limits.

Other limits include mixed-formation slot order, exact leader/summon composition,
some card/power callback/terminal ordering and Stasis return edge cases. Empty
per-enemy uncertainty lists do not cancel these global limits. New source
inspection should be tied to a concrete implementation need, not a broad audit.

**Historical clock and Reaper lost-heal proposals are superseded.** No frame,
wait or original-timer subsystem is introduced. Preserve earned Reaper healing
when combat ends; the old EN-U3 text is not an active requirement. TIME-001 remains
the approved Portal eligibility assumption. No task or fidelity gate is closed
by compiling this reference.

## Verify or consume

```sh
python3.12 -m unittest discover -s tests/python -p 'test_enemy_reference_dataset.py' -v
```

The standard-library-only checks run offline. They check roster uniqueness,
Ascension-band coverage, per-hit representation, selected exact A0 numbers,
local-key preservation, citations, normalized nominal choices and explicit
conflict boundaries. The single independent review's five findings were addressed
with failing-then-passing reference regressions: Awakened One power triggers,
Exploder scaling qualification, Shield Smash Block basis, Mind Bloom dependent
reachability, and Painful Stabs' discard destination.
**These are not executed tests of 65 enemies' mechanics.**

```python
import json
from pathlib import Path

data = json.loads(Path("data/enemies/enemies.json").read_text())
by_id = {enemy["id"]: enemy for enemy in data["enemies"]}
print(by_id["CORRUPT_HEART"]["moves"])
```

Update records and citations together. Keep exact local laws, provisional
identities and disputed claims distinguishable; do not remove qualifiers merely
to make a registry appear complete. This dataset makes no simulator edits.

## Attribution and licensing

Wiki-derived/adapted contents of `enemies.json`, `encounters.json` and the wiki
metadata in `sources.json` are provided under **CC BY-SA 4.0**, not the repository's
blanket MIT code license. Attribution: **Slay the Spire Wiki contributors**, with
individual revision-pinned pages in `sources.json`; linked histories identify
contributors. Changes include normalized short descriptions, structured fields,
selected pattern summaries and explicit reconciliation/conflict notes.
License: <https://creativecommons.org/licenses/by-sa/4.0/>.

Existing local research and the small test retain their project terms. The
secondary implementation is cited, not copied into the dataset. Mega Crit
retains rights in Slay the Spire; no original game code or assets are distributed.
