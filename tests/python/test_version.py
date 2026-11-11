"""M36 (U3): the delivered release is 1.0.0 in the package, the project metadata and the Cargo workspace."""

import tomllib
from pathlib import Path

import spire_lab

ROOT = Path(__file__).resolve().parents[2]


def test_release_version_is_one_point_zero_everywhere():
    pyproject = tomllib.loads((ROOT / "pyproject.toml").read_text(encoding="utf-8"))
    cargo = tomllib.loads((ROOT / "Cargo.toml").read_text(encoding="utf-8"))
    assert spire_lab.__version__ == "1.0.0"
    assert pyproject["project"]["version"] == spire_lab.__version__
    assert cargo["workspace"]["package"]["version"] == spire_lab.__version__
