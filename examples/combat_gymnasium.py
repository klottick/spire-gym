"""Finite headless masked caller rollout; not a learner or environment time cap."""

import numpy as np

from spire_lab.gymnasium import CombatVectorEnv


def main():
    env = CombatVectorEnv(3, workers=2)
    policy_rng = np.random.default_rng(7)  # Caller RNG, never gameplay RNG.
    completed = 0
    try:
        obs, _ = env.reset(seed=255)
        # This caller stops collecting after 64 batches. Unfinished episodes are
        # not truncated, scored as losses, or reset at the collection boundary.
        for _ in range(64):
            actions = np.array([
                policy_rng.choice(np.flatnonzero(mask)) for mask in obs["action_mask"]
            ], dtype=np.int64)
            obs, rewards, terminated, truncated, info = env.step(actions)
            assert not truncated.any()
            for slot in np.flatnonzero(terminated):
                final_obs = info["final_obs"][slot]
                final_info = info["final_info"]
                assert info["_final_obs"][slot] and info["_final_info"][slot]
                assert not final_obs["action_mask"].any()
                print(f"slot={slot} episode={final_info['episode_id'][slot]} "
                      f"reward={rewards[slot]:g} "
                      f"hp={final_info['hp_before_healing'][slot]}"
                      f"->{final_info['hp_after_cleanup'][slot]}")
                completed += 1
        print(f"Collected 64 batches; {completed} natural endings. No environment cap.")
    finally:
        env.close()


if __name__ == "__main__":
    main()
