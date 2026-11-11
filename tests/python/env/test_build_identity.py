import hashlib
from pathlib import Path

from spire_lab import _native


def _digest(root: Path) -> str:
    paths = set()
    for crate in ("spire-core", "spire-env", "spire-python"):
        base = root / "crates" / crate
        paths.add(base / "Cargo.toml")
        for path in (base / "src").rglob("*.rs"):
            if path.relative_to(base / "src").parts[0] != "tests":
                paths.add(path)
    paths.update(
        root / path
        for path in (
            "crates/spire-python/build.rs",
            "crates/spire-env/src/standard_vocab_v1.json",
            "crates/spire-env/src/standard_event_constants_v1.json",
            "Cargo.toml",
            "Cargo.lock",
            "pyproject.toml",
        )
    )
    digest = hashlib.sha256()
    digest.update(b"spire-file-digest-v1\0")
    for path in sorted(paths, key=lambda item: item.relative_to(root).as_posix().encode()):
        name = path.relative_to(root).as_posix().encode()
        content = path.read_bytes()
        digest.update(len(name).to_bytes(4, "little"))
        digest.update(name)
        digest.update(len(content).to_bytes(8, "little"))
        digest.update(content)
    return digest.hexdigest()


def _source_digest(file_digest: str, features: list[str], profile: str) -> str:
    payload = (
        b"spire-source-digest-v1\0"
        + file_digest.encode()
        + b"\0"
        + "\n".join(sorted(features)).encode()
        + b"\0"
        + profile.encode()
    )
    return hashlib.sha256(payload).hexdigest()


def test_build_identity_matches_checkout():
    root = Path(__file__).resolve().parents[3]
    info = _native.build_info()
    file_digest = _digest(root)
    assert info["file_digest"] == file_digest
    assert info["source_digest"] == _source_digest(file_digest, info["features"], info["profile"])
    assert info["features"] == ["spire-python/extension"]
    assert info["dirty"] in (True, False, "unknown")
