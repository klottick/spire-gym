"""Native public-search playouts (M41): one-move evaluation without per-step Python round trips.

For every legal root action, K sampled public worlds (common random numbers per sample index) take the root action,
then ``PublicSearch.playout_many`` plays each to combat end or ``--playout-steps`` under the declared public rule
``spire-gym-public-playout-v1``: uniform over legal actions whose Standard verb is not excluded (default: end turn and
discard potion), else over all legal actions, from a research stream keyed by the sample, the playout seed and the step.
A terminal playout scores its reward, a refused one -1, any other half the public HP balance. The printed values
describe this baseline rule only; they are not strategy.

``--benchmark`` measures playouts per second against the lock-step loop a caller runs without M41: one ``step_many``
call per playout step over every live playout, the rule evaluated in Python on full observations (dense and compact),
at each ``--workers`` count, on standalone starts and a Standard Run combat. Measurements only; no outcome claims.
"""

from __future__ import annotations

import argparse
import json
import random
import time

import numpy as np

from spire_lab import _native
from spire_lab.gymnasium import RunEnv, StandardCombatEnv


SEED = 4_100_041
EXCLUDED = ("end_turn", "discard_potion")


def catalog(character, encounter, ascension=20, potions=None):
    starter = {
        "ironclad": ([("strike", 5), ("defend", 4), ("bash", 1)], "burning_blood"),
        "silent": ([("strike_g", 5), ("defend_g", 5), ("neutralize", 1), ("survivor", 1)], "ring_of_the_snake"),
    }[character]
    return {
        "catalog_schema_id": "spire-gym-standard-combat-catalog-v1",
        "profile_id": "standard-combat-v1",
        "entries": [{
            "id": f"{character}-{encounter}", "character": character, "ascension": ascension, "encounter": encounter,
            "hp": 70, "max_hp": 80, "gold": 99,
            "deck": [{"card": card, "upgrades": 0} for card, count in starter[0] for _ in range(count)],
            "relics": [starter[1]], "potions": [None, None] if potions is None else list(potions),
        }],
    }


def standalone(encounter, seed, potions=None):
    env = StandardCombatEnv(catalog("ironclad", encounter, potions=potions), partition="test")
    observation, _ = env.reset(seed=seed)
    return env, observation


def run_combat(seed):
    """A Standard Run (normal creation) taken by its first legal action into its first combat."""
    env = RunEnv("ironclad", 20, profile="standard", partition="test")
    observation, _ = env.reset(seed=seed)
    for _ in range(80):
        if observation["enemies_occupied"].any():
            return env, observation
        legal = np.flatnonzero(observation["action_mask"].reshape(-1))
        observation, _, terminated, _, _ = env.step(int(legal[0]))
        if terminated:
            break
    raise RuntimeError("the Standard route did not reach a combat decision")


class View:
    """Column lookups resolved once from the descriptor, and the same rule evaluated in Python (the baseline)."""

    def __init__(self, descriptor):
        tensors = {tensor["key"]: tensor for tensor in descriptor["tensors"]}

        def column(key, field):
            return [c["field"] for c in tensors[key]["columns"]].index(field)

        self.player_hp, self.player_max = column("player", "hp"), column("player", "max_hp")
        self.enemy_hp, self.enemy_max = column("enemies", "hp"), column("enemies", "max_hp")
        self.verb = column("actions", "verb")
        verbs = descriptor["vocabularies"]["verb"]
        self.excluded = {verbs.index(v) for v in EXCLUDED}

    def choice(self, observation, rng):
        legal = [int(i) for i in observation["action_mask"].nonzero()[0]]
        verbs = observation["actions"][legal, self.verb]
        preferred = [i for i, v in zip(legal, verbs) if int(v) not in self.excluded]
        return rng.choice(preferred or legal)

    def balance(self, observation):
        player, enemies = observation["player"], observation["enemies"]
        alive = enemies[:, self.enemy_max] > 0
        total = float(enemies[alive, self.enemy_max].sum()) or 1.0
        mine = float(max(player[self.player_hp], 0)) / float(max(player[self.player_max], 1))
        return mine - float(enemies[alive, self.enemy_hp].clip(min=0).sum()) / total


def summary_balance(result, i):
    """Half-HP-balance input from a playout summary (the same rows the observation publishes)."""
    present = result["enemy_present"][i].astype(bool)
    total = float(result["enemy_max_hp"][i][present].sum()) or 1.0
    mine = float(max(int(result["player_hp"][i]), 0)) / float(max(int(result["player_max_hp"][i]), 1))
    return mine - float(result["enemy_hp"][i][present].clip(min=0).sum()) / total


def open_search(env, legal, samples, playout_steps, workers, observation="dense"):
    per_branch = playout_steps + 2
    limits = dict(max_live_branches=len(legal) * samples, max_branches=len(legal) * samples,
                  max_steps_per_branch=per_branch, max_total_steps=len(legal) * samples * per_branch)
    return env.public_search(SEED, limits=limits, workers=workers, observation=observation)


def root_step(search, legal, samples):
    """Every (root action, sample) branch takes its root action in one mask-view call."""
    pairs = [(action, sample) for action in legal for sample in range(samples)]
    branches = [search.branch(sample) for _, sample in pairs]
    try:
        _, rewards, terminated, _, _ = search.step_many(branches, [a for a, _ in pairs], view="mask")
        refused = [False] * len(pairs)
    except _native.RunUnsupportedBoundaryError:
        rewards, terminated, refused = [], [], []
        for branch, (action, _) in zip(branches, pairs, strict=True):
            try:
                _, reward, done, _, _ = branch.step(action, view="mask")
                rewards.append(reward), terminated.append(done), refused.append(False)
            except _native.RunUnsupportedBoundaryError:
                rewards.append(-1), terminated.append(False), refused.append(True)
    return pairs, branches, rewards, terminated, refused


def native_values(env, legal, samples, playout_steps, workers, rule="uniform_exclude_verbs"):
    """M41/M42: root step, then one ``playout_many`` call for every live branch under ``rule``."""
    search = open_search(env, legal, samples, playout_steps, workers)
    try:
        pairs, branches, rewards, terminated, refused = root_step(search, legal, samples)
        values = [float(r) if (t or f) else None for r, t, f in zip(rewards, terminated, refused, strict=True)]
        live = [i for i, v in enumerate(values) if v is None]
        if live:
            result = search.playout_many([branches[i] for i in live], rule=rule, max_steps=playout_steps)
            for j, i in enumerate(live):
                if result["terminated"][j]:
                    values[i] = float(result["rewards"][j])
                elif result["end_reason"][j] == "refused":
                    values[i] = -1.0
                else:
                    values[i] = 0.5 * summary_balance(result, j)
        return pairs, values, search.counters()["total_steps"]
    finally:
        search.close()


def lockstep_values(env, legal, samples, playout_steps, workers, observation):
    """Baseline: one ``step_many`` per playout step over every live playout, the rule evaluated in Python."""
    search = open_search(env, legal, samples, playout_steps, workers, observation)
    view = View(search.observation_spec())
    try:
        pairs = [(action, sample) for action in legal for sample in range(samples)]
        branches = [search.branch(sample) for _, sample in pairs]
        actions = [a for a, _ in pairs]
        rngs = [random.Random(SEED * 1_000_003 + sample) for _, sample in pairs]
        values = [None] * len(pairs)
        active = list(range(len(pairs)))
        for depth in range(playout_steps + 1):
            if not active:
                break
            try:
                observations, rewards, terminated, truncated, _ = search.step_many(
                    [branches[i] for i in active], [actions[i] for i in active], view="full")
                refused = [False] * len(active)
            except _native.RunUnsupportedBoundaryError:
                observations, rewards, terminated, truncated, refused = None, [], [], [], []
                rows = []
                for i in active:
                    try:
                        row, reward, done, cut, _ = branches[i].step(actions[i], view="full")
                        rows.append(row), rewards.append(reward), terminated.append(done), truncated.append(cut), refused.append(False)
                    except _native.RunUnsupportedBoundaryError:
                        rows.append(None), rewards.append(-1), terminated.append(False), truncated.append(False), refused.append(True)
            continuing = []
            for j, i in enumerate(active):
                obs = rows[j] if observations is None else {k: v[j] for k, v in observations.items()}
                if refused[j] or terminated[j]:
                    values[i] = float(rewards[j])
                elif truncated[j] or depth == playout_steps:
                    values[i] = 0.5 * view.balance(obs)
                else:
                    actions[i] = view.choice(obs, rngs[i])
                    continuing.append(i)
            active = continuing
        return pairs, values, search.counters()["total_steps"]
    finally:
        search.close()


def report(label, legal, pairs, values, samples):
    means = {}
    for (action, _), value in zip(pairs, values, strict=True):
        means.setdefault(action, []).append(value)
    best = max(means, key=lambda a: sum(means[a]) / samples)
    print(f"{label}: {len(legal)} legal root actions, K={samples}; best root action {best} "
          f"mean {sum(means[best]) / samples:+.3f}")


def demonstrate(samples, playout_steps, workers, rule="uniform_exclude_verbs"):
    for label, (env, observation) in (("standalone ironclad A20 jaw_worm", standalone("jaw_worm", SEED)),
                                      ("Standard Run first combat A20", run_combat(11))):
        try:
            legal = [int(i) for i in np.flatnonzero(observation["action_mask"].reshape(-1))]
            pairs, values, steps = native_values(env, legal, samples, playout_steps, workers, rule=rule)
            report(f"{label} ({rule})", legal, pairs, values, samples)
            print(f"  playouts={len(pairs)} native_steps={steps}")
        finally:
            env.close()


def measure(method, env, observation, samples, playout_steps, workers, repeats):
    legal = [int(i) for i in np.flatnonzero(observation["action_mask"].reshape(-1))]
    wall, cpu, steps, playouts = 0.0, 0.0, 0, 0
    for _ in range(repeats):
        w, c = time.perf_counter(), time.process_time()
        if method == "playout_many":
            pairs, _, n = native_values(env, legal, samples, playout_steps, workers, rule="uniform_exclude_verbs")
        elif method == "playout_many:tiered_verbs":
            pairs, _, n = native_values(env, legal, samples, playout_steps, workers, rule="tiered_verbs")
        else:
            pairs, _, n = lockstep_values(env, legal, samples, playout_steps, workers, method.split(":")[1])
        wall += time.perf_counter() - w
        cpu += time.process_time() - c
        steps += n
        playouts += len(pairs)
    return {"method": method, "workers": workers, "decisions": repeats, "playouts": playouts, "steps": steps,
            "wall_s_per_decision": wall / repeats, "cpu_s_per_decision": cpu / repeats,
            "playouts_per_s": playouts / wall, "steps_per_s": steps / wall}


def benchmark(samples, playout_steps, workers_list, repeats, encounters, tiered_only=False):
    rows = []
    workloads = [(f"standalone:{e}", lambda e=e: standalone(e, SEED)) for e in encounters]
    workloads.append(("standalone:jaw_worm_potions", lambda: standalone("jaw_worm", SEED, potions=["fire_potion", "block_potion"])))
    workloads.append(("run:first_combat", lambda: run_combat(11)))
    methods = ("playout_many", "playout_many:tiered_verbs") if tiered_only else (
        "step_many:dense", "step_many:compact", "playout_many", "playout_many:tiered_verbs"
    )
    for name, make in workloads:
        env, observation = make()
        try:
            for workers in workers_list:
                for method in methods:
                    row = {"schema": "spire-gym-m42-playout-benchmark-v1", "workload": name, "samples": samples,
                           "playout_steps": playout_steps, **measure(method, env, observation, samples, playout_steps, workers, repeats)}
                    rows.append(row)
                    print(json.dumps(row, sort_keys=True), flush=True)
        finally:
            env.close()
    by = {(r["workload"], r["workers"], r["method"]): r for r in rows}
    for (workload, workers, method), r in by.items():
        if method == "playout_many:tiered_verbs":
            base = by[(workload, workers, "playout_many")]
            ratio = r["playouts_per_s"] / base["playouts_per_s"]
            step_ratio = r["steps_per_s"] / base["steps_per_s"]
            print(f"{workload} workers={workers}: tiered_verbs / uniform = {ratio:.2f}x playouts/s ({step_ratio:.2f}x steps/s)")
        elif method == "playout_many" and not tiered_only:
            for base in ("step_many:dense", "step_many:compact"):
                ratio = r["playouts_per_s"] / by[(workload, workers, base)]["playouts_per_s"]
                print(f"{workload} workers={workers}: playout_many / {base} = {ratio:.1f}x playouts per second")


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--samples", type=int, default=8)
    parser.add_argument("--playout-steps", type=int, default=80)
    parser.add_argument("--workers", type=int, nargs="+", default=[1])
    parser.add_argument("--rule", choices=("uniform_exclude_verbs", "tiered_verbs"), default="uniform_exclude_verbs")
    parser.add_argument("--benchmark", action="store_true")
    parser.add_argument("--benchmark-tiered", action="store_true", help="compare tiered_verbs vs uniform_exclude_verbs only")
    parser.add_argument("--repeats", type=int, default=1, help="benchmark decisions per workload")
    parser.add_argument("--encounters", nargs="+", default=["jaw_worm", "cultist", "gremlin_nob"])
    args = parser.parse_args()
    if args.benchmark or args.benchmark_tiered:
        benchmark(args.samples, args.playout_steps, args.workers, args.repeats, args.encounters, tiered_only=args.benchmark_tiered)
    else:
        demonstrate(args.samples, args.playout_steps, args.workers[0], rule=args.rule)


if __name__ == "__main__":
    main()
