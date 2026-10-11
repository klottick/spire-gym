"""Behavior tests for the native import and device readiness probe.

These exercise real code: the compiled ``spire_lab._native`` extension and
the real optimizer probe. No test is expected to be skipped.
"""

import pytest
import torch

import spire_lab.readiness as readiness
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


def test_check_fails_closed_when_cpu_optimizer_step_is_noop(monkeypatch, capsys):
    # Real CPU forward/backward computation; only the Adam update is neutralized
    # in-memory, so the probe exercises the production path and reports
    # weights_changed=False instead of raising.
    noop_step = lambda self, *args, **kwargs: None
    monkeypatch.setattr(torch.optim.Adam, "step", noop_step)
    with pytest.raises(RuntimeError, match="weights_changed"):
        check(require_cuda=False)
    exit_code = readiness.main([])
    captured = capsys.readouterr()
    assert exit_code != 0
    assert '"status": "failed"' in captured.out


def test_check_fails_closed_when_required_cuda_report_is_unsuccessful(monkeypatch):
    # Validates result checking for a required CUDA probe. The CPU probe
    # stays real; only the CUDA report is faked with failure flags (this
    # exercises validation logic, not GPU success).
    monkeypatch.setattr(torch.cuda, "is_available", lambda: True)
    real_probe = readiness.probe
    failed = {
        "device": "cuda",
        "finite_gradient": False,
        "weights_changed": False,
        "synchronized": False,
        "torch": torch.__version__,
    }
    def fake_probe(device):
        return real_probe(device) if device == "cpu" else dict(failed)
    monkeypatch.setattr(readiness, "probe", fake_probe)
    with pytest.raises(RuntimeError, match="readiness check for cuda failed"):
        check(require_cuda=True)
