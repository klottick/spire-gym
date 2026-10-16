"""Fail-closed utilities for composing existing, private static evidence.

This module never executes game code or discovers inspection scope. Callers supply
the exact approved paths/digests. Fixtures and this code contain no game evidence.
"""

from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import re
import stat
from collections import Counter


class ReportError(ValueError):
    """Unsafe filesystem state or incomplete evidence binding."""


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


class PrivateTree:
    """Check applicable containers and every generated ancestor before access."""

    def __init__(self, root: Path, stage: Path, write_root: Path):
        self.root, self.stage, self.write_root = map(Path, (root, stage, write_root))
        self.events: list[dict] = []
        for path in (self.root, self.stage, self.write_root):
            self._lexical(path)
        if not self.stage.is_relative_to(self.root) or not self.write_root.is_relative_to(self.stage):
            raise ReportError("outside controlled root")

    @staticmethod
    def _lexical(path: Path):
        if not path.is_absolute() or ".." in path.parts:
            raise ReportError(f"non-lexical absolute path: {path}")

    def check(self, path: Path, *, missing: bool = False):
        """Also usable for pinned external tools; only stage reads use read()."""
        path = Path(path)
        self._lexical(path)
        observations = []
        leaf = None
        for part in reversed((path, *path.parents)):
            try:
                info = part.lstat()
            except FileNotFoundError:
                if missing:
                    continue
                raise ReportError(f"missing: {part}") from None
            if stat.S_ISLNK(info.st_mode):
                raise ReportError(f"symlink: {part}")
            controlled = part in (self.root, self.root / "inspection") or part.is_relative_to(self.stage)
            if controlled:
                if info.st_uid != os.getuid():
                    raise ReportError(f"owner: {part}")
                if stat.S_ISDIR(info.st_mode):
                    expected = 0o700
                elif stat.S_ISREG(info.st_mode) and part == path:
                    expected = 0o600
                    if info.st_nlink != 1:
                        raise ReportError(f"link count: {part}")
                else:
                    raise ReportError(f"type: {part}")
                if stat.S_IMODE(info.st_mode) != expected:
                    raise ReportError(f"mode: {part}")
            elif part != path and not stat.S_ISDIR(info.st_mode):
                raise ReportError(f"ancestor type: {part}")
            observations.append([str(part), info.st_uid, oct(stat.S_IMODE(info.st_mode))])
            if part == path:
                leaf = info
        self.events.append({"operation": "check", "path": str(path), "components": observations})
        return leaf

    def read(self, path: Path, expected: str | None = None) -> bytes:
        path = Path(path)
        if not path.is_relative_to(self.stage):
            raise ReportError(f"outside stage: {path}")
        before = self.check(path)
        if not stat.S_ISREG(before.st_mode):
            raise ReportError(f"not a regular file: {path}")
        fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW)
        with os.fdopen(fd, "rb") as stream:
            opened = os.fstat(stream.fileno())
            if (opened.st_dev, opened.st_ino, opened.st_mode, opened.st_uid, opened.st_nlink) != (
                before.st_dev, before.st_ino, before.st_mode, before.st_uid, before.st_nlink
            ):
                raise ReportError(f"substituted file: {path}")
            data = stream.read()
            after = os.fstat(stream.fileno())
            if (opened.st_size, opened.st_mtime_ns, opened.st_ctime_ns) != (
                after.st_size, after.st_mtime_ns, after.st_ctime_ns
            ):
                raise ReportError(f"changed while reading: {path}")
        actual = digest(data)
        if expected is not None and actual != expected:
            raise ReportError(f"digest mismatch: {path}")
        self.events.append({"operation": "read", "path": str(path), "bytes": len(data), "sha256": actual})
        return data

    def _mkdir(self, path: Path):
        if not path.is_relative_to(self.stage):
            raise ReportError(f"outside stage: {path}")
        info = self.check(path, missing=True)
        if info is not None:
            if not stat.S_ISDIR(info.st_mode):
                raise ReportError(f"not a directory: {path}")
            return
        self._mkdir(path.parent)
        self.check(path, missing=True)
        os.mkdir(path, 0o700)
        self.check(path)
        self.events.append({"operation": "mkdir", "path": str(path), "mode": "0700"})

    def write(self, path: Path, data: bytes):
        path = Path(path)
        self._lexical(path)
        if not path.is_relative_to(self.write_root):
            raise ReportError(f"outside write revision: {path}")
        self._mkdir(path.parent)
        if self.check(path, missing=True) is not None:
            raise ReportError(f"destination exists: {path}")
        fd = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600)
        with os.fdopen(fd, "wb") as stream:
            stream.write(data)
            stream.flush()
            os.fsync(stream.fileno())
        info = self.check(path)
        if info.st_size != len(data):
            raise ReportError(f"incomplete write: {path}")
        self.events.append({"operation": "create", "path": str(path), "bytes": len(data), "sha256": digest(data), "flags": "O_CREAT|O_EXCL|O_NOFOLLOW", "mode": "0600"})

    def inventory(self, roots: list[Path]) -> list[dict]:
        rows = []
        pending = list(roots)
        while pending:
            path = pending.pop()
            if not path.is_relative_to(self.stage):
                raise ReportError(f"outside stage: {path}")
            info = self.check(path)
            if stat.S_ISDIR(info.st_mode):
                with os.scandir(path) as entries:
                    pending.extend(Path(entry.path) for entry in entries)
            else:
                data = self.read(path)
                rows.append({"path": str(path), "bytes": len(data), "sha256": digest(data)})
        return sorted(rows, key=lambda row: row["path"])


def claim_sections(text: str) -> list[dict]:
    """Return exact inclusive line bounds and content, without label promotion."""
    records = []
    for match in re.finditer(r"^```json\n(.*?)\n```", text, re.M | re.S):
        data = json.loads(match.group(1))
        if "id" in data and "evidence_class" in data:
            records.append({"id": data["id"], "lines": [text.count("\n", 0, match.start()) + 1, text.count("\n", 0, match.end()) + 1], "text": match.group(0), "data": data})
    if not records:
        headings = list(re.finditer(r"^#{2,6} .*$", text, re.M))
        for number, heading in enumerate(headings):
            end = headings[number + 1].start() if number + 1 < len(headings) else len(text)
            section = text[heading.start():end]
            identity = re.search(r"^- \*\*id:\*\*\s*`?([^`;\s]+)", section, re.M)
            if identity:
                records.append({"id": identity.group(1), "lines": [text.count("\n", 0, heading.start()) + 1, text.count("\n", 0, end)], "text": section, "data": None})
    if not records or len({record["id"] for record in records}) != len(records):
        raise ReportError("missing or duplicate claim records")
    return records


def artifact_bindings(text: str) -> list[dict]:
    """Pair root-record paths and separately retained digest fields by role."""
    result = []
    for kind, suffix in (("raw", "class"), ("source", "java"), ("bytecode", "txt")):
        paths = set(re.findall(rf"(?<!\w){kind}\s+`(/[^`\s]+?\.{suffix})(?=[:`])", text))
        hashes = set(re.findall(rf"(?<!\w){kind}(?:\s+`[^`]+`)?\s+SHA256\s+`([a-f0-9]{{64}})`", text))
        if len(paths) != 1 or len(hashes) != 1:
            raise ReportError(f"missing or ambiguous {kind} binding")
        result.append({"kind": "raw_class" if kind == "raw" else kind, "path": paths.pop(), "sha256": hashes.pop()})
    return result


def line_ranges(text: str, line_count: int) -> list[list[int]]:
    ranges = [[int(a), int(b)] for a, b in re.findall(r"(?<![\w/])(\d+)\s*[-–]\s*(\d+)", text)]
    if not ranges or any(not 1 <= start <= end <= line_count for start, end in ranges):
        raise ReportError(f"missing or invalid line range: {text}")
    return ranges


def bytecode_anchor(text: str, members: list[list[str]], ranges: list[list[int]], offsets: list[list[int]], *, offset_names: list[str] | None = None) -> dict:
    """Verify mechanical anchors, not instruction meaning or runtime behavior."""
    lines = text.splitlines()
    header = re.search(r"^(?:(?:public|protected|private|abstract|final) +)*class +([\w.$]+)", text, re.M)
    if not header or any(member[0] != header.group(1) for member in members):
        raise ReportError("declaring class/member mismatch")
    declarations = list(re.finditer(r"^  (?:(?:public|protected|private)\b[^\n]*|static \{\};)\s*$", text, re.M))
    found = {}
    for number, declaration in enumerate(declarations):
        signature = declaration.group(0).strip()
        if "(" not in signature and signature != "static {};":
            continue
        end = declarations[number + 1].start() if number + 1 < len(declarations) else len(text)
        block = text[declaration.start():end]
        descriptor = re.search(r"^\s+descriptor:\s*(\S+)", block, re.M)
        if not descriptor:
            raise ReportError("missing member descriptor")
        if signature == "static {};":
            name = "<clinit>"
        else:
            name = signature.split("(", 1)[0].split()[-1]
            if name == header.group(1):
                name = "<init>"
        start_line = text.count("\n", 0, declaration.start()) + 1
        instructions = [(int(match.group(1)), start_line + block.count("\n", 0, match.start())) for match in re.finditer(r"^\s+(\d+):\s+[a-z][a-z0-9_]*\b", block, re.M)]
        key = (header.group(1), name, descriptor.group(1))
        if key in found:
            raise ReportError("duplicate member block")
        found[key] = instructions
    selected = {}
    for member in members:
        if tuple(member) not in found:
            raise ReportError(f"missing member: {member}")
        selected[tuple(member)] = found[tuple(member)]
    line_bindings = []
    for start, end in ranges:
        matching = [list(key) for key, instructions in selected.items() if any(start <= line <= end for _, line in instructions)]
        if not 1 <= start <= end <= len(lines) or not matching:
            raise ReportError("instruction line range/member mismatch")
        line_bindings.append({"lines": [start, end], "members": matching})
    if offset_names is not None and len(offset_names) != len(offsets):
        raise ReportError("offset member-name count mismatch")
    offset_bindings = []
    for number, (start, end) in enumerate(offsets):
        matching = []
        for key, instructions in selected.items():
            if offset_names is not None and key[1] != offset_names[number]:
                continue
            anchored_offsets = {offset for offset, line in instructions if not ranges or any(a <= line <= b for a, b in ranges)}
            if {start, end} <= anchored_offsets:
                matching.append(list(key))
        if not 0 <= start <= end or not matching:
            raise ReportError(f"instruction offset range/member mismatch: {start}-{end}")
        offset_bindings.append({"offsets": [start, end], "members": matching})
    return {"members": members, "lines": ranges, "offsets": offsets, "line_member_bindings": line_bindings, "offset_member_bindings": offset_bindings, "result": "PASS", "check": "same canonical member binds descriptor, instruction lines and offset endpoints; explicit named ranges preserved; not semantic re-review"}


def source_anchor(text: str, members: list[list[str]], ranges: list[list[int]]) -> dict:
    """Associate approved Java declaration names and bounded source lines."""
    lines = text.splitlines()
    if any(not 1 <= start <= end <= len(lines) for start, end in ranges):
        raise ReportError("source line range out of bounds")
    declarations = []
    for fqcn, name, _descriptor in members:
        if name == "<clinit>":
            pattern = r"^\s*static\s*\{"
        else:
            java_name = fqcn.rsplit(".", 1)[-1] if name == "<init>" else name
            pattern = rf"^\s*(?:public|protected|private)\s+[^;\n]*\b{re.escape(java_name)}\s*\("
        matches = [number for number, line in enumerate(lines, 1) if re.search(pattern, line)]
        if not matches:
            raise ReportError(f"missing source declaration: {fqcn}.{name}")
        declarations.extend(matches)
    for start, end in ranges:
        if not any(start <= declaration <= end for declaration in declarations):
            raise ReportError("source range does not contain a selected member declaration")
    return {"members": members, "lines": ranges, "declaration_lines": sorted(set(declarations)), "result": "PASS", "check": "each source range contains a selected declaration; JVM descriptors remain header/allowlist-bound; body-only source ranges fail closed"}


def allowlist_binding(text: str, entry: str, members: list[list[str]], raw_digest: str, *, supersession: dict | None = None) -> dict:
    lines = text.splitlines()
    starts = [number for number, line in enumerate(lines) if line == "### " + entry]
    if not starts:
        starts = [number for number, line in enumerate(lines) if "zip_entry" in line and f"`{entry}`" in line and "raw_class_sha256" in line]
    if len(starts) != 1:
        raise ReportError("missing or ambiguous allowlist entry")
    start = starts[0]
    end = next((number for number in range(start + 1, len(lines)) if lines[number].startswith(("## ", "### "))), len(lines))
    section = lines[start:end]
    raw_lines = [line for line in section if "raw_class_sha256" in line]
    if len(raw_lines) != 1:
        raise ReportError("missing or ambiguous raw digest")
    hashes = re.findall(r"`([a-f0-9]{64})`", raw_lines[0])
    if len(hashes) != 1 or hashes[0] != raw_digest:
        if not supersession or supersession.get("entry") != entry or supersession.get("effective") != raw_digest or supersession.get("bad") not in raw_lines[0]:
            raise ReportError("raw digest mismatch")
        exact = "### " + supersession["heading"]
        if lines.count(exact) != 1:
            raise ReportError("missing explicit supersession")
        override = "\n".join(lines[lines.index(exact):])
        if f"`{entry}`" not in override or f"`{raw_digest}`" not in override or f"`{supersession['bad']}`" not in override or "`raw_class_sha256` only" not in override:
            raise ReportError("supersession does not bind exact digest cell")
    for fqcn, name, descriptor in members:
        if fqcn.replace(".", "/") + ".class" != entry or not any(f"`{name}`" in line and f"`{descriptor}`" in line for line in section):
            raise ReportError(f"allowlist member mismatch: {fqcn}.{name}{descriptor}")
    return {"entry": entry, "lines": [start + 1, end], "raw_class_sha256": raw_digest, "member_count": len(members), "result": "PASS", "supersession": supersession if len(hashes) != 1 or hashes[0] != raw_digest else None}


def validate_archive(infos, listing: list[str], duplicate_exception: dict[str, int]) -> dict:
    names = [info.filename for info in infos]
    if names != listing:
        raise ReportError("archive listing mismatch")
    duplicates = {name: count for name, count in Counter(names).items() if count > 1}
    if duplicates != duplicate_exception:
        raise ReportError("unauthorized duplicate entries")
    for info in infos:
        name = info.filename
        if not name or name.startswith("/") or re.match(r"^[A-Za-z]:", name) or "\\" in name or "\x00" in name or ".." in name.split("/"):
            raise ReportError(f"unsafe archive entry: {name}")
        if stat.S_IFMT(info.external_attr >> 16) not in (0, stat.S_IFREG, stat.S_IFDIR):
            raise ReportError(f"unsafe archive type: {name}")
    return {"entries": len(infos), "unique_names": len(set(names)), "duplicates": duplicates, "result": "PASS", "duplicate_contents": "unread/unextracted"}


def tool_bindings(record: dict, allowed: set[str]) -> list[str]:
    hashes = [value for key, value in record.items() if key != "name"]
    if not hashes or any(value not in allowed for value in hashes):
        raise ReportError("missing or mismatched tool hash")
    return sorted(set(hashes))


def instruction_ranges(text: str) -> list[list[int]]:
    unquoted = re.sub(r"`[^`]+`", "", text)
    ranges = [[int(a), int(b)] for a, b in re.findall(r"(\d+)\s*[-–]\s*(\d+)", unquoted)]
    if not ranges or any(start > end for start, end in ranges):
        raise ReportError("missing or invalid instruction offset range")
    return ranges


def listing_anchor(text: str, line_count: int) -> dict:
    parenthetical = re.findall(r"\((?:(?:javap(?: listing)? )?lines )?(\d+\s*[-–]\s*\d+)\)", text)
    ranges = line_ranges(",".join(parenthetical), line_count)
    offsets = instruction_ranges(re.sub(r"\([^)]*\)", "", text))
    return {"lines": ranges, "offsets": offsets}


def named_instruction_ranges(text: str, aliases: dict[str, str]) -> list[tuple[str, list[int]]]:
    unquoted = re.sub(r"`(/[^`]+)`", "", text)
    unquoted = unquoted.replace("`", "")
    unquoted = re.sub(r"\([^)]*\)", "", unquoted)
    found = re.findall(r"([A-Za-z_$<>][\w.$<>]*)[ \t]+(?:instruction[ \t]+)?(?:offsets[ \t]*)?(\d+)\s*[-–]\s*(\d+)", unquoted)
    result = [(aliases.get(name, name), [int(start), int(end)]) for name, start, end in found]
    if not result or [bounds for _, bounds in result] != instruction_ranges(unquoted):
        raise ReportError("missing named instruction/member range")
    return result
