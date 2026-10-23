"""Explicit optional original-art preparation; never imported by capture or export.

Use --local-dir for user-supplied originals or --download --key for named HTTPS
acquisitions. Original game artwork remains owned by Mega Crit, never this repo.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import sys
import urllib.parse
import urllib.request

MAX_IMAGE_BYTES = 16 * 1024 * 1024


class HttpsOnlyRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        if urllib.parse.urlparse(newurl).scheme != "https":
            raise ValueError("refusing non-HTTPS redirect before following it")
        return super().redirect_request(req, fp, code, msg, headers, newurl)


def image_extension(data):
    # Passive header screening only, not a decoder or original-art authenticity check.
    if (len(data) >= 45 and data.startswith(b"\x89PNG\r\n\x1a\n")
            and data[8:16] == b"\0\0\0\rIHDR" and int.from_bytes(data[16:20], "big")
            and int.from_bytes(data[20:24], "big") and data.endswith(b"\0\0\0\0IEND\xaeB`\x82")):
        return ".png"
    if len(data) >= 20 and data.startswith(b"\xff\xd8\xff") and data.endswith(b"\xff\xd9"):
        return ".jpg"
    if (len(data) >= 20 and data[:4] == b"RIFF" and data[8:12] == b"WEBP"
            and data[12:16] in (b"VP8 ", b"VP8L", b"VP8X")
            and int.from_bytes(data[4:8], "little") + 8 == len(data)):
        return ".webp"
    raise ValueError("image must have a valid passive PNG/JPEG/WebP header")


def check_destination(path, expected):
    if path.is_symlink():
        raise ValueError(f"refusing symlink destination: {path}")
    if path.exists() and (not path.is_file() or path.read_bytes() != expected):
        raise ValueError(f"refusing to overwrite different destination: {path}")


def write_exclusive(path, data):
    check_destination(path, data)
    try:
        fd = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600)
    except FileExistsError:
        # Idempotent same-byte cache reuse is not an overwrite. Recheck any race.
        check_destination(path, data)
        return
    with os.fdopen(fd, "wb") as out:
        out.write(data)
        out.flush()
        os.fsync(out.fileno())


def prepare(args):
    sources = json.loads(args.sources.read_text())
    if not isinstance(sources, dict):
        raise ValueError("sources must be a key-to-HTTPS-URL object")
    keys = sorted(set(args.key or sources.keys()))
    if args.download and not args.key:
        raise ValueError("--download requires at least one explicitly named --key")
    selected = {}
    for key in keys:
        if key not in sources or not isinstance(sources[key], str):
            raise ValueError(f"unknown or invalid source key: {key}")
        url = urllib.parse.urlparse(sources[key])
        filename = urllib.parse.unquote(url.path.rsplit("/", 1)[-1])
        if (url.scheme != "https" or not url.netloc or not filename
                or filename in (".", "..") or "/" in filename or "\\" in filename):
            raise ValueError(f"source must be a named HTTPS image: {key}")
        selected[key] = (sources[key], filename)
    output = args.output_dir.absolute()
    for path in (output, *output.parents, output / "assets"):
        if path.is_symlink():
            raise ValueError(f"refusing symlink cache directory: {path}")
    images = {}
    entries = {}
    local = args.local_dir.resolve() if args.local_dir else None
    for key, (url, filename) in selected.items():
        if local is not None:
            source = local / filename
            if source.is_symlink() or not source.resolve().is_relative_to(local):
                raise ValueError(f"local artwork escapes originals directory: {key}")
            if not source.exists():
                print(f"Missing local original for {key}; labels will remain available", file=sys.stderr)
                continue
            with source.open("rb") as image:
                data = image.read(MAX_IMAGE_BYTES + 1)
        else:
            request = urllib.request.Request(url, headers={"User-Agent": "Spire-Gym-optional-art-preparation"})
            with urllib.request.build_opener(HttpsOnlyRedirect()).open(request, timeout=30) as image:
                if urllib.parse.urlparse(image.geturl()).scheme != "https":
                    raise ValueError("refusing non-HTTPS redirect")
                data = image.read(MAX_IMAGE_BYTES + 1)
        if len(data) > MAX_IMAGE_BYTES:
            raise ValueError(f"artwork exceeds size limit: {key}")
        filename = hashlib.sha256(data).hexdigest() + image_extension(data)
        relative = "assets/" + filename
        images[relative] = data
        entries[key] = {"path": relative, "source": url}
    manifest = json.dumps({"schemaVersion": 1, "assets": entries,
                           "attribution": ["Original Slay the Spire artwork belongs to Mega Crit Games."]},
                          indent=2, sort_keys=True).encode() + b"\n"
    # Check every destination before creating any cache file.
    check_destination(output / "manifest.json", manifest)
    for relative, data in images.items():
        check_destination(output / relative, data)
    output.mkdir(parents=True, exist_ok=True)
    (output / "assets").mkdir(exist_ok=True)
    for relative, data in images.items():
        write_exclusive(output / relative, data)
    write_exclusive(output / "manifest.json", manifest)
    return 0


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--sources", type=Path, default=Path(__file__).resolve().parents[1] / "crates/spire-report/assets/sources.json")
    parser.add_argument("--output-dir", type=Path, required=True)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--local-dir", type=Path)
    mode.add_argument("--download", action="store_true")
    parser.add_argument("--key", action="append")
    try:
        args = parser.parse_args(argv)
    except SystemExit as error:
        return int(error.code)
    try:
        return prepare(args)
    except (OSError, ValueError) as error:
        print(f"Artwork preparation failed: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
