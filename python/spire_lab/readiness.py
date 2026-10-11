"""Device readiness probing for Spire Lab.

Readiness runs a tiny, isolated forward/backward/optimizer step to prove
that real computation works on the requested device. No game state enters
readiness, and the probe is only a smoke computation - never a validated
learning result.

An unavailable or failing device is a readiness blocker. It is reported
as such; readiness never substitutes simulated GPU success, and a pass is
never inferred from device enumeration alone.
"""

from __future__ import annotations

import json
import subprocess
import sys

import numpy as np
import torch

from . import _native

# Tiny optimizer smoke parameters, using the approved pilot learning rate.
_LEARNING_RATE = 3e-4
_SEED = 7
_SHAPE = (16, 16)

# A readiness report counts as success only when every success flag is True.
_SUCCESS_FLAGS = ("finite_gradient", "weights_changed", "synchronized")

_CLI_FLAGS = ("--require-cuda",)


def _validate_probe(report: dict, device: str) -> None:
    """Fail closed unless every success flag in the probe report is True."""
    failed = [
        f"{name}={report.get(name)!r}"
        for name in _SUCCESS_FLAGS
        if report.get(name) is not True
    ]
    if failed:
        raise RuntimeError(
            f"readiness check for {device} failed: " + ", ".join(failed)
        )


def _driver_version() -> str | None:
    """Report the NVIDIA driver version via nvidia-smi, or ``None``."""
    try:
        output = subprocess.run(
            [
                "nvidia-smi",
                "--query-gpu=driver_version",
                "--format=csv,noheader",
            ],
            capture_output=True,
            text=True,
            check=True,
            timeout=10,
        )
        value = output.stdout.strip().splitlines()[0].strip()
        return value or None
    except Exception:
        return None


def probe(device: str) -> dict:
    """Run a tiny forward/backward/Adam step on ``device``.

    Returns a report with ``finite_gradient`` (finite and nonzero),
    ``weights_changed`` after a real optimizer step, and ``synchronized``
    (CUDA is synchronized when it is used).
    """
    target = torch.device(device)
    generator = torch.Generator(device=target)
    generator.manual_seed(_SEED)

    weight = torch.empty(_SHAPE, device=target, dtype=torch.float32)
    weight.normal_(generator=generator)
    weight.requires_grad_(True)
    before = weight.detach().clone()

    # Forward: a fixed deterministic input through the weight matrix,
    # quadratic loss so gradients are finite and nonzero wherever the
    # weights are.
    identity = torch.eye(_SHAPE[0], device=target, dtype=torch.float32)
    output = weight @ identity
    loss = torch.sum(torch.square(output))

    # Backward: the gradient check happens on the actual computed gradient.
    loss.backward()
    gradient = weight.grad
    finite_gradient = bool(
        gradient is not None
        and torch.isfinite(gradient).all().item()
        and (gradient.abs().sum() > 0).item()
    )

    # Optimizer step: a real Adam update must change the weights.
    optimizer = torch.optim.Adam((weight,), lr=_LEARNING_RATE)
    optimizer.step()
    weights_changed = bool((weight.detach() - before).abs().max().item() > 0)

    # Synchronize CUDA when it is used so the report reflects finished work.
    if target.type == "cuda":
        torch.cuda.synchronize(target)
    synchronized = True

    report: dict = {
        "device": str(target),
        "finite_gradient": finite_gradient,
        "weights_changed": weights_changed,
        "synchronized": synchronized,
        "torch": torch.__version__,
    }
    if target.type == "cuda":
        report.update(
            {
                "cuda_runtime": torch.version.cuda,
                "driver": _driver_version(),
                "gpu": torch.cuda.get_device_name(target),
                "capability": ".".join(
                    str(x) for x in torch.cuda.get_device_capability(target)
                ),
            }
        )
    return report


def check(require_cuda: bool) -> dict:
    """Check device readiness, never inferring a pass from enumeration.

    With ``require_cuda=True``, an unavailable CUDA device raises
    ``RuntimeError``. Every probe report must show
    ``finite_gradient``, ``weights_changed``, and ``synchronized`` as
    ``True``; false or missing flags and probe exceptions fail closed with
    ``RuntimeError`` (and a nonzero CLI exit).
    """
    if require_cuda and not torch.cuda.is_available():
        raise RuntimeError(
            "CUDA is required but no CUDA device is available; "
            "readiness check blocked."
        )

    report: dict = {
        "require_cuda": require_cuda,
        "native": _native.build_info(),
        "wheel": {"torch": torch.__version__, "numpy": np.__version__},
    }

    try:
        cpu_report = probe("cpu")
    except Exception as exc:  # noqa: BLE001 - readiness must fail closed
        raise RuntimeError(f"CPU readiness probe failed: {exc}") from exc
    _validate_probe(cpu_report, "cpu")
    report["cpu"] = cpu_report
    if require_cuda:
        try:
            cuda_report = probe("cuda")
        except Exception as exc:  # noqa: BLE001 - readiness must fail closed
            raise RuntimeError(f"CUDA readiness probe failed: {exc}") from exc
        _validate_probe(cuda_report, "cuda")
        report["cuda"] = cuda_report
    else:
        report["cuda_available"] = bool(torch.cuda.is_available())

    report["status"] = "ok"
    return report


def main(argv: list[str] | None = None) -> int:
    """CLI entry point: emit a JSON readiness report or a failure reason."""
    args = sys.argv[1:] if argv is None else argv
    unknown = [arg for arg in args if arg not in _CLI_FLAGS]
    if unknown:
        print(f"spire-lab readiness: unknown option(s): {', '.join(unknown)}")
        print(f"usage: python -m spire_lab.readiness [{': '.join(_CLI_FLAGS)}]")
        return 2

    require_cuda = "--require-cuda" in args
    try:
        report = check(require_cuda=require_cuda)
    except RuntimeError as exc:
        print(json.dumps({"status": "failed", "error": str(exc)}, indent=2))
        return 1

    print(json.dumps(report, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
