"""Offline power-reference checks; not simulation or original-runtime proof."""

import hashlib
import importlib.util
import json
from pathlib import Path
import re
import unittest


ROOT = Path(__file__).resolve().parents[2]
DATA = ROOT / "data/powers"
CATALOG = ROOT / "docs/research/2026-10-19-act1-complete-catalog-numeric.md"


class PowerReferenceTests(unittest.TestCase):
    def setUp(self):
        self.assertTrue((DATA / "powers.json").is_file(), "Missing player/enemy power reference")
        self.dataset = json.loads((DATA / "powers.json").read_text())
        self.powers = {power["key"]: power for power in self.dataset["powers"]}

    def test_preserves_all_existing_catalog_keys_and_exact_ids(self):
        rows = re.findall(r"^\| `(power\.[^`]+)` \| `([^`]+)` \| `([^`]+)` \|", CATALOG.read_text(), re.M)
        self.assertEqual(len(rows), 106)
        for key, game_id, alias in rows:
            with self.subTest(key=key):
                self.assertIn(key, self.powers)
                self.assertEqual(self.powers[key]["game_id"], game_id)
                self.assertEqual(self.powers[key]["class_alias"], alias)
                self.assertFalse(self.powers[key]["reference_extension"])

    def test_covers_wiki_roster_without_turning_orb_slots_into_a_power(self):
        rows = json.loads((DATA / "sources/wiki-powers.json").read_text())
        represented = {p["wiki_name"] for p in self.powers.values()}
        self.assertEqual({r["Name"] for r in rows} - represented, {"Orb Slot"})
        self.assertIn("Surrounded", represented)
        self.assertEqual(len(self.dataset["powers"]), len(self.powers))
        self.assertEqual(len({p["id"] for p in self.powers.values()}), len(self.powers))
        self.assertEqual(len(self.powers), 133)
        documented_ids = [p["game_id"] for p in self.powers.values() if p["game_id"] is not None]
        self.assertEqual(len(documented_ids), len(set(documented_ids)))
        self.assertTrue(any(e["name"] == "Orb Slot" for e in self.dataset["exclusions"]))

    def test_unknown_constructor_ids_are_explicit_not_inferred_from_names(self):
        for power in self.powers.values():
            if power["game_id"] is None:
                self.assertTrue(power["reference_extension"])
                self.assertEqual(power["identity_grade"], "unresolved_constructor_id")
                self.assertTrue(any("constructor ID" in text for text in power["uncertainties"]))
        self.assertIsNone(self.powers["power.reference.beat_of_death"]["game_id"])
        self.assertEqual(self.powers["power.weak_power"]["game_id"], "Weakened")
        self.assertEqual(self.powers["power.watcher_mark_power"]["game_id"], "PathToVictoryPower")

    def test_owners_are_recipients_not_the_sources_of_application(self):
        self.assertEqual(self.powers["power.reference.hex"]["owners"], ["PLAYER"])
        self.assertEqual(self.powers["power.reference.constricted"]["owners"], ["PLAYER"])
        self.assertEqual(self.powers["power.poison_power"]["owners"], ["ENEMY"])
        self.assertEqual(self.powers["power.strength_power"]["owners"], ["PLAYER", "ENEMY"])
        self.assertEqual(self.powers["power.reference.back_attack"]["owners"], ["ENEMY"])
        self.assertEqual(self.powers["power.reference.surrounded"]["owners"], ["PLAYER"])

    def test_classification_is_not_equated_with_benefit_or_wiki_taxonomy(self):
        death = self.powers["power.watcher_end_turn_death_power"]
        self.assertEqual(death["classification"]["reference_type"], "BUFF")
        self.assertEqual(death["classification"]["effect_on_owner"], "HARMFUL")
        self.assertEqual(death["wiki"]["type"], "Debuff")
        self.assertTrue(death["discrepancies"])
        shackled = self.powers["power.gain_strength_power"]
        self.assertEqual(shackled["classification"]["reference_type"], "DEBUFF")
        self.assertEqual(shackled["classification"]["effect_on_owner"], "BENEFICIAL")
        block_return = self.powers["power.watcher_block_return_power"]
        self.assertEqual(block_return["classification"]["reference_type"], "DEBUFF")
        self.assertEqual(block_return["wiki"]["type"], "Buff")
        self.assertEqual(self.powers["power.strength_power"]["classification"]["reference_type"], "SIGN_DEPENDENT")

    def test_player_enemy_variants_and_independent_payloads_do_not_collapse(self):
        player = self.powers["power.intangible_player_power"]
        enemy = self.powers["power.reference.intangible_enemy"]
        self.assertEqual(player["owners"], ["PLAYER"])
        self.assertEqual(enemy["owners"], ["ENEMY"])
        self.assertNotEqual(player["key"], enemy["key"])
        self.assertEqual(self.powers["power.regen_power"]["stacking"]["mode"], "intensity_and_duration")
        self.assertEqual(self.powers["power.reference.regenerate"]["stacking"]["mode"], "intensity")
        self.assertEqual(self.powers["power.reference.nightmare"]["game_id"], "Night Terror")
        for key in ("power.reference.nightmare", "power.the_bomb_power"):
            self.assertEqual(self.powers[key]["stacking"]["mode"], "independent_instances")
            self.assertTrue(self.powers[key]["stacking"]["state"])

    def test_trigger_distinctions_survive_normalization(self):
        self.assertIn("OWNER_START_TURN", self.powers["power.poison_power"]["lifecycle"]["triggers"])
        self.assertIn("PLAYER_POST_DRAW", self.powers["power.noxious_fumes_power"]["lifecycle"]["triggers"])
        self.assertIn("OWNER_END_TURN", self.powers["power.wraith_form_power"]["lifecycle"]["triggers"])
        self.assertTrue(self.powers["power.wraith_form_power"]["discrepancies"])
        self.assertIn("ON_ATTACK_DAMAGE_CALLBACK", self.powers["power.thorns_power"]["lifecycle"]["triggers"])
        self.assertIn("ON_ATTACK_CARD_PLAY", self.powers["power.sharp_hide_power"]["lifecycle"]["triggers"])

    def test_duplication_retains_round_decay_finding_and_tooltip_conflict(self):
        power = self.powers["power.duplication_power"]
        self.assertTrue(power["discrepancies"])
        self.assertIn("round_decay", power["lifecycle"])
        self.assertEqual(power["lifecycle"]["round_decay"], {
            "phase": "END_ROUND", "reduction": 1, "remove_if_already_zero": True,
        })
        self.assertTrue(any(c.get("path", "").endswith("act1-content-economy-potions.md") for c in power["citations"]))

    def test_time_warp_threshold_is_cited_to_the_page_that_states_it(self):
        power = self.powers["power.reference.time_warp"]
        self.assertIn("Buffs", {c.get("page") for c in power["citations"]})

    def test_all_records_have_usable_information_and_versioned_sources(self):
        pages = {p["title"]: p for p in json.loads((DATA / "sources/wiki-pages.json").read_text())}
        for power in self.powers.values():
            with self.subTest(power=power["key"]):
                self.assertTrue(power["effect"])
                self.assertNotRegex(power["effect"], r"\[\[|\[https?://|\{\{|<[^>]+>")
                self.assertTrue(power["owners"])
                self.assertTrue(power["stacking"]["amount_meaning"])
                self.assertTrue(power["lifecycle"]["removal"])
                self.assertTrue(power["citations"])
                self.assertFalse(power["evidence"]["original_runtime_validated"])
                for citation in power["citations"]:
                    if citation["kind"] == "wiki":
                        self.assertEqual(citation["revision_id"], pages[citation["page"]]["revision_id"])
                    else:
                        self.assertTrue((ROOT / citation["path"]).is_file())
        self.assertEqual(self.dataset["target_version"], "2.3")
        self.assertEqual(self.dataset["randomness_policy"], "RNG-001")
        for source in self.dataset["sources"]:
            for file in source.get("snapshots", []):
                self.assertEqual(hashlib.sha256((ROOT / file["path"]).read_bytes()).hexdigest(), file["sha256"])


class PowerReferenceBuilderTests(unittest.TestCase):
    def setUp(self):
        script = DATA / "build_dataset.py"
        self.assertTrue(script.is_file(), "Missing power-reference normalizer")
        spec = importlib.util.spec_from_file_location("power_reference_builder", script)
        self.builder = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(self.builder)

    def test_renders_links_and_preserves_nonupgrade_parentheses_and_energy(self):
        text = 'Gain [[File:GreenEnergy.png|20px]] (X+1) [https://example.org Energy] and [[Strength|Strength]].'
        self.assertEqual(self.builder.plain_text(text), "Gain [E] (X+1) Energy and Strength.")

    def test_unhandled_templates_are_not_silently_published(self):
        with self.assertRaises(ValueError):
            self.builder.plain_text("Gain {{unhandled|Strength}}")


if __name__ == "__main__":
    unittest.main()
