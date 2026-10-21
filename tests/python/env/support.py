import numpy as np


def template(**changes):
    value = dict(id="owned", encounter="cultist", player_hp=80,
                 player_max_hp=80, energy_per_turn=3, draw_per_turn=5,
                 enemy_hp=52, hand=[dict(kind="strike", upgraded=False)],
                 draw=[dict(kind="defend", upgraded=False)])
    value.update(changes)
    return value


def assert_numeric(raw, rows, spec):
    assert list(raw["obs"]) == [t["key"] for t in spec["tensors"]]
    for tensor in spec["tensors"]:
        array = raw["obs"][tensor["key"]]
        assert array.shape == (rows, *tensor["shape"])
        assert array.dtype == np.dtype(tensor["dtype"])
        assert array.flags.c_contiguous and array.dtype.isnative
    assert raw["tokens"].shape == (rows, 2)
    assert raw["tokens"].dtype == np.dtype("uint64")
    assert raw["tokens"].flags.c_contiguous
    assert set(raw["info"]) == {"episode_id", "scenario_id", "episode_steps", "outcome",
                               "hp_before_healing", "hp_after_cleanup",
                               "hp_before_healing_mask", "hp_after_cleanup_mask"}
    for array in raw["info"].values():
        assert array.shape == (rows,) and array.flags.c_contiguous
    assert raw["info"]["episode_steps"].dtype == np.uint64
    for key in ["outcome", "hp_before_healing", "hp_after_cleanup"]:
        assert raw["info"][key].dtype == np.int32
    for key in ["hp_before_healing_mask", "hp_after_cleanup_mask"]:
        assert raw["info"][key].dtype == np.bool_
    for key in ["episode_id", "scenario_id"]:
        assert raw["info"][key].dtype == np.dtype("U64")
