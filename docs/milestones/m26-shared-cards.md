# M26 shared and off-colour card coverage

Status: source merged as PR66 (`918f628`). Historical candidate status at freeze v2 (2026-11-01): the independent final review candidate incorporated corrections 7–11 below; this remains a finite M26 source record, not a separate public profile. The source was built early on the frozen M25 tree and rebased onto merged M25
(`main` at `3729b29e`, PR65). The design §16 interface check and the §6.3 site sweep were
rerun on the merged source with no unaccommodated difference. The freeze v2 tree has 1035
files; documentation edits after it are listed in the candidate.

This profile implements the approved [design](../superpowers/specs/2026-10-31-m26-shared-cards-design.md)
(version 3) and [plan](../superpowers/plans/2026-10-31-m26-shared-cards.md). Owners,
rulings, corrections and amended assertions are in the
[reconciliation](m26-shared-cards-reconciliation.md); every kind and pool is mapped to its
tests in the [coverage ledger](m26-shared-cards-coverage-ledger.md). The predecessor is M25 (Watcher A0
Acts 1–4 and Heart, [PR65](https://github.com/klottick/spire-gym/pull/65)).

## What M26 is

M26 is a card layer, not a Run profile. It adds no public profile, descriptor, fingerprint,
wire value or Python surface. Its results are reachable only through core tests and the
default-off `m26-diagnostics` feature. M27 (relics and potions) and M28 (events, Neow, shops
and standard pools) are the first callers.

After M26, in the native Rust core:

1. **Every standard card exists and plays.** 369 of the 370 dataset rows have a native,
   executable identity in base and upgraded form. M26 adds 5 colourless cards (Bandage Up,
   Apparition, Bite, J.A.X., Ritual Dagger) and 10 curses (Clumsy, Curse of the Bell, Decay,
   Doubt, Injury, Necronomicurse, Normality, Pain, Shame, Writhe), and gives Regret and
   Ascender's Bane their behaviour. Pride (Endless Mode only) is excluded.
2. **Any character can hold and play any legal card through that card's own implementation**,
   under the combat overlay `CombatItemContext::with_complete_cards()`. Where the source makes
   behaviour depend on the player's character (generators that draw from "your" pool, orb
   cards without orb slots, Alchemize's class potion pool), the shared owner follows that law.
3. **Every card producer pool is complete and has one owner** (`content::pools`), computed
   from the catalog, with literal size checks before any RNG is drawn.
4. **Permanent deck changes are complete and shared** (`run/mutations.rs`, under
   `CardAdmission::Complete`): obtain, upgrade (Searing Blow repeatable), remove (curse rules),
   transform, duplicate, Note For Yourself retrieval and pool draws. Ritual Dagger's growth
   persists to the Run deck beside Genetic Algorithm's.
5. **Copy, generated and status paths** are checked for every card kind and every character.

Frozen profiles stay frozen: combat-v1, the default and Gym66 profiles, policy-v3, and every
accepted M9–M25 profile keep their lists, descriptors, fingerprints, wire values and RNG
keys. The exceptions are mechanical exhaustive-match arms and the disclosed global
corrections in the reconciliation, some of which change behaviour that frozen profiles can
reach (listed there under "Corrections that change frozen behaviour").

## Shared-owner corrections

Each is fixed at the shared owner for every character and disclosed in the reconciliation (rule,
source, scope, tests and amended assertions):

| # | Rule | Owner | Test |
| --- | --- | --- | --- |
| G1–G7 | Planned corrections: Watcher reward fence, `selectable_pool`, L34, generator arms and Infernal Blade, off-colour gates, Alchemize, next-turn energy | design §11 | keys U, G, W |
| 1 | Red Skills and Powers have generated constructors | `card_state::generated` | `m26_generation_player_pool` |
| 2, 5 | Generated Watcher kinds get their initializer; Deus Ex Machina is unplayable without its draw (M25-owned, M25 G1/G2) | `card_state::generated`, `cards/mod.rs` | `m26_generated_watcher_kinds_match_their_m25_constructor`, `m26_cross_character_equivalence` |
| 3 | Clash's base cost does not depend on play legality | `ironclad::starting_cost` | `m26_infernal_blade_clash_beside_a_skill` |
| 6 | Curse of the Bell, Necronomicurse and Ascender's Bane cannot be removed | `run/mutations.rs` | `m26_mutation_obtain_upgrade_remove` |
| 7 | A Weak/Frail/Vulnerable from a monster or from an end-of-turn self-playing card is just-applied (Doubt, Shame) | `combat/powers.rs::just_applied` | `m26_end_turn_curse_debuffs_last_through_the_next_turn` |
| 8 | Burn, Decay, Doubt, Shame and Regret discard themselves when retained (M25 R4 generalized) | `content::plays_itself_at_end_of_turn`, `piles.rs` | `m26_end_turn_self_playing_cards_discard_when_retained` |
| 9 | Downing the last Darkling is Fatal (Feed, Hand of Greed, Lesson Learned, Ritual Dagger) | `darklings::final_down`, `ironclad/fatal.rs` | `m26_fatal_fires_on_the_final_darkling` |
| 10 | Master Reality upgrades any created card that can still upgrade (Searing Blow) | `piles::insert_generated` | `m26_master_reality_upgrades_a_copied_searing_blow` |
| 11 | Dead Dual Wield Master Reality guard removed | `ironclad/choices.rs` | (no behaviour change) |

Corrections 3 and 9 change behaviour that frozen profiles can reach; the reconciliation lists
them and the retained-outcome sweep.

## Unions

| Union | Count | Members |
| --- | --- | --- |
| Complete permanent (deck-legal) | 352 | 300 character cards, 35 ordinary colourless, 4 event colourless, 13 curses |
| Complete generated-only | 14 | 9 generated children, 5 statuses |
| Choice-only | 3 | Wish's options |
| Admitted in total | 369 | 370 rows minus Pride |

`content::complete` computes these from catalog availability and note storage; the counts are
literals in `crates/spire-core/tests/m26_catalog.rs`.

## Pools

Combat generator pools (HEALING excluded), per player character:

| Pool | Ironclad | Silent | Defect | Watcher |
| --- | --- | --- | --- | --- |
| Any | 70 | 70 | 70 | 69 |
| Attack | 28 | 26 | 23 | 24 |
| Skill | 28 | 33 | 34 | 33 |
| Power | 14 | 11 | 13 | 12 |
| Common (Hello World) | 20 | 19 | 18 | 19 |
| Colourless (L34) | 34 | 34 | 34 | 34 |
| Foreign Influence | 106 | 106 | 106 | 106 |

Run pools (no HEALING exclusion): Ordinary per colour (Red 72, Green 71, Blue 71, Purple 71),
any-colour 320 (76/160/84), colourless 35 (0/20/15), random curse 10, transform (input
removed from its family), Note For Yourself 349. Accepted orders are kept and checked as
permutations of the derived membership; new families are ordered Common → Uncommon → Rare,
then `CardId::as_str`. M26 appends three RNG scopes after M25's: `run.m26.card.offer`,
`run.m26.card.transform`, `run.m26.card.curse`.

## Evidence (grades kept separate)

M26 uses two grades. **Constructed**: an owner checkpoint built directly. **Supplied played**:
a supplied start, then only legal actions through the real owners. There are no Normal played
cases and no Native agreement runs, because M26 publishes no profile. No outcome or win is a
gate (2026-10-30 scope clarification).

| Key | Test file | Grade | Covers |
| --- | --- | --- | --- |
| U | `tests/m26_catalog.rs` | Constructed | Identities, unions 352/14/3, tags, pool sizes and order, G1–G3 |
| K | `combat/tests/m26_cards.rs`, `m26_corrections.rs` | Constructed | The 5 colourless cards and 12 curse behaviours, base and upgraded; corrections 7–10 and F10 for every character |
| W | `combat/tests/m26_offcolour_laws.rs`, `m26_capability.rs` | Constructed | Off-colour stances, Mantra, Scry, orbs at 0 slots and after Capacitor, G7 energy (Fasting), Alchemize, overlay inertness |
| X | `combat/tests/m26_cross_character.rs` | Constructed | Cross-character matrix and own-colour inertness (600 own rows, base and upgraded) |
| G | `combat/tests/m26_generation.rs`, `m26_pools.rs` | Constructed | Generator pools per character (8,800 samples), Infernal Blade per character |
| Y | `combat/tests/m26_copies.rs`, `m26_support.rs` | Constructed | Copy owner for every kind; generated children and statuses for every character |
| R | `run/tests/m26_card_mutations.rs`, `m26_pools.rs`, `m26_capability.rs` | Constructed | Run mutations, rejections atomic, Run pools and draw law, sampler RNG identity, Ritual Dagger journal and census |
| — | `combat/tests/m26_autoplay.rs` | Constructed | Normality under autoplay, Ritual Dagger exclusions |
| S | `run/tests/m26_supplied.rs` | Supplied played | One mixed off-colour/colourless/event/curse deck per character, played to a combat end and settled once; the Run deck equals the supplied deck plus the combat journal |

The four `m26_supplied` tests pass on the frozen tree: for each character one combat is
entered and settled once, with no fault, and the Run deck equals the supplied deck plus the
combat journal. Their printed run on the pre-rebase tree (H4-pre) reached Victory for all
four characters (Ironclad 13 actions / 3 turns, Silent 17 / 4, Defect 19 / 4, Watcher 27 / 7);
the tests assert Victory or Death, not the action counts. These are supplied fixtures, not
normal traversal.

**Freeze v2 suites** on the exact frozen tree (fresh copy, fresh `CARGO_TARGET_DIR`,
2026-11-01; tree manifest `c050eec2…`, 1035 files):

| Command | Passed | Failed | Ignored |
| --- | ---: | ---: | ---: |
| `cargo test --workspace --locked --offline --no-fail-fast` | 3461 | 0 | 11 |
| `cargo test -p spire-core --features m26-diagnostics --locked --offline --no-fail-fast` | 2835 | 0 | 5 |
| `cargo test -p spire-core --lib --locked --offline m26_` | 55 | 0 | 0 |
| `cargo test -p spire-core --locked --offline --test m26_catalog` | 5 | 0 | 0 |

The workspace run includes the M10, M15, M20, M21–M25 and training-profile suites. No
failure is inherited. There are no once-only outcome runs (design §14.4). The tree manifest
and logs are retained in the execution evidence under `main-v1/freeze-v2/` (freeze v1,
`main-v1/freeze/`, is superseded and kept as history). A retained-outcome sweep before freeze v2
replayed all 40 in-scope accepted M14/M15 receipts identically (reconciliation).

## Limits

M26 does not claim:

- any public profile, descriptor or Python surface for the complete card set;
- card reward screens, shop rows, Neow, events or their rarity schedules (M28);
- any relic or potion hook (M27). Under the overlay a potion that Alchemize gains is a valid
  belt item through settlement, but its use by a character whose base context does not
  already admit it is M27's potion lifecycle;
- Ascender's Bane creation (M30) or Note For Yourself persistence between runs (M34);
- Pride, A1–A20, original-runtime or Java RNG parity, or any strategy or win;
- observable `CardAcquired`/`CardRemoved` hook effects: the Run owners fire them, but no relic
  observer exists yet, so the tests check state only.

Disclosed project decisions (frontiers F1, F4, F5, F8–F12, F15, F16) are implementation
choices the cited sources most directly support, not validated source facts; they are listed
in the reconciliation. Constructed and supplied results are never presented as normal
traversal. Python delivery is not claimed.
