"""Small, explicit static-inspection session; never discovers study scope.

Supply five ``pins={role: (absolute_path, sha256)}`` roles: game/java/jar/javap/
cfr. ``allowed_argv`` contains exact approved commands (jar-tf for the pinned
game is implicit). Use read/write/inventory/run/guard_archive/extract_class only
inside ``with``.
All generated paths must be fresh; stdout is retained only at stdout_path.

``final_receipt`` is the actual OUTER harness receipt, available even on failure.
It contains final journal-retention events, inventory and closing fingerprints.
The private receipt explicitly ends before its own retention: it neither predicts
its own hash nor claims future closing. The caller must capture final_receipt in
its outer harness, privately if it contains sensitive paths. No infinite self-log.

This is an operational guard, not a sandbox: trusted pinned static tools and a
single producer are required. It inherits PrivateTree's ancestor-check/TOCTOU
limitations. Never run hostile tools or concurrently mutate the controlled tree.
"""

from __future__ import annotations

import io
import json
import os
from pathlib import Path
import re
import stat
import subprocess
import zipfile

from validation.evidence_report import PrivateTree, ReportError, digest, validate_archive


class AuditError(ReportError):
    """An operation, retention or identity bracket failed; no automatic retry."""


def _json(value):
    return (json.dumps(value, sort_keys=True, indent=2) + "\n").encode()


def _identity(info):
    # Reading can legitimately update atime; no other identity/content metadata
    # may change across the checked descriptor read.
    return (info.st_dev, info.st_ino, info.st_mode, info.st_uid, info.st_gid,
            info.st_nlink, info.st_size, info.st_mtime_ns, info.st_ctime_ns)


class StaticAudit:
    """Single-use context manager. cwd is always the fresh private write_root.

    Supported exact argv shapes: jar tf GAME / jar --version; java -version;
    javap [-p|-private|-s|-c|-verbose|-v] [-classpath GAME] CLASS_OR_CLASSFILE;
    javap -version; java -jar CFR --help (no class input; documented at
    https://www.benf.org/other/cfr/); java -jar CFR PRIVATE_CLASS --methodname NAME
    --innerclasses false --clobber false [--extraclasspath GAME]
    [--silent true|false] [--comments true|false] [--showversion true|false].
    NAME is an ASCII Java identifier or <init>/<clinit>, never a regex or group
    subset. CFR stdout only: outputdir, arbitrary JVM switches, argfiles, whole-
    class decompilation and jar extraction forbidden. Header/member approval and
    complete same-name output verification remain the producing caller's duty.
    Any operational failure blocks further study actions in this context;
    finalization still runs and preserves the first exception as a rejection cause.
    """

    SCRUB = ("JAVA_TOOL_OPTIONS", "_JAVA_OPTIONS", "JDK_JAVA_OPTIONS")
    ROLES = ("game", "java", "jar", "javap", "cfr")

    def __init__(self, *, root, stage, write_root, pins, allowed_argv=(),
                 duplicate_exception=None):
        if set(pins) != set(self.ROLES):
            raise AuditError("exactly five pins required: game/java/jar/javap/cfr")
        self.pins = {role: (Path(path), expected) for role, (path, expected) in pins.items()}
        for path, expected in self.pins.values():
            PrivateTree._lexical(path)
            if not re.fullmatch(r"[a-f0-9]{64}", expected):
                raise AuditError("pin must be a SHA256")
        if len({path for path, _ in self.pins.values()}) != 5:
            raise AuditError("five distinct pin paths required")
        self.tree = PrivateTree(Path(root), Path(stage), Path(write_root))
        self.allowed = frozenset(tuple(map(str, argv)) for argv in allowed_argv)
        for argv in self.allowed:
            self._static(argv)
        self.duplicates = dict(duplicate_exception or {})
        if any(not isinstance(n, int) or n < 2 for n in self.duplicates.values()):
            raise AuditError("duplicate exception requires exact counts >= 2")
        self.operations = []
        self.final_receipt = None
        self._active = self._used = False
        self._sequence = self._event_cursor = self._op_cursor = 0
        self._journal_broken = False
        self._failed = False
        self._first_error = None
        self._owns_root = False

    def _static(self, argv):
        paths = {role: str(path) for role, (path, _) in getattr(self, "_pins", self.pins).items()}
        if argv in ((paths["java"], "-version"), (paths["jar"], "--version"),
                    (paths["javap"], "-version"), (paths["jar"], "tf", paths["game"]),
                    (paths["java"], "-jar", paths["cfr"], "--help")):
            return
        if not argv or any("\x00" in word or word.startswith("@") for word in argv):
            raise AuditError("not a supported static argv")
        if argv[0] == paths["javap"]:
            args = list(argv[1:])
            while args and args[0] in {"-p", "-private", "-s", "-c", "-verbose", "-v"}:
                args.pop(0)
            if args[:2] == ["-classpath", paths["game"]]:
                del args[:2]
            if len(args) == 1 and (re.fullmatch(r"[A-Za-z_$][\w$]*(?:\.[A-Za-z_$][\w$]*)*", args[0])
                                   or self._class_path(args[0])):
                return
        if len(argv) >= 4 and argv[:3] == (paths["java"], "-jar", paths["cfr"]) and self._class_path(argv[3]):
            options = argv[4:]
            if len(options) % 2 == 0:
                pairs = list(zip(options[::2], options[1::2]))
                flags = dict(pairs)
                required = {"--methodname", "--innerclasses", "--clobber"}
                booleans = {"--silent", "--comments", "--showversion"}
                if (len(flags) == len(pairs) and required <= flags.keys()
                        and flags.keys() <= required | booleans | {"--extraclasspath"}
                        and flags["--innerclasses"] == flags["--clobber"] == "false"
                        and (flags["--methodname"] in {"<init>", "<clinit>"}
                             or re.fullmatch(r"[A-Za-z_$][A-Za-z0-9_$]*", flags["--methodname"]))
                        and all(flags[key] in {"true", "false"} for key in booleans & flags.keys())
                        and flags.get("--extraclasspath", paths["game"]) == paths["game"]):
                    return
        raise AuditError("not a supported static argv (game execution forbidden)")

    def _class_path(self, value):
        path = Path(value)
        return path.is_absolute() and ".." not in path.parts and path.is_relative_to(self.tree.stage) and path.suffix == ".class"

    def _record(self, operation, **fields):
        self.operations.append({"number": len(self.operations) + 1, "operation": operation, **fields})

    def _failure(self, operation, error):
        self._failed = True
        if self._first_error is None:
            self._first_error = error
        self._record("failure", attempted=operation, error_type=type(error).__name__, error=str(error))

    def _pin_bytes(self, role, checker):
        """Ancestry/type + checked descriptor read; callers bind expected hash."""
        path, _expected = self._pins[role]
        before = checker.check(path)
        if not stat.S_ISREG(before.st_mode):
            raise AuditError(f"pin not regular: {path}")
        fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW)
        with os.fdopen(fd, "rb") as stream:
            opened = os.fstat(stream.fileno())
            if _identity(before) != _identity(opened):
                raise AuditError(f"pin substituted: {path}")
            data = stream.read()
            if _identity(opened) != _identity(os.fstat(stream.fileno())):
                raise AuditError(f"pin changed while reading: {path}")
        checker.events.append({"operation": "read", "path": str(path), "bytes": len(data), "sha256": digest(data)})
        return data

    def _fingerprints(self):
        # A separate event stream makes it impossible to mistake pin reads for
        # bracketed private-input reads. Every role is attempted once, no retry.
        checker = PrivateTree(self.tree.root, self.tree.stage, self.tree.write_root)
        rows = []
        for role in self.ROLES:
            path, expected = self._pins[role]
            row = {"role": role, "path": str(path), "expected": expected, "match": False}
            start = len(checker.events)
            try:
                data = self._pin_bytes(role, checker)
                row.update(sha256=digest(data), bytes=len(data), match=digest(data) == expected)
            except Exception as error:
                row["error"] = f"{type(error).__name__}: {error}"
            row["events"] = checker.events[start:]
            rows.append(row)
        return rows

    def __enter__(self):
        if self._used:
            raise AuditError("session is single-use, not active")
        self._used = True
        self._pins = dict(self.pins)
        self.opening = self._fingerprints()
        if not all(row["match"] for row in self.opening):
            closing = self._fingerprints()
            self.final_receipt = {"status": "failed", "opening": self.opening,
                                  "closing": closing, "events": [], "operations": [],
                                  "closing_error": None if all(row["match"] for row in closing) else "closing pin mismatch",
                                  "error": "opening pin mismatch"}
            error = AuditError("opening pin mismatch; no private input access")
            if self.final_receipt["closing_error"]:
                error.add_note(self.final_receipt["closing_error"])
            raise error
        self._active = True
        try:
            if self.tree.check(self.tree.write_root, missing=True) is not None:
                raise AuditError("write_root exists; fresh exclusive directory required")
            self.tree._mkdir(self.tree.write_root.parent)
            # Unlike PrivateTree._mkdir, the revision leaf itself MUST NOT reuse
            # an existing directory, including one created during this check.
            os.mkdir(self.tree.write_root, 0o700)
            self._owns_root = True
            self.tree.check(self.tree.write_root)
            self.tree.events.append({"operation": "mkdir", "path": str(self.tree.write_root), "mode": "0700"})
            self._record("open", opening=self.opening)
            self._flush()
        except BaseException as error:
            self.__exit__(type(error), error, error.__traceback__)
            raise
        return self

    def _require(self):
        if not self._active:
            raise AuditError("audit is not active")
        if self._journal_broken:
            raise AuditError("journal previously failed; no retry")
        if self._failed:
            self._record("study_rejected", reason="failed audit context; fresh context required")
            raise AuditError("failed audit context; no further study actions; open a fresh context after diagnosis") from self._first_error

    def _flush(self):
        if self._journal_broken:
            raise AuditError("journal previously failed; no retry")
        self._sequence += 1
        path = self.tree.write_root / "journal" / f"{self._sequence:06d}.json"
        payload = {"number": self._sequence,
                   "operations": self.operations[self._op_cursor:],
                   "events": self.tree.events[self._event_cursor:]}
        # Journal write events belong to the NEXT leaf or the final outer receipt.
        self._op_cursor, self._event_cursor = len(self.operations), len(self.tree.events)
        try:
            self.tree.write(path, _json(payload))
        except BaseException:
            self._journal_broken = True
            raise

    def _call(self, label, action):
        self._require()
        try:
            result = action()
        except BaseException as original:
            self._failure(label, original)
            try:
                self._flush()
            except BaseException as retention:
                self._failure("journal", retention)
                original.add_note(f"distinct journal retention failure: {retention}")
            raise
        self._flush()
        return result

    def read(self, path, expected=None):
        return self._call("read " + str(path), lambda: self.tree.read(Path(path), expected))

    def write(self, path, data):
        def action():
            destination = Path(path)
            if destination == self.tree.write_root / "receipt.json" or destination.is_relative_to(self.tree.write_root / "journal"):
                raise AuditError("reserved audit retention path")
            self.tree.write(destination, data)
        return self._call("write " + str(path), action)

    def inventory(self, roots):
        def action():
            rows = self.tree.inventory(list(map(Path, roots)))
            self._record("inventory", rows=rows)
            return rows
        return self._call("inventory", action)

    def _run(self, argv, stdout_path=None):
        argv = tuple(map(str, argv))
        implicit = (str(self._pins["jar"][0]), "tf", str(self._pins["game"][0]))
        if argv not in self.allowed and argv != implicit:
            self._record("command_rejected", argv=list(argv), cwd=str(self.tree.write_root), reason="not allowlisted")
            raise AuditError("argv not in explicit allowlist")
        self._static(argv)
        env = os.environ.copy()
        for key in self.SCRUB:
            env.pop(key, None)
        row = {"argv": list(argv), "cwd": str(self.tree.write_root),
               "scrubbed": list(self.SCRUB), "umask": "077", "exit": None,
               "stdout_sha256": None, "stderr_sha256": None,
               "stdout_bytes": None, "stderr_bytes": None, "stdout_path": None}
        try:
            self.tree.check(self.tree.write_root)
            pin_paths = {str(path) for path, _expected in self._pins.values()}
            for role in self.ROLES:
                path, expected = self._pins[role]
                if str(path) in argv:
                    data = self._pin_bytes(role, self.tree)
                    actual = digest(data)
                    self._record("pin_check", role=role, path=str(path), sha256=actual, expected=expected, match=actual == expected)
                    if actual != expected:
                        raise AuditError(f"invocation pin mismatch: {path}")
            for word in argv[1:]:
                path = Path(word)
                if word not in pin_paths and path.is_absolute() and path.is_relative_to(self.tree.stage):
                    self.tree.read(path)
            result = subprocess.run(argv, cwd=self.tree.write_root, env=env, umask=0o077,
                                    stdin=subprocess.DEVNULL, stdout=subprocess.PIPE,
                                    stderr=subprocess.PIPE, check=False)
            row.update(exit=result.returncode, stdout_sha256=digest(result.stdout),
                       stderr_sha256=digest(result.stderr), stdout_bytes=len(result.stdout),
                       stderr_bytes=len(result.stderr))
            if stdout_path is not None:
                destination = Path(stdout_path)
                if destination == self.tree.write_root / "receipt.json" or destination.is_relative_to(self.tree.write_root / "journal"):
                    raise AuditError("reserved audit retention path")
                self.tree.write(destination, result.stdout)
                row["stdout_path"] = str(destination)
            if result.returncode:
                raise AuditError(f"static command exit {result.returncode}")
            return result
        except BaseException as error:
            row["error"] = f"{type(error).__name__}: {error}"
            raise
        finally:
            self._record("command", **row)

    def run(self, argv, *, stdout_path=None):
        return self._call("command", lambda: self._run(argv, stdout_path))

    def _guarded_archive(self):
        listing = self._run([self._pins["jar"][0], "tf", self._pins["game"][0]])
        game, expected = self._pins["game"]
        # Reread after listing as well: guard/extraction use these exact pinned
        # bytes, never a filename that could identify a different later archive.
        data = self._pin_bytes("game", self.tree)
        if digest(data) != expected:
            raise AuditError("game pin mismatch")
        self._record("archive_read", path=str(game), sha256=digest(data), bytes=len(data))
        with zipfile.ZipFile(io.BytesIO(data)) as archive:
            guard = validate_archive(archive.infolist(), listing.stdout.decode("utf-8").splitlines(), self.duplicates)
        self._record("archive_guard", **guard)
        return data, guard

    def guard_archive(self):
        """Fresh private-cwd jar-tf and metadata guard, without class extraction."""
        return self._call("archive guard", lambda: self._guarded_archive()[1])

    def extract_class(self, entry, *, output_path=None):
        """Fresh jar-tf and ZipInfo guard EACH call, before reading raw entry bytes.

        Duplicate exceptions permit archive guard only, never ambiguous extraction.
        Returns bytes; an optional explicit private output is exclusive and 0600.
        """
        def action():
            data, _guard = self._guarded_archive()
            with zipfile.ZipFile(io.BytesIO(data)) as archive:
                selected = [info for info in archive.infolist() if info.filename == entry]
                if len(selected) != 1 or not entry.endswith(".class") or selected[0].is_dir() or stat.S_IFMT(selected[0].external_attr >> 16) not in (0, stat.S_IFREG):
                    raise AuditError("class selection must be exact unique regular entry")
                raw = archive.read(selected[0])
            self._record("extract", entry=entry, bytes=len(raw), sha256=digest(raw))
            if output_path is not None:
                destination = Path(output_path)
                if destination == self.tree.write_root / "receipt.json" or destination.is_relative_to(self.tree.write_root / "journal"):
                    raise AuditError("reserved audit retention path")
                self.tree.write(destination, raw)
            return raw
        return self._call("extract " + str(entry), action)

    def __exit__(self, exc_type, original, traceback):
        if not self._active:
            return False
        retention_error = None
        inventory = []
        if original is not None:
            self._failure("context body", original)
        try:
            if not self._owns_root:
                raise AuditError("no fresh write_root owned; retention prohibited")
            self._flush()
            receipt = {"scope": "operations before receipt retention; outer receipt completes audit",
                       "opening": self.opening, "operations": list(self.operations),
                       "events": list(self.tree.events), "body_error": str(original) if original else None}
            path = self.tree.write_root / "receipt.json"
            payload = _json(receipt)
            self.tree.write(path, payload)
            self.tree.read(path, digest(payload))
            self.tree.inventory([self.tree.write_root])
            self._flush()
            # Finite recursion boundary: final leaf verification is outer-only.
            inventory = self.tree.inventory([self.tree.write_root])
        except BaseException as error:
            retention_error = error
            self._failure("final retention", error)
        finally:
            # Absolutely no filesystem/subprocess accesses below this bracket.
            closing = self._fingerprints()
            self._active = False
        closing_error = None if all(row["match"] for row in closing) else "closing pin mismatch"
        failed = self._failed or original is not None or retention_error is not None or closing_error is not None
        self.final_receipt = {"status": "failed" if failed else "closed", "opening": self.opening,
                              "closing": closing, "closing_error": closing_error,
                              "retention_error": str(retention_error) if retention_error else None,
                              "operations": list(self.operations), "events": list(self.tree.events),
                              "inventory": inventory,
                              "boundary": "final retention verification and closing are outer-harness-only"}
        if original is not None:
            if retention_error:
                original.add_note(f"distinct final retention failure: {retention_error}")
            if closing_error:
                original.add_note(closing_error)
            return False
        if retention_error is not None:
            if closing_error:
                retention_error.add_note(closing_error)
            raise retention_error
        if closing_error:
            raise AuditError(closing_error)
        return False
