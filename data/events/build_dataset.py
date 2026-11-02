#!/usr/bin/env python3
"""Build the cited, offline event reference; this is not a game simulator."""

import argparse
import hashlib
import json
from pathlib import Path


HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
GRADES = {"S", "BC", "D", "W", "U"}
POOLS = {"ordinary", "shrine", "one_time", "start"}
EXPECTED_IDS = {
    "neow", "upgrade_shrine", "purifier", "ancient_writing", "joust",
    "falling", "tomb_of_lord_red_mask", "big_fish", "dead_adventurer",
    "golden_idol", "mushrooms", "living_wall", "scrap_ooze", "shining_light",
    "cleric", "ssssserpent", "wing_statue", "world_of_goop", "addict",
    "beggar", "colosseum", "cursed_tome", "drug_dealer", "forgotten_altar",
    "ghosts", "masked_bandits", "nest", "library", "mausoleum", "vampires",
    "mind_bloom", "moai_head", "mysterious_sphere", "sensory_stone",
    "winding_halls", "golden_shrine", "match_and_keep", "transmogrifier",
    "wheel_of_change", "accursed_blacksmith", "bonfire_spirits", "face_trader",
    "fountain_of_cleansing", "lab", "note_for_yourself", "we_meet_again",
    "woman_in_blue", "designer", "duplicator", "knowing_skull", "nloth",
    "secret_portal",
}
CAPTURE_FILES = (
    "data/events/sources/wiki-pages.json",
    "data/events/sources/wiki-retrieval.json",
    "data/events/sources/ATTRIBUTION.md",
    "data/events/sources/wiki-pages-v3.json",
    "data/events/sources/wiki-retrieval-v3.json",
    "data/events/sources/ATTRIBUTION-v3.md",
)
SUPPORT_FILES = (
    "docs/research/2026-10-19-act1-content-event-neow.md",
    "docs/research/2026-10-19-act1-content-economy-potions.md",
    "docs/research/2026-10-18-ironclad-a0-acts23-research.md",
    "docs/research/2026-10-23-m9-act3-ending.md",
    "docs/intentional-deviations.md",
    "docs/milestones/m6-act2.md",
    "data/enemies/encounters.json",
    "data/relics/relics.json",
)


def _repo_path(relative):
    path = Path(relative)
    if path.is_absolute() or ".." in path.parts:
        raise ValueError(f"repository path must be relative and contained: {relative!r}")
    resolved = (ROOT / path).resolve()
    if resolved != ROOT and ROOT not in resolved.parents:
        raise ValueError(f"repository path escapes checkout: {relative!r}")
    return resolved


def _read_json(relative):
    with open(_repo_path(relative), "r", encoding="utf-8") as source:
        return json.load(source)


def _read_bytes(relative):
    with open(_repo_path(relative), "rb") as source:
        return source.read()


def _pages():
    pages = {}
    for relative in CAPTURE_FILES:
        if "wiki-pages" not in relative:
            continue
        for page in _read_json(relative):
            key = (page["title"], str(page["revision_id"]))
            if key in pages:
                previous = pages[key]
                # The v3 capture requests Match and Keep! by its display title;
                # it preserves the same permanent revision and wikitext as v2.
                stable_fields = ("title", "revision_id", "revision_timestamp", "wikitext")
                if any(previous.get(field) != page.get(field) for field in stable_fields):
                    raise ValueError(f"conflicting captured page revision: {key}")
                continue
            pages[key] = page
    return pages


def _validate_citation(citation, pages):
    grade = citation.get("grade")
    if grade not in GRADES:
        raise ValueError(f"invalid citation grade: {citation!r}")
    if citation.get("type") == "repo":
        path = citation.get("path")
        line_number = citation.get("line")
        if not isinstance(line_number, int) or line_number < 1:
            raise ValueError(f"repository citation needs a positive line: {citation!r}")
        relative = Path(path)
        if relative.is_absolute() or ".." in relative.parts:
            raise ValueError(f"citation path escapes repository: {citation!r}")
        with open(_repo_path(path), "r", encoding="utf-8") as source:
            lines = source.read().splitlines()
        if line_number > len(lines) or not lines[line_number - 1].strip():
            raise ValueError(f"cited repository line is missing or empty: {citation!r}")
    elif citation.get("type") == "wiki":
        key = (citation.get("page"), str(citation.get("revision_id")))
        if key not in pages:
            raise ValueError(f"captured wiki page/revision is missing: {key}")
    else:
        raise ValueError(f"unknown citation type: {citation!r}")


def _record_citations(record):
    yield from record["eligibility"]["citations"]
    for branch in record["branches"]:
        yield from branch["citations"]


def build_dataset():
    annotations = _read_json("data/events/annotations.json")
    pages = _pages()
    records = annotations["records"]
    if len(records) != 52 or {record["id"] for record in records} != EXPECTED_IDS:
        raise ValueError("event identity set must contain the 52 assigned records")

    output_records = []
    for record in records:
        if record["pool"] not in POOLS:
            raise ValueError(f"invalid pool for {record['id']}: {record['pool']}")
        if not record["acts"] or any(not isinstance(act, int) for act in record["acts"]):
            raise ValueError(f"invalid acts for {record['id']}")
        if record["candidate_source_grade"] not in GRADES:
            raise ValueError(f"invalid candidate source grade for {record['id']}")
        if record["eligibility"]["grade"] not in GRADES:
            raise ValueError(f"invalid eligibility grade for {record['id']}")
        if not record["branches"]:
            raise ValueError(f"event has no branches: {record['id']}")
        for citation in _record_citations(record):
            _validate_citation(citation, pages)
        source_pages = sorted({
            (citation["page"], str(citation["revision_id"]))
            for citation in _record_citations(record) if citation["type"] == "wiki"
        })
        rendered = dict(record)
        rendered["source_pages"] = [
            {"title": title, "revision_id": int(revision)}
            for title, revision in source_pages
        ]
        output_records.append(rendered)

    source_manifest = []
    for relative in (*CAPTURE_FILES, *SUPPORT_FILES):
        source_manifest.append({
            "path": relative,
            "sha256": hashlib.sha256(_read_bytes(relative)).hexdigest(),
        })

    return {
        "schema_version": 1,
        "purpose": "Offline event and shrine implementation reference; not a runtime registry or original-runtime validation.",
        "grade_legend": annotations["grade_legend"],
        "decisions": annotations["decisions"],
        "sources": source_manifest,
        "coverage": {"events_and_shrines": len(output_records), "includes_neow": True},
        "records": output_records,
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="verify events.json without writing it")
    args = parser.parse_args()
    target = _repo_path("data/events/events.json")
    dataset = build_dataset()
    rendered = (json.dumps(dataset, ensure_ascii=False, indent=2) + "\n").encode("utf-8")
    if args.check:
        try:
            actual = _read_bytes("data/events/events.json")
        except FileNotFoundError:
            raise SystemExit("events.json is missing; run build_dataset.py")
        if actual != rendered:
            raise SystemExit("events.json differs from deterministic output")
        print("events.json matches deterministic output")
        return 0
    with open(target, "wb") as destination:
        destination.write(rendered)
    print(f"wrote {len(dataset['records'])} event records to data/events/events.json")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
