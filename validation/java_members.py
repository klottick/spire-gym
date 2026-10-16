"""Bounded, fail-closed member inspection for javap -s/-c and CFR text.

Pure text utilities, not a Java compiler. All line bounds are 1-based and
inclusive in the original input; block_text preserves the original characters.
Unsupported/ambiguous declaration syntax raises ReportError rather than guessing.
"""

from __future__ import annotations

from dataclasses import dataclass
import re
from collections.abc import Iterable

from validation.evidence_report import ReportError, digest


_ID = r"[A-Za-z_$][\w$]*"
_QUALIFIED = rf"{_ID}(?:\.{_ID})*"
_MODIFIERS = {"public", "protected", "private", "static", "final", "abstract",
              "native", "synchronized", "strictfp", "default", "transient", "volatile"}
_STATEMENT_WORDS = {"return", "throw", "yield", "else", "new", "if", "while", "for",
                    "switch", "case", "catch", "try", "do", "assert", "break", "continue"}


@dataclass(frozen=True)
class JavapMember:
    fqcn: str
    name: str
    descriptor: str
    signature: str
    signature_line: int
    descriptor_line: int
    start_line: int
    end_line: int
    block_text: str

    @property
    def canonical(self) -> tuple[str, str, str]:
        return self.fqcn, self.name, self.descriptor


@dataclass(frozen=True)
class JavapSelection:
    fqcn: str
    name: str
    expected_descriptors: tuple[str, ...]
    members: tuple[JavapMember, ...]
    class_header: str
    class_header_line: int
    source_sha256: str
    text: str


@dataclass(frozen=True)
class CfrMember:
    fqcn: str
    name: str
    signature: str
    signature_line: int
    start_line: int
    end_line: int
    block_text: str


@dataclass(frozen=True)
class CfrGroup:
    fqcn: str
    name: str
    members: tuple[CfrMember, ...]
    source_sha256: str


def _check_fqcn(fqcn: str) -> None:
    if not re.fullmatch(_QUALIFIED, fqcn, re.ASCII):
        raise ReportError("invalid declaring class")


def _descriptor(value: str) -> None:
    # JVMS field types, excluding void except as a scalar return type.
    internal = r"[A-Za-z_$][A-Za-z0-9_$]*(?:/[A-Za-z_$][A-Za-z0-9_$]*)*"
    field = rf"\[*(?:[BCDFIJSZ]|L{internal};)"
    if not re.fullmatch(rf"\((?:{field})*\)(?:V|{field})", value):
        raise ReportError("malformed method descriptor")


def _declaration_name(signature: str, fqcn: str) -> str:
    """Recognize a deliberately bounded Java/javap declaration header."""
    stack = []
    for char in signature:
        if char in "<[":
            stack.append(char)
        elif char in ">]":
            if not stack or stack.pop() != {">": "<", "]": "["}[char]:
                raise ReportError("unbalanced declaration delimiters")
    if stack:
        raise ReportError("unbalanced declaration delimiters")
    match = re.fullmatch(rf"(?P<before>[\w$<>\[\].,?&\s]+)\((?P<args>[^()]*)\)"
                         rf"\s*(?:throws\s+{_QUALIFIED}(?:\s*,\s*{_QUALIFIED})*)?", signature.strip())
    if not match or not re.fullmatch(r"[\w$<>\[\].,?&\s]*", match["args"]):
        raise ReportError("unsupported method declaration")
    before = match["before"].strip()
    token = re.search(rf"({_QUALIFIED})$", before)
    if not token:
        raise ReportError("missing member name")
    raw_name = token[0]
    prefix = before[:token.start()].strip()
    words = prefix.split()
    while words and words[0] in _MODIFIERS:
        words.pop(0)
    remainder = " ".join(words)
    if set(re.findall(_ID, before)) & _STATEMENT_WORDS:
        raise ReportError("statement is not a member declaration")
    # Generic method/constructor type parameters precede the return type.
    if remainder.startswith("<"):
        depth = 0
        for pos, char in enumerate(remainder):
            depth += (char == "<") - (char == ">")
            if depth == 0:
                remainder = remainder[pos + 1:].strip()
                break
        else:
            raise ReportError("unbalanced generic declaration")
    constructor = raw_name in {fqcn, fqcn.rsplit(".", 1)[-1]} and not remainder
    if constructor:
        return "<init>"
    if "." in raw_name or not remainder or raw_name in _MODIFIERS:
        raise ReportError("missing method return type")
    if not re.fullmatch(r"[\w$<>\[\].,?&\s]+", remainder):
        raise ReportError("unsupported return type")
    return raw_name


def _javap_switches(lines: list[str], member_indent: int, is_method: bool) -> None:
    """Accept braces only for an actual, complete javap switch instruction table."""
    code = False
    switch = False
    default = False
    for raw in lines:
        line = raw.strip()
        indent = len(raw) - len(raw.lstrip())
        if switch:
            if indent <= member_indent:
                raise ReportError("switch table escapes javap member")
            if line == "}":
                if not default:
                    raise ReportError("javap switch table lacks default target")
                switch = False
            elif re.fullmatch(r"(?:-?\d+|default):\s*\d+", line):
                if line.startswith("default:"):
                    if default:
                        raise ReportError("duplicate javap switch default")
                    default = True
            else:
                raise ReportError("malformed javap switch table")
        elif line == "}":
            raise ReportError("orphan javap closing brace")
        elif re.fullmatch(r"\d+:\s+(?:tableswitch|lookupswitch)\s+\{\s*(?://.*)?", line):
            if not code or indent <= member_indent:
                raise ReportError("javap switch outside method code")
            switch, default = True, False
        elif any(char in line.split("//", 1)[0] for char in "{}"):
            raise ReportError("unsupported brace-bearing javap fragment")
        elif line == "Code:":
            code = is_method and indent > member_indent
    if switch:
        raise ReportError("unterminated javap switch table")


def parse_javap_members(text: str, fqcn: str) -> tuple[JavapMember, ...]:
    """Parse all methods (including <init>/<clinit>), ignoring field descriptors.

    Requires exactly one matching javap class/interface/enum wrapper and one
    valid descriptor per method. Duplicate canonical identities are rejected.
    Member blocks stop at *any* subsequent member, including fields/initializers.
    """
    _check_fqcn(fqcn)
    lines = text.splitlines(keepends=True)
    headers = [(i, line) for i, line in enumerate(lines)
               if re.search(r"\b(?:class|interface|enum)\s+", line) and line.rstrip().endswith("{")]
    if len(headers) != 1:
        raise ReportError("expected one javap declaring-class wrapper")
    header_index, header = headers[0]
    identity = re.search(rf"\b(?:class|interface|enum)\s+({_QUALIFIED})", header)
    if not identity or identity[1] != fqcn:
        raise ReportError("javap declaring-class mismatch")
    nonempty = [i for i, line in enumerate(lines) if line.strip()]
    close = nonempty[-1]
    if close <= header_index or lines[close].strip() != "}":
        raise ReportError("incomplete javap wrapper")
    boundaries = []
    for i in range(header_index + 1, close):
        line = lines[i].strip()
        # This check MUST precede semicolon/declaration recognition: reference
        # return descriptors end in ';' and contain parentheses.
        if not line or line.startswith("descriptor:"):
            continue
        if line == "static {};":
            boundaries.append((i, "<clinit>"))
        elif line.endswith(";") and re.fullmatch(r"[\w$<>\[\].,?& ();]+", line):
            if "(" in line:
                boundaries.append((i, _declaration_name(line[:-1], fqcn)))
            elif re.fullmatch(rf"[\w$<>\[\].,?&\s]+\s+{_ID};", line):
                boundaries.append((i, None))
    # Standard javap puts declarations at one indentation level and all their
    # descriptors/code metadata deeper. Unknown peer-level text is never allowed
    # to be swallowed into a preceding selected method block.
    if boundaries:
        indentation = min(len(lines[i]) - len(lines[i].lstrip()) for i, _ in boundaries)
        starts = {i for i, _ in boundaries}
        for i in range(header_index + 1, close):
            if lines[i].strip() and len(lines[i]) - len(lines[i].lstrip()) <= indentation and i not in starts:
                raise ReportError("unrecognized top-level javap declaration")
    elif any(line.strip() for line in lines[header_index + 1:close]):
        raise ReportError("unrecognized javap class content")
    if any(line.strip().startswith("descriptor:") for line in lines[header_index + 1:(boundaries[0][0] if boundaries else close)]):
        raise ReportError("orphan javap descriptor")
    if any(line.strip() == "}" for line in lines[header_index + 1:(boundaries[0][0] if boundaries else close)]):
        raise ReportError("extra javap closing wrapper")
    members = []
    seen = set()
    for index, (start, name) in enumerate(boundaries):
        end = boundaries[index + 1][0] if index + 1 < len(boundaries) else close
        while end > start + 1 and not lines[end - 1].strip():
            end -= 1
        _javap_switches(lines[start + 1:end], len(lines[start]) - len(lines[start].lstrip()), name is not None)
        if name is None:
            continue
        descriptors = [(i, line.strip()[len("descriptor:"):].strip())
                       for i, line in enumerate(lines[start + 1:end], start + 1)
                       if line.strip().startswith("descriptor:")]
        if len(descriptors) != 1:
            raise ReportError("missing or duplicate javap method descriptor")
        descriptor_line, descriptor = descriptors[0]
        _descriptor(descriptor)
        if name == "<init>" and not descriptor.endswith(")V"):
            raise ReportError("constructor has non-void descriptor")
        if name == "<clinit>" and descriptor != "()V":
            raise ReportError("invalid static initializer descriptor")
        canonical = fqcn, name, descriptor
        if canonical in seen:
            raise ReportError("duplicate javap member")
        seen.add(canonical)
        members.append(JavapMember(fqcn, name, descriptor, lines[start].strip(), start + 1,
                                   descriptor_line + 1, start + 1, end, "".join(lines[start:end])))
    return tuple(members)


def select_javap_group(text: str, fqcn: str, name: str,
                       expected_descriptors: Iterable[str]) -> JavapSelection:
    """Select only a complete, exactly approved same-name overload group."""
    expected = tuple(expected_descriptors)
    if not expected or len(set(expected)) != len(expected):
        raise ReportError("approved descriptor set must be nonempty and unique")
    for descriptor in expected:
        _descriptor(descriptor)
    members = tuple(m for m in parse_javap_members(text, fqcn) if m.name == name)
    if {m.descriptor for m in members} != set(expected):
        raise ReportError("complete observed overload set differs from approval")
    header_line, header = next((i, line) for i, line in enumerate(text.splitlines(), 1)
                               if re.search(r"\b(?:class|interface|enum)\s+", line) and line.rstrip().endswith("{"))
    selected = header + "\n" + "".join(m.block_text.rstrip("\r\n") + "\n" for m in members) + "}\n"
    return JavapSelection(fqcn, name, tuple(sorted(expected)), members, header, header_line,
                          digest(text.encode("utf-8")), selected)


def _mask_java(source: str) -> str:
    """Mask literals/comments without moving any original offset or newline."""
    out = list(source)
    i = 0
    while i < len(source):
        start = i
        literal = False
        if source.startswith("//", i):
            end = source.find("\n", i)
            i = len(source) if end == -1 else end
        elif source.startswith("/*", i):
            end = source.find("*/", i + 2)
            if end == -1:
                raise ReportError("unterminated CFR comment")
            i = end + 2
        elif source[i] in "\"'":
            literal = True
            if source.startswith('"""', i):
                raise ReportError("unsupported CFR text block")
            quote = source[i]
            i += 1
            while i < len(source) and source[i] != quote:
                if source[i] in "\r\n":
                    raise ReportError("unterminated CFR literal")
                i += 2 if source[i] == "\\" else 1
            if i >= len(source):
                raise ReportError("unterminated CFR literal")
            i += 1
        else:
            i += 1
            continue
        for pos in range(start, i):
            if source[pos] not in "\r\n":
                out[pos] = " "
        if literal:
            # Keep a sentinel so a literal outside a body cannot disappear and
            # turn malformed source into an apparently valid declaration.
            out[start] = source[start]
    return "".join(out)


def _brace_end(mask: str, start: int) -> int:
    depth = 0
    for i in range(start, len(mask)):
        depth += (mask[i] == "{") - (mask[i] == "}")
        if depth == 0:
            return i + 1
    raise ReportError("unbalanced CFR braces")


def validate_cfr_group(source: str, fqcn: str, name: str, expected_count: int) -> CfrGroup:
    """Check actual top-level declarations in a CFR group, not invocation text.

    Accepts a member fragment or one class wrapper; ordinary semicolon-terminated
    fields are ignored. All actual method/constructor declarations must have the
    requested canonical name. One leading bare @Override marker is accepted on
    methods. Other annotations, nested types, initializer blocks, text blocks
    and unrecognized declaration syntax fail closed.
    """
    _check_fqcn(fqcn)
    if type(expected_count) is not int or expected_count < 1:
        raise ReportError("expected positive CFR overload count")
    mask = _mask_java(source)
    start, stop = 0, len(mask)
    package = re.match(rf"\s*package\s+({_QUALIFIED})\s*;", mask)
    if package:
        if package[1] != fqcn.rpartition(".")[0]:
            raise ReportError("CFR package mismatch")
        start = package.end()
    while match := re.match(r"\s*import\s+(?:static\s+)?[\w$.*]+\s*;", mask[start:]):
        start += match.end()
    wrapper = re.match(rf"\s*(?:(?:public|protected|private|abstract|final|static|strictfp)\s+)*"
                       rf"(?:class|interface|enum)\s+({_ID})\b[^{{;]*\{{", mask[start:])
    if wrapper:
        if wrapper[1] != fqcn.rsplit(".", 1)[-1]:
            raise ReportError("CFR declaring-class mismatch")
        opening = start + wrapper.end() - 1
        closing = _brace_end(mask, opening)
        if mask[closing:].strip():
            raise ReportError("extra CFR class/output")
        start, stop = opening + 1, closing - 1
    members = []
    cursor = start
    while cursor < stop:
        while cursor < stop and mask[cursor].isspace():
            cursor += 1
        if cursor == stop:
            break
        begin = cursor
        parens = 0
        while cursor < stop:
            char = mask[cursor]
            parens += (char == "(") - (char == ")")
            if parens < 0:
                raise ReportError("unbalanced CFR declaration parentheses")
            if parens == 0 and char in ";{}":
                break
            cursor += 1
        if cursor >= stop or mask[cursor] == "}" or parens:
            raise ReportError("incomplete CFR declaration")
        header = mask[begin:cursor].strip()
        terminal = mask[cursor]
        override = re.match(r"@Override\s+", header)
        if override:
            header = header[override.end():]
        if re.search(r"\b(class|interface|enum|record)\b", header) or "@" in header:
            raise ReportError("unsupported CFR nested type or annotation")
        if "=" in header or "(" not in header:
            if override or terminal != ";" or not re.match(rf"(?:[\w$<>\[\].,?&]+\s+)+{_ID}\s*(?:=|$)", header):
                raise ReportError("ambiguous CFR field/initializer")
            cursor += 1
            continue
        actual_name = _declaration_name(header, fqcn)
        if override and actual_name == "<init>":
            raise ReportError("Override marker is only supported on methods")
        if actual_name != name:
            raise ReportError("extra CFR member name in selected group")
        end = cursor + 1 if terminal == ";" else _brace_end(mask, cursor)
        if end > stop:
            raise ReportError("CFR member escapes declaring class")
        line = source.count("\n", 0, begin) + 1
        members.append(CfrMember(fqcn, name, source[begin:cursor].strip(), line, line,
                                 source.count("\n", 0, end - 1) + 1, source[begin:end]))
        cursor = end
    if len(members) != expected_count:
        raise ReportError("CFR observed overload count differs from approval")
    return CfrGroup(fqcn, name, tuple(members), digest(source.encode("utf-8")))
