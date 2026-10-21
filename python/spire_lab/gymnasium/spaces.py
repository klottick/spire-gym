"""Build standard spaces from the native observation descriptor."""

import gymnasium as gym
import numpy as np


def spaces_from_spec(spec: dict) -> tuple[gym.spaces.Dict, gym.spaces.Discrete]:
    entries = []
    for tensor in spec["tensors"]:
        shape = tuple(tensor["shape"])
        dtype = np.dtype(tensor["dtype"])
        if tensor["space"] == "MultiBinary":
            space = gym.spaces.MultiBinary(shape)
            if space.dtype != dtype:
                raise ValueError("MultiBinary descriptor must have int8 dtype")
        elif tensor["space"] == "Box":
            low = np.broadcast_to(np.asarray(tensor["low"], dtype=dtype), shape).copy()
            high = np.broadcast_to(np.asarray(tensor["high"], dtype=dtype), shape).copy()
            space = gym.spaces.Box(low=low, high=high, dtype=dtype)
        else:
            raise ValueError(f"unsupported descriptor space: {tensor['space']}")
        entries.append((tensor["key"], space))
    action = spec["action_space"]
    if action["type"] != "Discrete":
        raise ValueError("expected native Discrete action descriptor")
    return gym.spaces.Dict(entries), gym.spaces.Discrete(action["n"], start=action["start"])
