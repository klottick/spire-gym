"""Offline integrity/normalization checks, not original-game mechanics proof."""

import collections
import hashlib
import json
from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
DATA = ROOT / "data/cards"


class CardReferenceDatasetTests(unittest.TestCase):
    def setUp(self):
        self.assertTrue((DATA / "cards.json").is_file(), "Missing all-character card reference")
        self.dataset = json.loads((DATA / "cards.json").read_text())
        self.cards = {card["id"]: card for card in self.dataset["cards"]}

    def test_complete_identity_coverage_without_collapsing_starters(self):
        self.assertEqual(len(self.dataset["cards"]), 370)
        self.assertEqual(len(self.cards), 370)
        self.assertNotIn("INVALID", self.cards)
        self.assertEqual(len({c["game_id"] for c in self.cards.values()}), 370)
        self.assertEqual(collections.Counter(c["character"] for c in self.cards.values()), {
            "IRONCLAD": 75, "SILENT": 75, "DEFECT": 75, "WATCHER": 75, None: 70,
        })
        for color, game_suffix in (("RED", "R"), ("GREEN", "G"), ("BLUE", "B"), ("PURPLE", "P")):
            for kind in ("STRIKE", "DEFEND"):
                self.assertEqual(self.cards[f"{kind}_{color}"]["game_id"], f"{kind.title()}_{game_suffix}")

    def test_every_wiki_row_has_exactly_one_record_and_revision_citation(self):
        rows = json.loads((DATA / "sources/wiki-cards.json").read_text())
        pages = {p["title"]: p for p in json.loads((DATA / "sources/wiki-pages.json").read_text())}
        self.assertEqual({c["source"]["wiki_name"] for c in self.cards.values()}, {r["Name"] for r in rows})
        for card in self.cards.values():
            with self.subTest(card=card["id"]):
                self.assertTrue(card["source"]["url"].startswith("https://slaythespire.wiki.gg/wiki/"))
                self.assertEqual(card["source"]["revision_id"], pages[card["source"]["page"]]["revision_id"])
                self.assertFalse(card["evidence"]["original_runtime_validated"])
                for reference in card["local_research"]:
                    self.assertTrue((ROOT / reference["path"]).is_file())
                    self.assertIn("draft", reference["grade"])
                for related in card["related_cards"]:
                    self.assertIn(related, self.cards)
                self.assertEqual(card["upgraded"] is None, card["upgrade"]["mode"] == "none")
                for variant in (card["base"], card["upgraded"]):
                    if variant is None:
                        continue
                    self.assertIsInstance(variant["cost"], int)
                    self.assertGreaterEqual(variant["cost"], -2)
                    self.assertTrue(variant["description"].strip())
                    self.assertNotRegex(variant["description"], r"\[\[|\[https?://|\{\{|<[^>]+>|\([^)]*;[^)]*\)")
                    self.assertIsInstance(variant["requires_enemy_target"], bool)

    def test_printed_pairs_cover_all_characters_and_preserve_energy(self):
        cases = {
            "BASH": (2, 2, "Deal 8 damage.", "Apply 3 Vulnerable."),
            "BLADE_DANCE": (1, 1, "Add 3 Shivs", "Add 4 Shivs"),
            "ADRENALINE": (0, 0, "Gain [E].", "Gain [E] [E]."),
            "AGGREGATE": (1, 1, "every 4 cards", "every 3 cards"),
            "FISSION": (0, 0, "Remove all your Orbs.", "Evoke all your Orbs."),
            "ERUPTION": (2, 1, "Deal 9 damage.", "Deal 9 damage."),
            "MIRACLE": (0, 0, "Gain [E].", "Gain [E] [E]."),
            "BLOODLETTING": (0, 0, "Gain [E] [E].", "Gain [E] [E] [E]."),
            "HEMOKINESIS": (1, 1, "Deal 15 damage.", "Deal 20 damage."),
        }
        for card_id, (cost, plus_cost, text, plus_text) in cases.items():
            with self.subTest(card=card_id):
                card = self.cards[card_id]
                self.assertEqual(card["base"]["cost"], cost)
                self.assertEqual(card["upgraded"]["cost"], plus_cost)
                self.assertIn(text, card["base"]["description"])
                self.assertIn(plus_text, card["upgraded"]["description"])

    def test_flags_targeting_and_printed_quantities_are_not_conflated(self):
        self.assertFalse(self.cards["TRUE_GRIT"]["base"]["flags"]["exhaust"])
        for card_id in ("WARCRY", "COLLECT", "DEUS_EX_MACHINA"):
            self.assertTrue(self.cards[card_id]["base"]["flags"]["exhaust"])
        self.assertTrue(self.cards["CALCULATED_GAMBLE"]["base"]["flags"]["exhaust"])
        self.assertFalse(self.cards["CALCULATED_GAMBLE"]["upgraded"]["flags"]["exhaust"])
        self.assertTrue(self.cards["APPARITION"]["base"]["flags"]["ethereal"])
        self.assertFalse(self.cards["APPARITION"]["upgraded"]["flags"]["ethereal"])
        self.assertFalse(self.cards["AFTER_IMAGE"]["base"]["flags"]["innate"])
        self.assertTrue(self.cards["AFTER_IMAGE"]["upgraded"]["flags"]["innate"])
        self.assertFalse(self.cards["WELL_LAID_PLANS"]["base"]["flags"]["retain"])
        self.assertTrue(self.cards["PROTECT"]["base"]["flags"]["retain"])
        self.assertTrue(self.cards["LEG_SWEEP"]["base"]["requires_enemy_target"])
        self.assertTrue(self.cards["PRESSURE_POINTS"]["base"]["requires_enemy_target"])
        self.assertTrue(self.cards["BLIND"]["base"]["requires_enemy_target"])
        self.assertFalse(self.cards["BLIND"]["upgraded"]["requires_enemy_target"])
        self.assertFalse(self.cards["BARRICADE"]["base"]["flags"]["exhaust"])
        self.assertEqual(self.cards["BRILLIANCE"]["color"], "PURPLE")
        self.assertEqual(self.cards["A_THOUSAND_CUTS"]["base"]["printed_damage"], 1)

    def test_special_upgrade_modes_and_outside_standard_mode_content(self):
        self.assertEqual(self.cards["BURN"]["upgrade"]["mode"], "enemy_only")
        self.assertIn("take 4 damage", self.cards["BURN"]["upgraded"]["description"])
        self.assertIsNone(self.cards["DAZED"]["upgraded"])
        self.assertEqual(self.cards["SEARING_BLOW"]["upgrade"]["damage_formula"], "12 + n * (n + 7) / 2")
        for card_id in ("BECOME_ALMIGHTY", "LIVE_FOREVER", "FAME_AND_FORTUNE"):
            card = self.cards[card_id]
            self.assertEqual(card["upgrade"]["mode"], "parent_choice")
            self.assertEqual(card["base"]["cost"], -2)
            self.assertTrue(card["implementation_notes"])
            self.assertEqual(card["availability"]["kind"], "choice_only")
        self.assertEqual(self.cards["EXPUNGER"]["base"]["cost"], 1)
        self.assertIn("X times", self.cards["EXPUNGER"]["base"]["description"])
        self.assertEqual(self.cards["WHIRLWIND"]["base"]["cost"], -1)
        self.assertFalse(self.cards["PRIDE"]["availability"]["standard_run"])
        self.assertEqual(self.cards["ASCENDERS_BANE"]["availability"]["minimum_ascension"], 10)

    def test_preserves_research_qualification_and_source_hashes(self):
        self.assertEqual(self.dataset["target_version"], "2.3")
        self.assertEqual(self.dataset["edition"], "unmodded PC")
        self.assertTrue(self.dataset["limitations"])
        self.assertTrue(self.dataset["sources"])
        for source in self.dataset["sources"]:
            for snapshot in source.get("snapshots", []):
                self.assertEqual(hashlib.sha256((ROOT / snapshot["path"]).read_bytes()).hexdigest(), snapshot["sha256"])

    def test_keeps_printed_order_distinct_from_draft_execution_findings(self):
        # Text order is not evidence that the resolver executes in that order.
        for card_id in ("BRUTALITY", "FTL"):
            card = self.cards[card_id]
            self.assertIn("discrepancies", card)
            self.assertTrue(card["discrepancies"])
            self.assertEqual(card["discrepancies"][0]["field"], "printed_order_vs_execution_order")
            self.assertTrue(card["research_notes"])
        self.assertTrue(self.cards["HALT"]["research_notes"])


if __name__ == "__main__":
    unittest.main()
