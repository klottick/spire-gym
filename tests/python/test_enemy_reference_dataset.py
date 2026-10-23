"""Offline integrity checks for authored reference data, not a gameplay oracle."""

from fractions import Fraction
import json
from pathlib import Path
import re
import unittest


ROOT = Path(__file__).resolve().parents[2]
DATA = ROOT / "data" / "enemies"
ROSTER = set("""
ACID_SLIME_L ACID_SLIME_M ACID_SLIME_S AWAKENED_ONE BEAR BLUE_SLAVER
BOOK_OF_STABBING BRONZE_AUTOMATON BRONZE_ORB BYRD CENTURION CHOSEN CORRUPT_HEART
CULTIST DAGGER DARKLING DECA DONU EXPLODER FAT_GREMLIN FUNGI_BEAST GIANT_HEAD
GREEN_LOUSE GREMLIN_LEADER GREMLIN_NOB GREMLIN_WIZARD HEXAGHOST JAW_WORM
LAGAVULIN LOOTER MAD_GREMLIN MUGGER MYSTIC NEMESIS ORB_WALKER POINTY RED_LOUSE
RED_SLAVER REPTOMANCER REPULSOR ROMEO SENTRY SHELLED_PARASITE SHIELD_GREMLIN
SLIME_BOSS SNAKE_PLANT SNEAKY_GREMLIN SNECKO SPHERIC_GUARDIAN SPIKER SPIKE_SLIME_L
SPIKE_SLIME_M SPIKE_SLIME_S SPIRE_GROWTH SPIRE_SHIELD SPIRE_SPEAR TASKMASTER
THE_CHAMP THE_COLLECTOR THE_GUARDIAN THE_MAW TIME_EATER TORCH_HEAD TRANSIENT
WRITHING_MASS
""".split())


class EnemyReferenceTests(unittest.TestCase):
    def setUp(self):
        self.assertTrue((DATA / "enemies.json").is_file(), "Missing enemy reference")
        self.data = json.loads((DATA / "enemies.json").read_text())
        self.records = self.data["enemies"]
        self.by_id = {record["id"]: record for record in self.records}
        self.sources = json.loads((DATA / "sources.json").read_text())["sources"]
        self.encounters = json.loads((DATA / "encounters.json").read_text())

    def test_target_and_evidence_boundary(self):
        self.assertEqual(self.data["game"]["version"], "2.3")
        self.assertEqual(self.data["game"]["platform"], "PC")
        self.assertEqual(self.data["game"]["ascensions"], [0, 20])
        self.assertEqual(self.data["verification_level"], "reference_not_runtime_verified")
        self.assertEqual(self.data["project_policy"]["rng"], "RNG-001")

    def test_complete_unique_roster(self):
        self.assertEqual(set(self.by_id), ROSTER)
        self.assertEqual(len(self.records), 65)
        self.assertEqual(len({r["key"] for r in self.records}), 65)
        self.assertEqual(len({r["game_id"] for r in self.records}), 65)
        self.assertNotIn("APOLOGY_SLIME", self.by_id)

    def test_hp_bands_cover_every_ascension_without_overlap(self):
        for record in self.records:
            levels = []
            for band in record["hp"]["bands"]:
                self.assertLessEqual(band["min"], band["max"], record["id"])
                self.assertGreater(band["min"], 0, record["id"])
                levels.extend(range(band["ascension_min"], band["ascension_max"] + 1))
            self.assertEqual(sorted(levels), list(range(21)), record["id"])

    def test_every_enemy_has_moves_ai_and_lifecycle_information(self):
        for record in self.records:
            with self.subTest(enemy=record["id"]):
                self.assertTrue(record["name"])
                self.assertTrue(record["moves"])
                self.assertEqual(len(record["moves"]), len({m["id"] for m in record["moves"]}))
                self.assertTrue(record["ai"]["opening"])
                self.assertTrue(record["ai"]["history_and_state_rules"])
                self.assertTrue(record["lifecycle"])
                for move in record["moves"]:
                    self.assertTrue(move["effect"])
                    self.assertIn(move["damage"]["kind"], {"none", "fixed", "range", "expression"})
                    self.assertTrue(move["target"])

    def test_nominal_chances_are_normalized_and_not_claimed_as_iid(self):
        for record in self.records:
            self.assertFalse(record["ai"]["iid_turns"])
            for group in record["ai"]["nominal_choices"]:
                self.assertTrue(group["context"])
                self.assertIn(group["precision"], {"documented_exact", "wiki_nominal", "wiki_approximate"})
                self.assertEqual(sum((Fraction(c["chance"]) for c in group["choices"]), Fraction()), 1)
                for choice in group["choices"]:
                    self.assertGreater(Fraction(choice["chance"]), 0)
                    self.assertIn(choice["move"], {m["id"] for m in record["moves"]})

    def test_existing_monster_keys_and_ids_are_preserved(self):
        text = (ROOT / "docs/research/2026-10-19-act1-complete-catalog-numeric.md").read_text()
        expected = re.findall(r"^\| `(monster\.[^`]+)` \| `([^`]+)`", text, re.M)
        self.assertEqual(len(expected), 25)
        by_game = {r["game_id"]: r for r in self.records}
        for key, game_id in expected:
            self.assertEqual(by_game[game_id]["key"], key)

    def test_citations_and_local_qualifications(self):
        for record in self.records:
            self.assertTrue(record["citations"])
            for source_id in record["citations"]:
                source = self.sources[source_id]
                self.assertTrue(source["supports"])
                if source["kind"] == "wiki":
                    self.assertGreater(source["revision_id"], 0)
                    self.assertIn("oldid=", source["url"])
                if source["kind"] == "local_research":
                    self.assertTrue((ROOT / source["local_path"]).is_file())
            if record["local_a0"] is not None:
                self.assertEqual(record["local_a0"]["scope"], "Ironclad A0 Act 1 local record; not all-ascension/runtime validation")
        self.assertEqual(sum(r["local_a0"] is not None for r in self.records), 25)

    def test_encounter_references_and_selection_are_separate_from_moves(self):
        self.assertTrue(self.encounters["selection_rules"])
        self.assertEqual({e["act"] for e in self.encounters["encounters"]}, {1, 2, 3, 4})
        self.assertEqual(len({e["id"] for e in self.encounters["encounters"]}), len(self.encounters["encounters"]))
        for encounter in self.encounters["encounters"]:
            self.assertTrue(encounter["formation"])
            self.assertLessEqual(set(encounter["enemy_ids"]), ROSTER)
            self.assertTrue(encounter["citations"])
            for source in encounter["citations"]:
                self.assertIn(source, self.sources)

    def test_known_classification_and_ascension_edges(self):
        for name in ("SPIKER", "TRANSIENT", "WRITHING_MASS"):
            self.assertEqual(self.by_id[name]["category"], "NORMAL")
        self.assertEqual(self.by_id["ROMEO"]["category"], "EVENT")
        self.assertEqual(self.by_id["BRONZE_ORB"]["category"], "MINION")
        self.assertEqual(self.by_id["CORRUPT_HEART"]["hp"]["bands"][-1]["min"], 800)
        self.assertEqual(self.by_id["BEAR"]["hp"]["bands"][0]["max"], 42)
        self.assertTrue(self.by_id["JAW_WORM"]["variants"])
        self.assertTrue(self.by_id["LAGAVULIN"]["variants"])
        self.assertTrue(self.by_id["DARKLING"]["variants"])

    def test_retained_exact_jaw_and_acid_redirects_are_not_rounded(self):
        jaw = self.by_id["JAW_WORM"]["ai"]
        self.assertIn("exact_a0_choices", jaw)
        post_chomp = jaw["exact_a0_choices"]["after_chomp"]
        self.assertEqual(Fraction(post_chomp["bellow"]), Fraction(189, 320))
        self.assertNotEqual(Fraction(post_chomp["bellow"]), Fraction(59, 100))
        for name in ("ACID_SLIME_L", "ACID_SLIME_M"):
            ai = self.by_id[name]["ai"]
            self.assertEqual(ai["exact_a0_redirects"]["qA"], "6710887/16777216")
            self.assertEqual(ai["exact_a0_redirects"]["scope"], "A0 retained local finding only")

    def test_minion_hp_follows_parent_family_and_does_not_scale_as_normal(self):
        for name in ("BRONZE_ORB", "TORCH_HEAD"):
            self.assertEqual(self.by_id[name]["hp"]["bands"][1]["ascension_min"], 9)
        self.assertEqual(self.by_id["SENTRY"]["hp"]["bands"][1]["ascension_min"], 8)
        self.assertEqual(self.by_id["BLUE_SLAVER"]["hp"]["bands"][1]["ascension_min"], 7)

    def test_damage_is_per_hit_and_status_destinations_are_not_deck_edits(self):
        def move(enemy, name):
            return next(m for m in self.by_id[enemy]["moves"] if m["id"] == name)
        shots = move("CORRUPT_HEART", "blood_shots")["damage"]["bands"]
        self.assertEqual([(b["ascension_min"], b["amount"], b["hits"]) for b in shots], [(0, 2, 12), (4, 2, 15)])
        self.assertEqual(move("RED_LOUSE", "bite")["damage"]["kind"], "range")
        self.assertIn("discard", move("SENTRY", "bolt")["effect"])
        self.assertIn("draw", move("REPULSOR", "daze")["effect"])
        self.assertIn("permanent", move("WRITHING_MASS", "parasite")["target"])

    def test_disputed_breakpoints_are_not_silently_selected(self):
        for name in ("MYSTIC", "SNAKE_PLANT", "SPIRE_GROWTH", "TASKMASTER", "WRITHING_MASS"):
            self.assertTrue(self.by_id[name]["uncertainties"], name)
            self.assertIn("secondary:port", self.by_id[name]["citations"])

    def test_awakened_powers_describe_their_triggers_not_only_amounts(self):
        rules = " ".join(self.by_id["AWAKENED_ONE"]["lifecycle"])
        self.assertIn("Power card", rules)
        self.assertIn("Strength1", rules)
        self.assertIn("A19:2", rules)
        self.assertIn("end of its own turn", rules)
        self.assertIn("heal10", rules)
        self.assertIn("A19:15", rules)

    def test_exploder_fixed_explosion_does_not_assert_ordinary_attack_scaling(self):
        record = self.by_id["EXPLODER"]
        explosion = next(m for m in record["moves"] if m["id"] == "explode")
        self.assertNotIn("Strength and other live modifiers apply separately", explosion["damage"]["basis"])
        self.assertIn("modifier_scope", explosion["damage"])
        self.assertIn("secondary", explosion["damage"]["modifier_scope"].lower())
        self.assertTrue(any("modifier" in note for note in record["uncertainties"]))
        self.assertIn("secondary:port", record["citations"])

    def test_shield_smash_does_not_equate_block_with_player_hp_loss(self):
        record = self.by_id["SPIRE_SHIELD"]
        smash = next(m for m in record["moves"] if m["id"] == "smash")
        self.assertIn("not actual player HP lost", smash["effect"])
        self.assertIn("secondary", smash["effect"].lower())
        self.assertIn("A18", smash["effect"])
        self.assertIn("99", smash["effect"])
        self.assertIn("secondary:port", record["citations"])

    def test_reachable_acts_include_event_boss_split_dependents(self):
        for encounter in self.encounters["encounters"]:
            for enemy_id in encounter["enemy_ids"]:
                self.assertIn(encounter["act"], self.by_id[enemy_id]["acts"], (encounter["id"], enemy_id))

    def test_painful_stabs_keeps_discard_destination(self):
        self.assertTrue(any("Wound" in rule and "discard" in rule for rule in self.by_id["BOOK_OF_STABBING"]["lifecycle"]))


if __name__ == "__main__":
    unittest.main()
