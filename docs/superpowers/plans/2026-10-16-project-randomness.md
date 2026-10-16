# Project Randomness Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement the approved standalone Python randomness foundation under intentional deviation RNG-001, with exact project-contract tests rather than original-RNG parity.

**Architecture:** Immutable domain-derived key/cursor values feed HMAC-SHA256 counter words, unbiased samplers and Fisher–Yates shuffle. Each operation returns a value and new state; unrelated streams never share consumption. No combat-state integration or original game access is included.

**Tech Stack:** Python 3.12 standard library (`dataclasses`, `hmac`), existing pytest 9.1.1; no new dependencies.

**Spec:** `docs/superpowers/specs/2026-10-16-project-randomness-design.md` (user approved 2026-10-16).

**Execution status (2026-10-16):** All three tasks, fresh per-task reviews and final
integration review complete, no findings. Controller verification: `395 passed in
3.50s`; fixed fixture and four-file source checks pass. No combat integration,
staging, commits, pushes or cleanup. Reports and preserved history:
`.superpowers/sdd/2026-10-16-project-randomness/progress.md`.

## Global Constraints

- “Do not yet change `model.RngState`, `Scenario`, `CombatState`, content IDs or the existing snapshot schema.”
- “Do not add multiple backends now.”
- “Keep `model.py`, `snapshot.py` and all existing structural tests intact.”
- “All operation signatures take state first and return `(value, next_state)`.”
- “Sampling is pure and modifies neither input state nor caller-owned collections.”
- “Every integer must be an exact Python int, not bool.”
- “Exhaustion is an administrative error, never a loss, resample or wrapped stream.”
- “No additional original-RNG semantic review or raw-vector transport is a prerequisite for this project-owned contract.”
- Use `RNG_CONTRACT_ID = "spire-lab-rng-hmac-sha256-v1"`, 32-byte keys, BE32 identity framing and BE64 block indices exactly as specified. No native/tensor imports except standard-library hashing's own implementation; no Java, private fixtures or original floating overloads.
- User approved this plan and authorized execution on 2026-10-16 (“Yes begin”). Preserve the existing isolated worktree and unrelated work. No staging, commits, pushes, installation, cleanup or evidence changes are authorized; omit skill-default commit steps.
- Preserve the authorized Sol subagent implementation/review workflow; tasks are sequential because they extend one module. Review each completed task before the next and perform final integration review. Do not reopen the original RNG study.

## Review Focus

1. Ambiguous identity framing or shared cursors: `test_framing_is_unambiguous` and `test_interleaving_preserves_other_streams` must prevent unrelated stream coupling (Task1).
2. Invalid bools, subclasses and late invalid weights: validate before any word draw, including deterministic endpoints; `test_invalid_sampler_inputs_do_not_draw` (Task2).
3. Last word/block lane and rejection at exhaustion: never wrap, return a fabricated result or lose the latest cursor; `test_last_word_and_exhaustion` (Task1), `test_rejection_exhaustion_retains_cursor` (Task2).
4. Bound one, probability endpoints and zero weights: no unnecessary draws; `test_deterministic_cases_do_not_draw`, `test_weighted_zero_entries_and_boundaries` (Task2).
5. Shuffle identity, duplicates and mid-operation exhaustion: preserve caller inputs/elements and expose latest cursor; `test_shuffle_ownership_and_exhaustion` (Task3).

---

## File map and precomputed expectations

| File | Ownership |
|---|---|
| Create `python/spire_lab/combat/rng.py` | All RNG types, fixed constants, two small private encoding/HMAC helpers and five public operations |
| Create `tests/python/combat/test_rng.py` | Fixed fixtures, types/validation, stream derivation, words, replay/isolation and final composition |
| Create `tests/python/combat/test_rng_sampling.py` | Sampler/shuffle boundaries, forced draws and exhaustive small cases |
| Create `tests/python/fixtures/combat/project_rng_v1.json` | Exact project/public expected values; no loader, Java or private artifact dependency |

Planning source: `.superpowers/sdd/2026-10-16-project-randomness/planned-vectors.json`.
Copy its exact **8889 bytes** into the test fixture during Task1; SHA256
`369d91e2ce3be38c191ef5ceab6fe6e1bf5c73f21504e1748074e9c35d962bad`.
Do not regenerate expectations from `rng.py` or silently refresh them after failure.
Schema is `spire-lab-project-rng-fixtures-v1`; exact stream IDs are
`shuffle-train`, `hp-train`, `ai-train`, `ai-enemy2`, `ai-final`. There are two
RFC4231 primitive cases, five keys, fifteen blocks and forty-five word checks.
Word indices are decimal strings `0`–`7` and `18446744073709551615`; words are
sixteen-digit unsigned hex. Block indices are decimal strings. Every row and
boundary must be tested; none is optional or skipped.

Planning calculations cross-checked `hmac.digest` against explicit RFC2104
ipad/opad composition, both using SHA256; both matched the two published RFC4231
answers. Framing was also independently assembled as hex and parsed back into
fields. This is a bounded cross-check, not independence of the underlying SHA256
implementation or original-game evidence. Fixture provenance is part of the JSON.

Current Python3.12 [hmac](https://docs.python.org/3.12/library/hmac.html) and
[hashlib](https://docs.python.org/3.12/library/hashlib.html) docs were consulted
after Context7 rejected the request for quota. Use `hmac.digest(key, message,
"sha256")`; do not implement HMAC/SHA256 in product code.

## Task 1: Immutable stream identity, derivation and word sequence

**Files:** Create `rng.py`, `test_rng.py` and the JSON fixture above.

**Interfaces produced, owned by `rng.py`:**
- Frozen `StreamId(domain: str, scope: tuple[str, ...])`.
- Frozen `RandomStreamState(stream_id: StreamId, key: bytes, word_index: int)`.
- `RngExhaustedError(state: RandomStreamState)`; diagnostic attribute **`.state`** holds the latest state.
- `make_stream(root_seed: bytes, partition: str, episode_id: str, stream_id: StreamId) -> RandomStreamState`.
- `draw_u64(state: RandomStreamState) -> tuple[int, RandomStreamState]`.
- Private `_frame_fields(fields: tuple[str, ...]) -> bytes`, `_hmac_sha256(key: bytes, message: bytes) -> bytes`. The HMAC helper is an ordinary primitive wrapper, not restricted to 32-byte keys; RFC vectors use shorter keys. Validate project key lengths in the public state/factory boundaries.

- [x] **Step1: Create the fixed JSON fixture** from the exact planning source/identity above. Do not create product code or generate new expectations.
- [x] **Step2: Write failing tests** in `test_rng.py`, defining `M=2**64`. Use a test-local JSON reader, not a new framework or edits to existing `conftest.py`.

| Test name | Exact assertions |
|---|---|
| `test_fixture_identity_and_coverage` | Exact fixture bytes/hash above; schema/contract/root seed match; exact two primitive and five stream IDs; exactly fifteen blocks/forty-five word rows; each stream has indices `0..7,M-1` |
| `test_public_hmac_known_answers` | Both `primitive_cases`: `_hmac_sha256(bytes.fromhex(key_hex), bytes.fromhex(message_hex)).hex() == digest_hex` |
| `test_framing_is_unambiguous` | `_frame_fields(("a","bc")).hex() == "000000020000000161000000026263"`; `("ab","c")` gives `"000000020000000261620000000163"`; assert unequal |
| `test_project_stream_and_word_vectors` | For every row, exact frame/key match; initial cursor `0`; assert `_hmac_sha256(key, b"spire-lab/rng/v1/block\0" + int(block_index).to_bytes(8,"big")).hex() == block_hex` for every block; directly restore each of its nine word indices, assert `value == int(hex,16)` and returned cursor `index+1`; input unchanged |
| `test_replay_from_each_lane` | For each row and starting lane `0,1,2,3`, draw through index `7`; match every remaining recorded word; repeat from saved/deep-copied state identically |
| `test_interleaving_preserves_other_streams` | Keep `shuffle-train` and `ai-train` saved; advance HP and enemy2 streams in different interleavings; target first eight outputs remain their fixed fixture words and cursors `8`; no shared mutation |
| `test_identity_dimensions_and_creation_order` | Derive same IDs in opposite creation orders: equal states/outputs. From `ai-train`, vary episode to `episode-2`, combat ID to `combat-2`, enemy to `enemy-2`, partition (all four), or domain to `combat.enemy.hp`: exact frames change. Reversing root bytes instead changes the HMAC key argument while the frame stays equal; capture primitive arguments to assert that distinction. Repeats have equal identities/cursors; output inequality is not an independence proof |
| `test_last_word_and_exhaustion` | At `2**64-1`, use fixture's last word, return cursor `2**64`; next draw raises `RngExhaustedError` with `.state == returned_state`; input states stay unchanged |
| `test_values_are_frozen_and_owned` | Assigning declared fields raises `FrozenInstanceError`; scope/key immutable; identical derivations compare equal; fresh results do not mutate saved values |
| `test_import_and_hash_seed_independence` | Clean subprocess imports only RNG path, no `numpy`, `torch`, `spire_lab._native`; compare first eight fixed words under `PYTHONHASHSEED=1` and `2`. Set child `PYTHONPATH` to repository `python/`; no installs |

Add parameterized `test_identity_and_state_validation` with these cases, each
asserting the named exception. Patch `_hmac_sha256` to fail if called while testing
invalid `make_stream` inputs, so failure is demonstrably before derivation.

| Boundary | TypeError cases | ValueError cases / valid edges |
|---|---|---|
| Root/key | bytearray, int, bytes subclass | lengths `31,33` invalid; `32` valid |
| Identity fields | non-str/str subclass, scope list/tuple subclass, non-StreamId/subclass | empty, space, NUL, DEL, `é`, length256 invalid; `!`, `~`, length255 valid for episode/scope fields |
| Domain/scope | wrong owned types | unknown domain; shuffle arity0/2, HP/AI arity1/3 invalid; shuffle arity1 and HP/AI arity2 valid |
| Partition | non-str/subclass | `Train`/unknown invalid; `train,selection,final,test` valid |
| Cursor | `True,1.0,"0"`, int subclass | `-1,2**64+1` invalid; `0,2**64-1,2**64` valid |
| Operation state | `draw_u64(None)` | must raise TypeError, not AttributeError |

- [x] **Step3: Verify RED** `.venv/bin/python -B -m pytest tests/python/combat/test_rng.py -q`; expected missing `spire_lab.combat.rng` before implementation, not a fixture/path error.
- [x] **Step4: Implement only Task1 interfaces** in `rng.py`: strict built-in type/value checks, frozen dataclasses, immutable registry for the three domain arities, exact spec prefixes/framing/BE64/four lanes and checked exhausted sentinel. No future-operation stubs. `RngExhaustedError.state` must not hold a fabricated fresh/reset stream.
- [x] **Step5: Verify GREEN** with the same command; all listed fixture/validation cases pass with zero skips. Then run `.venv/bin/python -B -m pytest tests/python/combat -q` to include existing structural tests.
- [x] **Step6: Review Task1** with a fresh reviewer; record exact commands/results and resolve material findings before Task2. No staging/commit command.

## Task 2: Unbiased bounded, rational and weighted sampling

**Files:** Modify `rng.py`; create `test_rng_sampling.py`.

**Interfaces consumed:** Task1 `StreamId`, `RandomStreamState`, `make_stream`, `draw_u64`, `RngExhaustedError.state`, with the exact signatures above.

**Interfaces produced:**
- `uniform_below(state: RandomStreamState, bound: int) -> tuple[int, RandomStreamState]`.
- `bernoulli(state: RandomStreamState, numerator: int, denominator: int) -> tuple[bool, RandomStreamState]`.
- `weighted_index(state: RandomStreamState, weights: tuple[int, ...]) -> tuple[int, RandomStreamState]`.

- [x] **Step1: Write failing sampler tests** using self-authored word sequences. Test-local `force_words(monkeypatch, words)` patches `rng.draw_u64`, advances via `dataclasses.replace`, and raises `RngExhaustedError(state)` at the spec cursor limit before accessing the next scripted word. Running out of scripted words before that limit is a test-helper error, not simulated RNG exhaustion. It records requested states; it is not a product injectable RNG API. Use `M=2**64` and `s=make_stream(bytes(range(32)), "train", "episode-1", StreamId("combat.shuffle", ("combat-1",)))` as the saved baseline.

| Test name | Exact assertions |
|---|---|
| `test_rejection_consumes_only_local_words` | Words `[M-1,5]`, bound10: result `5`, cursor `start+2`, rejected word not reused; independent other stream unchanged |
| `test_full_word_bound` | Word `M-1`, bound `M`: result `M-1`, cursor `start+1` |
| `test_rejection_exhaustion_retains_cursor` | Start `M-1`, word `M-1`, bound10 rejects; next attempt raises with `.state.word_index == M`; original start unchanged |
| `test_deterministic_cases_do_not_draw` | Patch draws to fail; `(uniform_below(s,1),bernoulli(s,0,7),bernoulli(s,7,7),weighted_index(s,(0,1,0))) == ((0,s),(False,s),(True,s),(1,s))`, including exhausted `s` |
| `test_bernoulli_exact_small_outcomes` | For each forced word `0,1,2,3,4` from fresh state, `(2,5)` gives `[True,True,False,False,False]`; exact bool type; each cursor `start+1` |
| `test_weighted_zero_entries_and_boundaries` | Weights `(0,2,0,3)`, forced words `0..4`: indices `[1,1,3,3,3]`; weights `(0,M,0)`, word `M-1`: index1/one word |
| `test_invalid_sampler_inputs_do_not_draw` | Patch draws to fail; parameterized cases below raise TypeError/ValueError before drawing, even on exhausted state; saved state unchanged |

Invalid input matrix: non-state argument to all three operations => TypeError,
including deterministic endpoints. Integer inputs `True,1.0,"1"` and int subclasses => TypeError.
Bounds `0,-1,M+1` => ValueError. Bernoulli `(n,d)` values `(-1,5),(6,5),(0,0),
(1,-1),(1,M+1)` => ValueError; `(0,M),(M,M)` are valid non-consuming edges.
Weights list/generator/tuple subclass, `(1,True)`, `(1,object())` => TypeError;
`()`, `(0,0)`, `(1,-1)`, `(M,1)` => ValueError. The late-invalid-member cases
must fail even if an earlier weight could otherwise have won.

- [x] **Step2: Verify RED** `.venv/bin/python -B -m pytest tests/python/combat/test_rng_sampling.py -q`; expected missing sampler interfaces, not a broken fake-word helper.
- [x] **Step3: Implement the three typed functions** in `rng.py`. Validate state/arguments before deterministic early returns. Use exact `L=M-M%bound`, repeated draw-until-accepted and modulo; implement rational comparison and first cumulative sum strictly greater than the sample. Return actual advanced state, including rejection consumption; propagate latest exhaustion unchanged. No float arithmetic, hidden retry cap, histogram tuning or global state.
- [x] **Step4: Verify GREEN** with the same command, then `.venv/bin/python -B -m pytest tests/python/combat -q`; zero skips and all prior tests pass.
- [x] **Step5: Review Task2** with a fresh reviewer, including cumulative boundaries and max-bound arithmetic; retain verification notes, no staging/commit.

## Task 3: Owned shuffle and composed replay acceptance

**Files:** Modify `rng.py`, `test_rng_sampling.py`, `test_rng.py`; keep fixed JSON unchanged.

**Interfaces consumed:** Task1 state/derivation/draw, Task2 `uniform_below` and samplers.

**Interface produced:** `shuffle[T](state: RandomStreamState, items: list[T] | tuple[T, ...]) -> tuple[list[T], RandomStreamState]`.

- [x] **Step1: Add failing shuffle/composition tests** with these exact assertions. Existing test modules import `shuffle` only when this task begins; do not add an earlier stub to make collection pass. For the six exhaustive paths, patch `rng.uniform_below` with a test-local recorder returning scripted `j` and cursor+1, so requested bounds can be asserted. Exhaustion tests instead keep the real bounded sampler and use Task2's forced-word helper.

| Test name | Exact assertions |
|---|---|
| `test_shuffle_all_three_element_paths` | Force successive bounded choices `(j2,j1)` from `{0,1,2}×{0,1}`; requested bounds exactly `[3,2]`; six outputs equal the six permutations of `("a","b","c")`, each once; cursor `start+2` |
| `test_shuffle_ownership_and_exhaustion` | With `a,b=object(),object()`, three references `[a,b,a]`: output is a new list with exactly two `is a` and one `is b`, original unchanged. Input `["a","b","c","d"]` at cursor `M-2` with forced words `[0,M-1]`: partial shuffle then bound3 rejection/exhaustion; error cursor `M`, caller list unchanged, no returned permutation |
| `test_shuffle_empty_singleton_and_invalid_inputs` | `[]`, `()`, `[a]`, `(a,)` return new lists with original elements, no draws even when exhausted. None/generator/list-or-tuple subclass => TypeError without iteration/draw; non-state also TypeError |
| `test_project_composed_replay` | From fixed `shuffle-train` state, call `uniform_below(10)`, `bernoulli(2,5)`, `weighted_index((0,2,0,3))`, `shuffle(("a","b","c"))` in order: values exactly `[9,False,1,["c","b","a"]]`, cursor5. Repeat from saved/deep-copied state identically; assert fixture integration row agrees |

- [x] **Step2: Verify RED** `.venv/bin/python -B -m pytest tests/python/combat/test_rng.py tests/python/combat/test_rng_sampling.py -q`; expected missing `shuffle` before implementing it.
- [x] **Step3: Implement `shuffle`** in `rng.py`: validate exact state/container types before copying, copy once, descend `n-1..1` with `uniform_below(i+1)`, return detached list/state. Preserve element objects. Do not catch exhaustion into a result, change input lists, or deep-copy cards. Do not wire a combat mechanic to this function yet.
- [x] **Step4: Verify focused and complete suites**:

```bash
.venv/bin/python -B -m pytest tests/python/combat/test_rng.py tests/python/combat/test_rng_sampling.py -q
.venv/bin/python -B -m pytest tests/python/combat -q
.venv/bin/python -B -m pytest tests/python -q
git diff --check
```

Run each command separately. Expected: exit0, every new test/fixture row passes,
no new skips, structural tests unchanged and no whitespace errors. Verify source
fixture SHA again; no new/private oracle, GPU test dependency or installation.
Existing unrelated failure => report it without rewriting old contracts or
claiming the RNG milestone fully verified. Do not infer test count from fixture
word count or report these tests as game correctness/performance results.

- [x] **Step5: Final review and handoff**: fresh integration reviewer checks the four-file change against the approved spec and RNG-001, including exact fixtures, pure-state/type/consumption behavior, dependency boundary and no combat integration. Report actual test counts/commands and any deferred findings; mark this RNG slice implemented only after verification. Do not mark Python combat, Windows validation, Rust parity or training complete. No commit/push without separate consent.

## Plan self-review and execution handoff

- Spec coverage: derivation/state/version/lanes/exhaustion and dependency boundary → Task1; bounded/rational/weighted laws → Task2; uniform shuffle and composed replay → Task3. Scheduling isolation tested at stream level; pool allocation, policy views and combat integration remain explicitly out of scope.
- Type consistency: all operations use `RandomStreamState`; root derivation is separate; `.state` is the exhaustion diagnostic; `shuffle` alone is generic in element type. No unowned `RngState`, Java result types or fixture adapter introduced.
- Every Review Focus item has an owning red/green test. Forced words and bounded choices are test-only monkeypatches, not production backends.
- All expectations are fixed public/project material before product code. The planning record is 8889 bytes, not an original-bytecode oracle or a claimed security proof.
- This three-task plan was approved for execution on 2026-10-16. It preserves the existing Sol subagent workflow. Execution approval is not staging/commit consent.
