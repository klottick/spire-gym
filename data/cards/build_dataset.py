#!/usr/bin/env python3
"""Regenerate the offline card reference; this is not a simulator or mechanics verifier.

Uses only public wiki snapshots, an attributed identity map, and existing authored
research. No compiler, original game, network, or third-party packages are needed.
"""

import argparse
import collections
import hashlib
import html
import json
from pathlib import Path
import re
import urllib.parse


HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
WIKI = "https://slaythespire.wiki.gg/wiki/"
CHARACTERS = {"RED": "IRONCLAD", "GREEN": "SILENT", "BLUE": "DEFECT", "PURPLE": "WATCHER"}
CHOICES = {"BECOME_ALMIGHTY", "LIVE_FOREVER", "FAME_AND_FORTUNE"}
WATCHER_SPECIALS = {"BETA", "OMEGA", "EXPUNGER", "MIRACLE", "INSIGHT", "SAFETY", "SMITE", "THROUGH_VIOLENCE"}
EVENT_CARDS = {"APPARITION", "BITE", "JAX", "RITUAL_DAGGER"}
TARGETED_SKILLS = {
    "CATALYST", "CORPSE_EXPLOSION", "DARK_SHACKLES", "DEADLY_POISON", "DISARM",
    "JUDGMENT", "LEG_SWEEP", "MALAISE", "PRESSURE_POINTS", "SPOT_WEAKNESS", "TERROR",
}
RESEARCH = {
    "IRONCLAD": "docs/research/2026-10-19-act1-content-red-colorless.md",
    "SHARED": "docs/research/2026-10-19-act1-content-red-colorless.md",
    "SILENT": "docs/research/2026-10-19-act1-content-silent.md",
    "DEFECT": "docs/research/2026-10-19-act1-content-defect.md",
    "WATCHER": "docs/research/2026-10-19-act1-content-watcher.md",
}


def load(path):
    return json.loads(path.read_text(encoding="utf-8"))


def render_description(raw, upgraded):
    """Resolve Cargo's (base;upgrade) alternatives while preserving energy images."""
    text = re.sub(r"\[\[File:[^\]|]*Energy\.png[^\]]*\]\]", " [E] ", raw, flags=re.I)
    text = re.sub(r"\[\[(?:File|Image):[^\]]*\]\]", "", text, flags=re.I)
    text = re.sub(r"<br\s*/?>", "\n", text, flags=re.I)
    text = re.sub(r"<[^>]*>", "", text)
    text = re.sub(r"\[\[([^\[\]]+)\]\]", lambda m: m[1].split("|")[-1], text)
    text = re.sub(r"\[(https?://[^\s\]]+)(?:\s+([^\]]+))?\]", lambda m: m[2] or m[1], text)
    text = html.unescape(text).replace("'''", "").replace("''", "")
    text = re.sub(r"\(([^();]*);([^();]*)\)", lambda m: m[2 if upgraded else 1], text)
    text = "\n".join(re.sub(r"[ \t]+", " ", line).strip() for line in text.splitlines())
    text = re.sub(r"\s+([.,])", r"\1", text).strip()
    if re.search(r"\[\[|\[https?://|\{\{|<[^>]+>|\([^)]*;[^)]*\)", text):
        raise ValueError(f"Unhandled wiki markup: {text!r}")
    return text


def requires_enemy_target(card_id, card_type, description, upgraded):
    # A player-selected enemy, not the original game's full target enum or legality.
    if card_id in {"BLIND", "TRIP"}:
        return not upgraded
    if card_type == "ATTACK":
        return not re.search(r"all enemies|random enemy", description, re.I)
    return card_id in TARGETED_SKILLS


def make_variant(row, card_id, upgraded, vocabulary):
    description = render_description(row["Description"], upgraded)
    cost = int((row["CostPlus"] or row["Cost"]) if upgraded else row["Cost"])
    flags = {
        flag: bool(re.search(r"(?:^|[.!?]\s+)" + flag + r"(?:\.(?:\s|$)|$)", description, re.I))
        for flag in ("innate", "ethereal", "exhaust", "retain", "unplayable")
    }
    # Its draw-triggered self-exhaust is embedded in a sentence, not standalone.
    if card_id == "DEUS_EX_MACHINA":
        flags["exhaust"] = True
    flags["unplayable"] = flags["unplayable"] or cost == -2
    damage = re.search(r"\bDeal (\d+) damage\b", description, re.I)
    block = re.search(r"\bGain (\d+) Block\b", description, re.I)
    keywords = {word for word in vocabulary if re.search(
        r"(?<!\w)" + re.escape(word) + r"(?!\w)", description, re.I
    )}
    if "[E]" in description:
        keywords.add("Energy")
    return {
        "cost": cost, "description": description,
        "printed_damage": int(damage[1]) if damage else None,
        "printed_block": int(block[1]) if block else None,
        "requires_enemy_target": bool(requires_enemy_target(card_id, row["Type"].upper(), description, upgraded)),
        "flags": flags, "keywords": sorted(keywords),
    }


def wiki_url(link):
    # Cargo encodes fragment delimiters and HTML-escapes apostrophes in Link.
    decoded = html.unescape(urllib.parse.unquote(link)).replace(" ", "_")
    return WIKI + urllib.parse.quote(decoded, safe="_#().-'")


def availability(card_id, row):
    if card_id in CHOICES:
        return {"kind": "choice_only", "standard_run": True, "parent": "WISH"}
    if card_id == "PRIDE":
        return {"kind": "endless_mode_curse", "standard_run": False}
    if card_id == "ASCENDERS_BANE":
        return {"kind": "starting_curse", "standard_run": True, "minimum_ascension": 10}
    if row["Type"] in {"Curse", "Status"}:
        return {"kind": row["Type"].lower(), "standard_run": True}
    if row["Rarity"] == "Basic":
        return {"kind": "starter", "standard_run": True}
    if card_id in EVENT_CARDS:
        return {"kind": "event", "standard_run": True}
    if row["Rarity"] == "Special":
        return {"kind": "generated", "standard_run": True}
    return {"kind": "ordinary_pool", "standard_run": True}


def local_research(card_id, character):
    if card_id == "PRIDE":
        return []  # Endless-mode content is not covered by the standard-run drafts.
    family = character or "SHARED"
    if card_id in WATCHER_SPECIALS or card_id in CHOICES:
        family = "WATCHER"
    elif card_id == "SHIV":
        family = "SILENT"
    return [{
        "path": RESEARCH[family],
        "grade": "source-grounded author draft; unresolved composition fields retained",
        "scope": "Historical Ironclad A0 Act 1/off-color reachability study, not full-character run acceptance",
    }]


def snapshot(path):
    return {"path": str(path.relative_to(ROOT)), "sha256": hashlib.sha256(path.read_bytes()).hexdigest()}


def build_dataset():
    rows = load(HERE / "sources/wiki-cards.json")
    pages = {p["title"]: p for p in load(HERE / "sources/wiki-pages.json")}
    retrieval = load(HERE / "sources/wiki-retrieval.json")
    identities = load(HERE / "sources/identifiers.json")
    notes = load(HERE / "implementation-notes.json")
    annotations = load(HERE / "research-annotations.json")
    by_name = {row["Name"]: row for row in rows}
    id_by_name = {item["wiki_name"]: item["id"] for item in identities["records"]}
    if len(rows) != len(by_name) or set(by_name) != set(id_by_name):
        raise ValueError("Wiki/identity coverage differs; review the source snapshots")
    if any(row["Game"] != "1" for row in rows):
        raise ValueError("A non-StS1 row was included")
    if (set(notes) | set(annotations)) - set(id_by_name.values()):
        raise ValueError("Notes refer to nonexistent reference IDs")
    vocabulary = set()
    for row in rows:
        for tag in re.findall(r"<span\b[^>]*>", row["Description"]):
            if re.search(r'class=[\'"][^\'"]*(?:keyword|power)-trigger', tag):
                match = re.search(r'data-name=[\'"]([^\'"]+)', tag)
                if match and not match[1].startswith("@"):
                    vocabulary.add(html.unescape(match[1]))
    cards = []
    for item in sorted(identities["records"], key=lambda item: item["id"]):
        card_id = item["id"]
        row = by_name[item["wiki_name"]]
        mode = "none" if row["NoUpgrade"] == "1" else "standard"
        upgrade = {"mode": mode, "max_player_upgrades": 0 if mode == "none" else 1}
        if card_id == "BURN":
            upgrade = {"mode": "enemy_only", "max_player_upgrades": 0}
        elif card_id == "SEARING_BLOW":
            upgrade = {"mode": "repeatable", "max_player_upgrades": None,
                       "damage_formula": "12 + n * (n + 7) / 2"}
        elif card_id in CHOICES:
            upgrade = {"mode": "parent_choice", "max_player_upgrades": 0, "parent": "WISH"}
        related_names = re.findall(
            r'<span class="card-trigger[^>]*data-name="([^"]+)"', row["Description"]
        )
        related = {id_by_name[name] for name in related_names}
        if card_id == "WISH":
            related.update(CHOICES)
        page = pages[row["_pageName"]]
        character = CHARACTERS.get(row["Color"].upper())
        cards.append({
            "id": card_id, "game_id": item["game_id"], "name": item["name"],
            "character": character,
            "color": "CURSE" if row["Type"] == "Curse" else row["Color"].upper(),
            "rarity": row["Rarity"].upper(), "type": row["Type"].upper(),
            "base": make_variant(row, card_id, False, vocabulary),
            "upgraded": None if mode == "none" else make_variant(row, card_id, True, vocabulary),
            "upgrade": upgrade, "availability": availability(card_id, row),
            "related_cards": sorted(related - {card_id}),
            "implementation_notes": notes.get(card_id, []),
            "research_notes": annotations.get(card_id, {}).get("notes", []),
            "discrepancies": annotations.get(card_id, {}).get("discrepancies", []),
            "source": {
                "id": "sts-wiki", "page": page["title"], "url": wiki_url(row["Link"]),
                "revision_id": page["revision_id"],
                "revision_url": WIKI + "Special:PermanentLink/" + str(page["revision_id"]),
                "wiki_name": row["Name"], "wiki_color": row["Color"], "wiki_rarity": row["Rarity"],
            },
            "local_research": local_research(card_id, character),
            "evidence": {"primary": "community-wiki description", "original_runtime_validated": False},
        })
    return {
        "schema_version": 1, "game": "Slay the Spire", "edition": "unmodded PC", "target_version": "2.3",
        "retrieved_at": retrieval["retrieved_at"],
        "purpose": "Offline implementation reference for future all-character Spire Gym content; not a runtime content registry.",
        "limitations": [
            "PC v2.3 is the target, not a binary-verified version claim. The wiki is a community source and its pages can contain errors.",
            "Existing local research is cited at its draft/source-grounded grade; it is not promoted to accepted full-character or original-runtime evidence.",
            "Descriptions and printed quantities do not specify callback ordering, RNG-stream assignment, copying, modifier rounding, legal autoplay, or terminal queue completion.",
            "Reference IDs and game string IDs are not Spire Gym numeric registry IDs. Historical simulator mechanics metadata is not imported as truth.",
            "Pool/availability labels are broad acquisition categories, not complete reward/transform/generator eligibility or probability laws.",
            "Wiki STATUS/CURSE rarity labels are display taxonomy, not necessarily the original game's CardRarity enum. Source labels are preserved.",
            "Cargo rows and page text were fetched separately. Page revision IDs identify the supporting page snapshots, not a transactional revision of the Cargo table.",
        ],
        "sources": [
            {"id": "sts-wiki", "title": "Slay the Spire Wiki contributors: StS1 Cards table and card pages",
             "url": WIKI + "Cards_List", "query_url": retrieval["cargo_query_url"],
             "license": "CC-BY-SA-4.0", "license_url": "https://creativecommons.org/licenses/by-sa/4.0/",
             "snapshots": [snapshot(path) for path in sorted((HERE / "sources").glob("wiki-*.json"))]},
            {"id": "identifiers", "title": "gamerpuppy/sts_lightspeed: identity map only",
             "url": identities["source_url"], "revision": identities["revision"],
             "license": "MIT", "snapshots": [snapshot(HERE / "sources/identifiers.json")]},
            {"id": "local-research", "title": "Existing independently worded Spire Gym research drafts",
             "grade": "draft/source-grounded, not original-runtime validated",
             "snapshots": [snapshot(ROOT / path) for path in sorted(set(RESEARCH.values()))]},
        ],
        "coverage": {
            "cards": len(cards), "physical_card_kinds": len(cards) - len(CHOICES), "choice_only_kinds": len(CHOICES),
            "wiki_pages": len(pages),
            "characters": dict(sorted(collections.Counter(c["character"] or "SHARED" for c in cards).items())),
        },
        "cards": cards,
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="Compare regenerated data without modifying files")
    args = parser.parse_args()
    dataset = build_dataset()
    output = HERE / "cards.json"
    text = json.dumps(dataset, ensure_ascii=False, indent=2) + "\n"
    if args.check:
        if output.read_text(encoding="utf-8") != text:
            raise SystemExit("Dataset differs from its inputs; run data/cards/build_dataset.py")
        print(f"Reference regenerates identically: {dataset['coverage']['cards']} records")
    else:
        output.write_text(text, encoding="utf-8")
        print(json.dumps(dataset["coverage"], indent=2))


if __name__ == "__main__":
    main()
