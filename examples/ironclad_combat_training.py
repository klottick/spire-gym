"""Public standalone/batch/normal-Run smoke, no training or winning-policy claim."""

from __future__ import annotations

import numpy as np

from spire_lab.gymnasium import IroncladCombatEnv, IroncladCombatVectorEnv, RunEnv


def catalog():
    cards = [
        ("immolate", 1),
        ("warcry", 1),
        ("armaments", 1),
        ("strike", 0),
        ("defend", 0),
        ("whirlwind", 0),
    ]
    return {
        "entries": [
            {
                "id": name,
                "encounter": encounter,
                "hp": hp,
                "max_hp": 80,
                "deck": [{"card": c, "upgrades": u} for c, u in cards],
                "relics": ["burning_blood", "anchor"],
                "potions": ["elixir", "heart_of_iron", None],
            }
            for name, encounter, hp in [
                ("sentries", "three_sentries", 64),
                ("snecko", "snecko", 1),
            ]
        ]
    }


def first_legal(obs):
    return int(np.flatnonzero(obs["action_mask"])[0])


def main():
    single = IroncladCombatEnv(catalog(), partition="test")
    obs, info = single.reset(seed=11, options={"entry_id": "sentries"})
    for _ in range(128):
        obs, _reward, terminated, truncated, info = single.step(first_legal(obs))
        if terminated or truncated:
            break
    print(
        "standalone",
        info["outcome"],
        "HP",
        int(obs["player"][0]),
        "caller-budget-ended" if not terminated else "natural-end",
    )
    print("common-view", single.observation_spec()["fingerprint"])
    single.close()

    batch = IroncladCombatVectorEnv(2, catalog(), partition="test")
    obs, info = batch.reset(
        seed=[11, 12], options={"entry_ids": ["sentries", "snecko"]}
    )
    for _ in range(128):
        actions = np.array(
            [first_legal({"action_mask": mask}) for mask in obs["action_mask"]],
            dtype=np.int64,
        )
        obs, rewards, terminal, truncated, info = batch.step(actions)
        if terminal.any():
            print("batch old finals", terminal.tolist(), rewards.tolist())
            # Only explicit reset replaces old final rows; ongoing rows stay put.
            obs, info = batch.reset(options={"reset_mask": terminal.copy()})
            break
    batch.close()

    run = RunEnv("ironclad", 0, profile="m10-ironclad-policy", partition="test")
    obs, info = run.reset(seed=11)
    saw_combat = False
    for _ in range(128):
        view = run.combat_view()
        learned = view["info"]["routing"] != "noncombat"
        saw_combat |= learned
        action = first_legal(view["obs"] if learned else obs)
        obs, _native_reward, run_terminal, truncated, info = run.step(action)
        if run_terminal or truncated:
            break
    print(
        "normal Run",
        "combat-seen",
        saw_combat,
        "run-terminal",
        run_terminal,
        "last-exit",
        run.combat_view()["info"]["last_exit"],
    )
    run.close()


if __name__ == "__main__":
    main()
