"""Small shared checks for the public Gymnasium vector surfaces."""

import numpy as np


def _equal_observations(left, right):
    assert left.keys() == right.keys()
    for key in left:
        np.testing.assert_array_equal(left[key], right[key])


def assert_vector_conformance(env, *, seed=73):
    observations, info = env.reset(seed=seed)
    assert observations.keys() == env.single_observation_space.spaces.keys()
    for row in range(env.num_envs):
        single = {key: values[row] for key, values in observations.items()}
        assert env.single_observation_space.contains(single)
        for value in single.values():
            assert value.dtype == np.dtype(value.dtype).newbyteorder("=")
    assert isinstance(observations["action_mask"], np.ndarray)
    assert observations["action_mask"].dtype == np.dtype(np.int8)
    _assert_info_masks(info, env.num_envs)

    again, _ = env.reset(seed=seed)
    _equal_observations(observations, again)
    actions = np.asarray([
        int(np.flatnonzero(row.reshape(-1))[0])
        for row in again["action_mask"]
    ], dtype=np.int64)
    stepped, rewards, terminated, truncated, step_info = env.step(actions)
    assert rewards.shape == terminated.shape == truncated.shape == (env.num_envs,)
    assert rewards.dtype == np.dtype(np.float32)
    assert terminated.dtype == truncated.dtype == np.dtype(np.bool_)
    assert not truncated.any()
    for row in range(env.num_envs):
        single = {key: values[row] for key, values in stepped.items()}
        assert env.single_observation_space.contains(single)
    _assert_info_masks(step_info, env.num_envs)
    if "final_obs" in step_info:
        for row, final in enumerate(step_info["final_obs"]):
            if final is not None:
                assert env.single_observation_space.contains(final)
    for key in stepped:
        assert not np.shares_memory(observations[key], again[key])
        assert not np.shares_memory(again[key], stepped[key])
    return observations, info


def _assert_info_masks(info, size):
    if "slots" in info:  # The compatibility Gym66 facade keeps its row dictionaries.
        return
    for key, values in info.items():
        if key.startswith("_"):
            continue
        mask_key = "_" + key
        if mask_key in info:
            assert info[mask_key].shape == (size,)
            assert info[mask_key].dtype == np.dtype(np.bool_)
        if isinstance(values, np.ndarray) and values.ndim:
            assert values.shape[0] == size
