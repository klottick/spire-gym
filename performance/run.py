"""Build and collect opt-in native measurements without touching baseline files."""
import argparse
import datetime
import hashlib
import json
import os
import platform
import subprocess
import sys
import tempfile
import time
from pathlib import Path
from .results import canonical_bytes, digest, integer, load_catalog, require, validate_result

ROOT = Path(__file__).resolve().parent.parent
RELEASE = {"opt-level": 3, "debug": False, "debug-assertions": False,
           "overflow-checks": False, "lto": False, "codegen-units": 16,
           "panic": "unwind", "incremental": False, "strip": "none", "rpath": False}


def invoke(args, root, env=None):
    completed = subprocess.run([str(a) for a in args], cwd=root, env=env, check=True,
                               text=True, capture_output=True)
    if completed.stderr:
        print(completed.stderr, end="", file=sys.stderr)
    return completed.stdout


def source_fingerprint(root):
    paths = invoke(["git", "ls-files", "-z", "--cached", "--others", "--exclude-standard", "--",
                    "crates", "performance", "Cargo.toml", "Cargo.lock", "rust-toolchain.toml", ".cargo"], root)
    fingerprint = hashlib.sha256()
    for name in sorted(set(paths.split("\0")) - {""}):
        path = root / name
        require(not path.is_symlink() and path.resolve().is_relative_to(root.resolve()), "source symlink outside benchmark contract")
        require(path.is_file(), f"missing source file: {name}")
        data = path.read_bytes()
        fingerprint.update(name.encode()); fingerprint.update(b"\0")
        fingerprint.update(str(len(data)).encode()); fingerprint.update(b"\0"); fingerprint.update(data)
    return fingerprint.hexdigest()


def collect_metadata(root, runner_id):
    require(runner_id.strip().lower() not in {"", "unknown", "n/a"}, "runner ID must identify this controlled host")
    rustc = invoke(["rustc", "-vV"], root).strip()
    host = next((line.removeprefix("host: ") for line in rustc.splitlines() if line.startswith("host: ")), "")
    require(host, "missing compiler host target")
    cpu = "unknown"
    try:
        cpu = next((line.split(":", 1)[1].strip() for line in Path("/proc/cpuinfo").read_text().splitlines()
                    if line.startswith("model name")), "unknown")
    except OSError:
        pass
    configs = {}
    for path in sorted([*root.glob(".cargo/config*"), *root.glob("crates/*/.cargo/config*")]):
        if path.is_file():
            configs[str(path.relative_to(root))] = hashlib.sha256(path.read_bytes()).hexdigest()
    return {"provenance": {"git_revision": invoke(["git", "rev-parse", "HEAD"], root).strip(),
                           "dirty": bool(invoke(["git", "status", "--porcelain", "--untracked-files=normal"], root))},
            "machine": {"runner_id": runner_id, "cpu": cpu, "os": f"{platform.system()} {platform.release()}",
                        "arch": platform.machine()},
            "build": {"rustc_vv": rustc, "target": host, "profile": {},
                      "flags": ["encoded-rustflags=", *[f"{k}={json.dumps(v)}" for k,v in sorted(RELEASE.items())]],
                      "config_sha256": digest(configs)}}


def build_and_collect(root, options):
    for key, low, high in [("warmup", 0, 100), ("samples", 1, 100), ("iterations", 1, 1000)]:
        integer(getattr(options, key), low, high, key)
    catalog = load_catalog(root/"performance"/"catalog.json")
    selected = {c["id"] for c in catalog["cases"] if c["status"] == "executable" and
                (options.case is None or options.case == c["id"])}
    require(selected, "unknown or planned case")
    metadata = collect_metadata(root, options.runner_id)
    original_source = source_fingerprint(root)
    env = os.environ.copy()
    for key in ["RUSTC", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER", "CARGO_BUILD_TARGET",
                "CARGO_BUILD_RUSTC", "CARGO_BUILD_RUSTC_WRAPPER", "CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER"]:
        require(not env.get(key), f"unsupported custom compiler/target/wrapper: {key}")
    env.pop("RUSTFLAGS", None)
    env["CARGO_ENCODED_RUSTFLAGS"] = ""
    # Keying build outputs by public source content prevents stale binaries from
    # being reused after timestamp-preserving source replacement.
    target = Path(options.target_dir).resolve()/"collected"/original_source
    env.update(CARGO_TARGET_DIR=str(target), CARGO_BUILD_JOBS="2", RUSTUP_OFFLINE="1")
    args = ["cargo", "build", "--release", "--locked", "--offline", "-p", "spire-core",
            "--example", "performance", "--message-format=json", "--target", metadata["build"]["target"],
            "--config", 'build.rustc="rustc"', "--config", 'build.rustc-wrapper=""',
            "--config", 'build.rustc-workspace-wrapper=""']
    for key, value in RELEASE.items():
        env["CARGO_PROFILE_RELEASE_"+key.upper().replace("-", "_")] = str(value).lower()
        args.extend(["--config", f"profile.release.{key}={json.dumps(value)}"])
    started = time.perf_counter()
    output = invoke(args, root, env)
    build_seconds = time.perf_counter() - started
    artifacts = [m for line in output.splitlines() if (m := json.loads(line)).get("reason") == "compiler-artifact"
                 and m.get("target", {}).get("name") == "performance" and "example" in m["target"].get("kind", [])]
    require(len(artifacts) == 1, "missing or ambiguous performance compiler artifact")
    artifact = artifacts[0]
    executable = Path(artifact.get("executable") or "").resolve()
    require(executable.is_relative_to(target) and executable.is_file(), "mismatched release executable")
    profile = artifact["profile"]
    require(profile.get("opt_level") == "3" and profile.get("debug_assertions") is False and
            profile.get("test") is False, "unexpected benchmark build profile")
    metadata["build"]["profile"] = profile
    command = [str(executable), "--samples", str(options.samples), "--iterations", str(options.iterations),
               "--warmup", str(options.warmup)]
    if options.case: command.extend(["--case", options.case])
    native = validate_result(json.loads(invoke(command, root, env)))
    require(native["schema"] == "spire-gym-native-samples-v1", "not native measurement output")
    require(native["catalog_sha256"] == digest(catalog), "binary/catalog mismatch")
    require(native["sampling"] == {k: getattr(options, k) for k in ("warmup", "samples", "iterations")}, "binary sampling mismatch")
    require({(c["id"],c["boundary"]) for c in native["cases"]} ==
            {(id,boundary) for id in selected for boundary in ("native-transitions", "public-decision-loop")},
            "missing/unexpected benchmark rows")
    require(source_fingerprint(root) == original_source, "source changed during build/collection")
    require(invoke(["git", "rev-parse", "HEAD"], root).strip() == metadata["provenance"]["git_revision"],
            "revision changed during collection")
    native["schema"] = "spire-gym-performance-v1"
    native.update(metadata)
    native["provenance"].update(source_sha256=original_source, build_seconds=build_seconds)
    return validate_result(native)


def publish(path, document, overwrite=False):
    validate_result(document)
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = None
    try:
        with tempfile.NamedTemporaryFile(dir=path.parent, prefix=".performance-", delete=False) as stream:
            temporary = Path(stream.name)
            stream.write(canonical_bytes(document)); stream.flush(); os.fsync(stream.fileno())
        if overwrite:
            os.replace(temporary, path)
        else:
            os.link(temporary, path)  # Atomic exclusive publication, even with concurrent callers.
    finally:
        if temporary is not None: temporary.unlink(missing_ok=True)


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--runner-id", required=True)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--target-dir", type=Path, default=ROOT/"target"/"performance-baseline")
    parser.add_argument("--samples", type=int, default=7)
    parser.add_argument("--iterations", type=int, default=10)
    parser.add_argument("--warmup", type=int, default=1)
    parser.add_argument("--case")
    parser.add_argument("--overwrite", action="store_true")
    args = parser.parse_args(argv)
    stamp = datetime.datetime.now(datetime.timezone.utc).strftime("%Y%m%dT%H%M%S.%fZ")
    output = args.output or ROOT/"target"/"performance-results"/f"{stamp}.json"
    try:
        for key, low, high in [("warmup", 0, 100), ("samples", 1, 100), ("iterations", 1, 1000)]:
            integer(getattr(args, key), low, high, key)
        require(args.runner_id.strip().lower() not in {"", "unknown", "n/a"}, "missing runner identity")
        require(args.overwrite or not output.exists(), "output exists; choose another file or explicitly --overwrite")
        document = build_and_collect(ROOT, args)
        publish(output, document, args.overwrite)
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        print(f"performance collection: {error}", file=sys.stderr)
        if isinstance(error, subprocess.CalledProcessError) and error.stderr:
            print(error.stderr, file=sys.stderr)
        return 2
    print(f"Wrote {output} ({len(document['cases'])} timing rows)", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
