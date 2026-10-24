"""Minimal masked-action tool demo, not a learner or tuned game strategy."""

import argparse

from spire_lab.gymnasium import RunEnv


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--character", default="ironclad")
    parser.add_argument("--ascension", type=int, default=0)
    parser.add_argument("--partition", default="test")
    parser.add_argument("--boss-history", default="time-eater-unseen")
    parser.add_argument("--seed", type=int, default=11)
    parser.add_argument("--budget", type=int, default=8192)
    parser.add_argument("--nonattacking", action="store_true")
    args = parser.parse_args()
    if not 0 <= args.budget <= 8192:
        parser.error("caller budget must be in 0..8192")
    game = RunEnv(
        args.character,
        args.ascension,
        partition=args.partition,
        boss_history=args.boss_history,
    )
    try:
        observation, _ = game.reset(seed=args.seed)
        game.action_space.seed(args.seed)  # Only caller sampling, not gameplay RNG.
        verbs = game.observation_spec()["vocabularies"]["verb"]
        for count in range(args.budget):
            if args.nonattacking:
                legal = observation["action_mask"].nonzero()[0]
                turns = [
                    int(i)
                    for i in legal
                    if observation["actions"][i, 0] == verbs.index("end_turn")
                ]
                action = turns[0] if turns else int(legal[0])
            else:
                action = game.action_space.sample(mask=observation["action_mask"])
            observation, _, terminated, truncated, info = game.step(action)
            if truncated:
                raise RuntimeError("base RunEnv must not impose truncation")
            if terminated:
                print(
                    f"{info['terminal_reason']}: {count + 1} actions; "
                    f"act{observation['context'][2]} floor{observation['context'][3]}; "
                    f"reward{observation['context'][7]} (not Heart victory)"
                )
                return 0
        parser.exit(1, "unfinished caller budget; not a gameplay loss or truncation\n")
    finally:
        game.close()


if __name__ == "__main__":
    raise SystemExit(main())
