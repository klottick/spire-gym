"""Behavior tests for the native import and device readiness probe.

These exercise real code: the compiled ``spire_lab._native`` extension and
the real optimizer probe. No test is expected to be skipped.
"""

import pytest
import torch

from spire_lab import _native
from spire_lab.readiness import check, probe


def test_native_import():
    assert _native.build_info()["extension"] == "spire-python"


def test_cpu_optimizer_probe():
    report = probe("cpu")
    assert report["finite_gradient"] and report["weights_changed"]
    assert report["synchronized"]


def test_check_requires_cuda_raises_when_unavailable(monkeypatch):
    monkeypatch.setattr(torch.cuda, "is_available", lambda: False)
    with pytest.raises(RuntimeError, match="no CUDA device is available"):
        check(require_cuda=True)


def test_check_without_cuda():
    report = check(require_cuda=False)
    assert report["status"] == "ok"
    assert report["native"]["extension"] == "spire-python"
    assert report["cpu"]["finite_gradient"] and report["cpu"]["weights_changed"]
