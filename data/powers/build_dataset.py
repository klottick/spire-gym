#!/usr/bin/env python3
"""Build a qualified, offline buff/debuff reference, not a gameplay registry."""

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
CATALOG = "docs/research/2026-10-19-act1-complete-catalog-numeric.md"
RESEARCH = {
    "M8": "data/enemies/m8-static-findings.md",
    "RED": "docs/research/2026-10-19-act1-content-red-colorless.md",
    "SILENT": "docs/research/2026-10-19-act1-content-silent.md",
    "DEFECT": "docs/research/2026-10-19-act1-content-defect.md",
    "WATCHER": "docs/research/2026-10-19-act1-content-watcher.md",
    "ENEMIES": "docs/research/2026-10-19-act1-content-enemies.md",
    "POTIONS": "docs/research/2026-10-19-act1-content-economy-potions.md",
    "TRANSITIVE": "docs/research/2026-10-19-act1-transitive-execution-closure.md",
}
WIKI = "https://slaythespire.wiki.gg/wiki/"
BOTH_OWNERS = {"Artifact", "Barricade", "Metallicize", "Plated Armor", "Ritual", "Strength", "Thorns", "Vulnerable", "Weak"}
ENEMY_OWNERS = {
    "Angry", "Back Attack", "Beat of Death", "Block Return", "Choked", "Corpse Explosion",
    "Curiosity", "Curl Up", "Enrage", "Explosive", "Fading", "Flying", "Invincible",
    "Life Link", "Lock-On", "Malleable", "Mark", "Minion", "Mode Shift", "Painful Stabs",
    "Poison", "Reactive", "Regenerate", "Shackled", "Sharp Hide", "Shifting", "Slow", "Split",
    "Spore Cloud", "Stasis", "Strength Up", "Thievery", "Time Warp", "Unawakened",
}


def load(path):
    return json.loads(path.read_text(encoding="utf-8"))


def plain_text(raw):
    text = re.sub(r"\[\[File:[^\]|]*Energy\.png[^\]]*\]\]", " [E] ", raw, flags=re.I)
    text = re.sub(r"\[\[(?:File|Image):[^\]]*\]\]", "", text, flags=re.I)
    text = re.sub(r"<br\s*/?>", "\n", text, flags=re.I)
    text = re.sub(r"<[^>]*>", "", text)
    text = re.sub(r"\[\[([^\[\]]+)\]\]", lambda m: m[1].split("|")[-1], text)
    text = re.sub(r"\[(https?://[^\s\]]+)(?:\s+([^\]]+))?\]", lambda m: m[2] or m[1], text)
    text = html.unescape(text).replace("'''", "").replace("''", "")
    text = "\n".join(re.sub(r"[ \t]+", " ", line).strip() for line in text.splitlines())
    text = re.sub(r"\s+([.,])", r"\1", text).strip()
    if re.search(r"\[\[|\[https?://|\{\{|<[^>]+>", text):
        raise ValueError(f"Unhandled wiki markup: {text!r}")
    return text


def slug(name):
    return re.sub(r"[^a-z0-9]+", "_", name.lower()).strip("_")


def stack_mode(label):
    return {
        "Intensity": "intensity", "Duration": "duration", "Counter": "charges",
        "Intensity and Duration": "intensity_and_duration",
        "Intensity and Counter": "stateful", "Does not stack": "nonstacking",
        "Doesn't stack": "nonstacking",
    }.get(label, "unknown")


def description_triggers(text):
    # These are deliberately NOT source callback names or a complete trigger inventory.
    result = []
    if re.search(r"(?:start|beginning) of (?:your|its|the) turn", text, re.I):
        result.append("DESCRIPTION_OWNER_START_TURN")
    if re.search(r"end of (?:your|its|this|every|the|X) turn", text, re.I):
        result.append("DESCRIPTION_OWNER_END_TURN")
    if re.search(r"whenever you play|next.*played|when you play", text, re.I):
        result.append("DESCRIPTION_CARD_PLAY_RELATED")
    if re.search(r"attacked|receiving attack|when hit|on the attacker", text, re.I):
        result.append("DESCRIPTION_ATTACK_RELATED")
    if re.search(r"on death", text, re.I):
        result.append("DESCRIPTION_OWNER_DEATH")
    return result or ["PASSIVE_OR_TRIGGER_NOT_SPECIFIED"]


def citation(page_name, pages):
    page = pages[page_name]
    return {
        "kind": "wiki", "page": page_name, "revision_id": page["revision_id"],
        "revision_timestamp": page["revision_timestamp"],
        "url": WIKI + urllib.parse.quote(page_name.replace(" ", "_"), safe="_:'()."),
        "revision_url": WIKI + "Special:PermanentLink/" + str(page["revision_id"]),
        "grade": "community documentation; not version-pinned runtime evidence",
    }


def snapshot(path):
    return {"path": str(path.relative_to(ROOT)), "sha256": hashlib.sha256(path.read_bytes()).hexdigest()}


def make_power(key, row, catalog, annotations, pages):
    note = annotations.get(key, {})
    wiki_type = row["Type"].upper()
    reference_type = note.get("type", wiki_type)
    game_id = catalog["game_id"] if catalog else note.get("game_id")
    names = row["Name"]
    owners = ["PLAYER", "ENEMY"] if names in BOTH_OWNERS else ["ENEMY"] if names in ENEMY_OWNERS else ["PLAYER"]
    if key == "power.reference.intangible_enemy":
        owners = ["ENEMY"]
    effect = plain_text(row["Description"])
    mode = note.get("stacking", stack_mode(row["Stacks"]))
    amount_defaults = {
        "intensity": "Effect magnitude X; see the full effect and any additional state rather than assuming every scalar equals amount.",
        "duration": "Duration/countdown; the exact owner, first-application flag and reduction checkpoint are effect-specific.",
        "charges": "Remaining qualifying uses/applications, not turns or repeated multiplication of one effect.",
        "nonstacking": "Presence/marker, not a linear magnitude; exact sentinel and duplicate-application behavior need qualification.",
        "intensity_and_duration": "Current payload and its diminishing future ticks are related; not a fixed payload for a separate duration.",
        "stateful": "More than one conceptual field may be required; the source summary is not a complete state layout.",
        "independent_instances": "Independent per-installation payload/state; do not merge solely by display name.",
        "threshold_counter": "Progress toward an effect-specific threshold, not an unconditional damage intensity.",
        "unknown": "Not established by the retrieved source; do not substitute zero or an unlimited duration.",
    }
    progression = {
        "Permanent": "The wiki calls this ongoing within combat; explicit removal/cleansing and effect-specific hooks remain separate.",
        "Removed": "The wiki indicates removal between turns but does not by itself establish the precise owner/phase.",
        "Conserved": "The wiki labels this Conserved; that label alone does not qualify cross-turn persistence or the exact expiry/consumption hook.",
        "Decremented": "The wiki indicates decrementing; its precise hook/first-application behavior needs the named power's qualification.",
        "Reset": "The wiki indicates a reset; configured reset value and remaining current value are distinct.",
        "Counter": "Ambiguous wiki metadata; not automatically interpreted as a per-turn decrement.",
    }.get(row["BetweenTurns"], "Not specified by the primary summary; see the supporting page and qualified notes.")
    citations = [citation(row["_pageName"], pages)]
    if catalog:
        citations.append({"kind": "local_document", "path": CATALOG, "section": "2.9",
                          "grade": "historical catalog; identity/direct-owner evidence only, not mechanic acceptance"})
    for ref in note.get("research", []):
        family, section = ref.split(":", 1)
        citations.append({"kind": "local_document", "path": RESEARCH[family], "section": section,
                          "grade": "retained source-grounded draft/local finding; unresolved composition fields not promoted"})
    for page in note.get("extra_wiki", []):
        if page != row["_pageName"]:
            citations.append(citation(page, pages))
    uncertainties = list(note.get("uncertainties", []))
    if game_id is None:
        uncertainties.append("Exact constructor ID is not established by the retained public catalog/research; null is intentional, not an inferred display-name ID.")
    if reference_type == "UNKNOWN":
        uncertainties.append("Technical BUFF/DEBUFF classification is unresolved; practical harm does not decide it.")
    if not row["BetweenTurns"] and "turn_progression" not in note and "removal" not in note:
        uncertainties.append("The primary table does not specify exact expiry/reset/removal timing.")
    if row["Name"] == "Mantra":
        uncertainties.append("The Cargo Cap field is truncated mid-markup. It is preserved raw, not used as a numeric cap or complete rule.")
    return {
        "key": key, "id": key.removeprefix("power.").upper(), "game_id": game_id,
        "class_alias": catalog["class_alias"] if catalog else note.get("class_alias"),
        "reference_extension": catalog is None,
        "identity_grade": "catalog_documented" if catalog else "documented_in_local_research" if game_id else "unresolved_constructor_id",
        "name": names + (" (player)" if key == "power.intangible_player_power" else " (enemy)" if key == "power.reference.intangible_enemy" else ""),
        "wiki_name": names, "owners": owners,
        "owner_scope": "Normal unmodded acquisition/encounter recipients, not arbitrary debug/mod application compatibility",
        "classification": {
            "reference_type": reference_type,
            "effect_on_owner": note.get("benefit", "BENEFICIAL" if wiki_type == "BUFF" else "HARMFUL"),
            "basis": "qualified annotation; consult citations/discrepancies" if "type" in note else "community wiki classification, not independently constructor-verified",
        },
        "effect": note.get("effect", effect),
        "stacking": {"mode": mode, "amount_meaning": note.get("amount", amount_defaults[mode]),
                     "state": note.get("state", ["presence"] if mode == "nonstacking" else ["amount; additional fields not enumerated by this summary"]),
                     "caps": note.get("caps")},
        "lifecycle": {
            "triggers": note.get("triggers", description_triggers(effect)),
            "trigger_basis": "annotated finding at its stated grade" if "triggers" in note else "primary-description cues only, not source callback names",
            "turn_progression": note.get("turn_progression", progression),
            "round_decay": note.get("round_decay"),
            "removal": note.get("removal", "Effect-specific consumption, cleansing, explicit removal and combat settlement require their own qualified path; no universal removal rule inferred."),
            "reset": "Combat-scoped state. Resolve qualified victory/death effects before disposing of combat state; queueing is not proof of completion.",
        },
        "source_owner_mentions": catalog["source_owners"] if catalog else None,
        "wiki": {"type": row["Type"], "stacking": row["Stacks"], "between_turns": row["BetweenTurns"],
                 "cap_raw": row["Cap"], "effect": effect},
        "notes": note.get("notes", []), "discrepancies": note.get("discrepancies", []),
        "uncertainties": uncertainties, "citations": citations,
        "evidence": {"original_runtime_validated": False,
                     "scope": "PC v2.3 reference target, not binary/version/whole-game mechanical acceptance"},
    }


def build_dataset():
    rows = load(HERE / "sources/wiki-powers.json")
    if any(row["Game"] != "1" for row in rows):
        raise ValueError("Non-StS1 entry in power source")
    by_name = {row["Name"]: row for row in rows}
    if len(by_name) != len(rows):
        raise ValueError("Duplicate wiki names need explicit owner/variant disambiguation")
    pages = {p["title"]: p for p in load(HERE / "sources/wiki-pages.json")}
    catalog = load(HERE / "sources/catalog-powers.json")
    aliases = load(HERE / "sources/wiki-name-aliases.json")
    annotations = load(HERE / "annotations.json")
    retrieval = load(HERE / "sources/wiki-retrieval.json")
    powers = []
    used_names = set()
    for item in catalog:
        name = aliases.get(item["game_id"], item["game_id"])
        used_names.add(name)
        powers.append(make_power(item["key"], by_name[name], item, annotations, pages))
    for name in sorted(set(by_name) - used_names - {"Orb Slot"}):
        powers.append(make_power("power.reference." + slug(name), by_name[name], None, annotations, pages))
    powers.append(make_power("power.reference.intangible_enemy", by_name["Intangible"], None, annotations, pages))
    # Module-only supplement: the enemy page distinguishes this indicator from Back Attack.
    surrounded = {"Name": "Surrounded", "_pageName": "Module:Powers/data", "Type": "Debuff",
                  "Description": "Receive 50% more damage if attacked from behind. Use targeting cards or potions to change your orientation.",
                  "Stacks": "Does not stack", "BetweenTurns": "", "Cap": ""}
    powers.append(make_power("power.reference.surrounded", surrounded, None, annotations, pages))
    powers.sort(key=lambda power: power["key"])
    keys = {p["key"] for p in powers}
    if len(keys) != len(powers) or set(annotations) - keys:
        raise ValueError("Duplicate keys or orphaned annotations")
    local_paths = {c["path"] for p in powers for c in p["citations"] if c["kind"] == "local_document"}
    return {
        "schema_version": 1, "game": "Slay the Spire", "edition": "unmodded PC", "target_version": "2.3",
        "retrieved_at": retrieval["retrieved_at"], "randomness_policy": "RNG-001",
        "purpose": "Player/enemy buff and debuff implementation reference; not a runtime registry or implemented-mechanics declaration",
        "coverage": {"records": len(powers), "existing_catalog_keys": len(catalog), "wiki_rows": len(rows),
                     "wiki_pages": len(pages), "reference_extensions": sum(p["reference_extension"] for p in powers),
                     "unresolved_constructor_ids": sum(p["game_id"] is None for p in powers),
                     "owner_groups": dict(collections.Counter("+".join(p["owners"]) for p in powers))},
        "limitations": [
            "Complete against the captured StS1 wiki roster (except explicit non-power exclusions), the retained 106-key catalog, and named supplements; not an independently exhaustive original class/unused/debug inventory.",
            "PC v2.3 is the target. Community wiki revisions and historical static findings do not establish binary/version provenance or original-runtime validation.",
            "New reference keys are not accepted native catalog registrations. Unknown constructor IDs/class aliases stay null; display names are not used to guess them.",
            "Description-derived trigger labels are not exact source callbacks. Unknown phase/order, first-application flags, owner lifecycle, terminal pruning and modifier composition remain explicit obligations.",
            "Technical type and benefit differ. Positive/negative stat changes, beneficial debuffs and harmful buffs require their qualified application/cleansing paths.",
            "Wiki stacking/BetweenTurns/Cap labels are descriptive and sometimes wrong or truncated. caps:null means unestablished, not unbounded. No blanket 999 cap is imposed.",
            "Existing local findings retain their historical Ironclad A0 Act1/off-color scope and unclosed fields; citations do not promote whole-character/all-act acceptance.",
            "TheBomb's documented literal is a family identifier; separate per-cast state/live IDs are not required to equal one unsuffixed string. Night Terror retains independent templates.",
            "RNG-001 requires applicable conditional laws and deliberate dependencies, not Java PRNG/seed/cursor/incidental coupling parity. No simulator, observation API, dependencies or private source artifacts are changed.",
        ],
        "shared_rules": [
            {"rule": "Queued ApplyPowerAction and direct addPower are different paths. Source callbacks, target validity, named relic prevention, Artifact, matching-ID stacking and new insertion have distinct checkpoints.", "source": RESEARCH["TRANSITIVE"] + " section 3"},
            {"rule": "No single global callback order: power priority ties/list order, source/target/player/enemy dispatch and queued work are site-specific. Do not infer a universal all-powers-before-all-relics pipeline.", "source": RESEARCH["TRANSITIVE"] + " section 3"},
            {"rule": "Actual draw, card use, after-card-played, positive pre-loss damage callbacks, final HP loss, card Block and raw power Block are different events.", "source": RESEARCH["SILENT"] + " section 7; " + RESEARCH["DEFECT"] + " section 5.1"},
        ],
        "exclusions": [
            {"name": "Orb Slot", "reason": "Capacity/slot state, not a buff/debuff power identity; kept in raw wiki roster for auditable exclusion."},
            {"name": "Stances, Orbs, and Status/Curse cards", "reason": "Different state/content domains; effects involving them are cross-referenced, not reclassified as powers."},
        ],
        "sources": [
            {"id": "wiki", "title": "Slay the Spire Wiki contributors: StS1 Powers table and supporting pages",
             "query_url": retrieval["query_url"], "license": "CC-BY-SA-4.0",
             "license_url": "https://creativecommons.org/licenses/by-sa/4.0/",
             "snapshots": [snapshot(path) for path in sorted((HERE / "sources").glob("wiki-*.json"))]},
            {"id": "local", "grade": "retained public source-grounded findings, not original-runtime validation",
             "snapshots": [snapshot(ROOT / path) for path in sorted(local_paths)] +
                          [snapshot(HERE / "sources/catalog-powers.json"), snapshot(HERE / "sources/wiki-name-aliases.json")]},
        ],
        "powers": powers,
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="Regenerate in memory and compare without writing")
    args = parser.parse_args()
    dataset = build_dataset()
    text = json.dumps(dataset, ensure_ascii=False, indent=2) + "\n"
    path = HERE / "powers.json"
    if args.check:
        if path.read_text(encoding="utf-8") != text:
            raise SystemExit("Power reference differs from its inputs; regenerate with data/powers/build_dataset.py")
        print(f"Power reference regenerates identically: {len(dataset['powers'])} records")
    else:
        path.write_text(text, encoding="utf-8")
        print(json.dumps(dataset["coverage"], indent=2))


if __name__ == "__main__":
    main()
