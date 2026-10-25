"""Small masked-action M10 tool demo; not a learner or strategy."""

import argparse
import sys
from typing import Any

from spire_lab.gymnasium import RunEnv

M10_PROFILE = "m10-ironclad-heart"


def _action(
    game: RunEnv,
    observation: dict[str, Any],
    spec: dict[str, Any],
    nonattacking: bool,
) -> int:
    legal = observation["action_mask"].nonzero()[0]
    if legal.size == 0:
        raise RuntimeError("nonterminal Run has no legal public actions")
    if nonattacking:
        end_turn = spec["vocabularies"]["verb"].index("end_turn")
        action_tensor = next(
            tensor for tensor in spec["tensors"] if tensor["key"] == "actions"
        )
        verb_column = action_tensor["fields"].index("verb")
        turns = [
            int(index)
            for index in legal
            if observation["actions"][index, verb_column] == end_turn
        ]
        return turns[0] if turns else int(legal[0])
    return game.action_space.sample(mask=observation["action_mask"])


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profile", default=M10_PROFILE)
    parser.add_argument("--character", default="ironclad")
    parser.add_argument("--ascension", type=int, default=0)
    parser.add_argument(
        "--history", "--boss-history", dest="boss_history", default="time-eater-unseen"
    )
    parser.add_argument("--seed", type=int, default=11)
    parser.add_argument("--budget", type=int, default=8192)
    parser.add_argument("--nonattacking", action="store_true")
    args = parser.parse_args(argv)
    if not 0 <= args.budget <= 8192:
        parser.error("caller budget must be in 0..8192")

    game = None
    try:
        game = RunEnv(
            args.character,
            args.ascension,
            partition="test",
            boss_history=args.boss_history,
            record_replay=True,
            profile=args.profile,
        )
        observation, _ = game.reset(seed=args.seed)
        spec = game.observation_spec()
        game.action_space.seed(args.seed)  # Caller RNG only, not Run gameplay RNG.

        for accepted in range(args.budget):
            action = _action(game, observation, spec, args.nonattacking)
            observation, _reward, terminated, truncated, info = game.step(action)
            if truncated:
                raise RuntimeError("base RunEnv must not impose truncation")
            if terminated:
                # Report the actual terminal. Do not infer a Heart route or award.
                print(f"{info['terminal_reason']}: {accepted + 1} actions")
                return 0

        print(
            "unfinished caller budget; this is not a gameplay loss or Gym truncation",
            file=sys.stderr,
        )
        return 1
    except (ValueError, RuntimeError, NotImplementedError) as error:
        print(f"Run error: {error}", file=sys.stderr)
        return 1
    finally:
        if game is not None:
            game.close()


if __name__ == "__main__":
    raise SystemExit(main())
