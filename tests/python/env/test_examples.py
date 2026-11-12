"""Smoke the Standard and retained legacy example scripts as subprocesses."""

from __future__ import annotations

import os
from pathlib import Path
import re
import subprocess
import sys
import time


ROOT = Path(__file__).resolve().parents[3]


CASES = [
    ("standard_run.py", [], ("exact fork reproduced", "from_record(")),
    ("run_vector_rollout.py", ["--steps", "8"], ("episodes=", "slot_steps_per_s=", "completed_episodes")),
    ("standard_combat_vector.py", ["--steps", "8"], ("episodes=", "slot_steps_per_s=", "completed_episodes")),
    ("run_vector_phase_timing.py", ["--slots", "4", "--steps", "4"], ("spire-gym-m37-python-phase-timing-v1", '"binding"')),
    ("run_vector_phase_timing.py", ["--slots", "4", "--steps", "2", "--loops", "binding", "--observation", "compact"], ("spire-gym-m37-python-phase-timing-v1", '"observation": "compact"')),
    ("speed_check.py", ["--steps", "2"], ("spire-gym-speed-check-python-v1", '"workers_16"')),
    ("speed_check.py", ["--steps", "1", "--observation", "compact"], ("spire-gym-speed-check-python-v1", '"observation": "compact"')),
    ("public_lookahead.py", ["--samples", "1"], ("legal root actions", "counters=")),
    ("tactical_lessons.py", [], ("first_legal:", "seeded_random:")),
    ("combat_gymnasium.py", [], ("Collected 64 batches", "natural endings")),
    ("ironclad_combat_training.py", [], ("common-view", "normal Run")),
    ("run_m9_gymnasium.py", [], ("actions;",)),
    ("run_m10_gymnasium.py", [], ("actions",)),
]


def test_all_example_scripts_smoke():
    environment = os.environ.copy()
    for script, args, expected in CASES:
        command = [sys.executable, "-B", str(ROOT / "examples" / script), *args]
        started = time.perf_counter()
        result = subprocess.run(
            command,
            cwd=ROOT,
            env=environment,
            capture_output=True,
            text=True,
            timeout=120,
            check=False,
        )
        elapsed = time.perf_counter() - started
        print(f"SCRIPT_RESULT {script} exit={result.returncode} wall_s={elapsed:.3f}")
        assert result.returncode == 0, (
            f"{script} exited {result.returncode}\nstdout:\n{result.stdout}\nstderr:\n{result.stderr}"
        )
        output = result.stdout + result.stderr
        for marker in expected:
            assert marker in output, f"{script} did not print {marker!r}\n{output}"
        if script in {"run_vector_rollout.py", "standard_combat_vector.py"}:
            assert re.search(r"episodes=\d+ steps=8 slot_steps_per_s=[0-9]+(?:\.[0-9]+)?", result.stdout)
