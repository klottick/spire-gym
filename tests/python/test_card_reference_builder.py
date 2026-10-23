"""Focused checks on normalization, not a general mechanics verifier."""

import importlib.util
from pathlib import Path
import unittest


SCRIPT = Path(__file__).resolve().parents[2] / "data/cards/build_dataset.py"


class CardReferenceBuilderTests(unittest.TestCase):
    def setUp(self):
        self.assertTrue(SCRIPT.is_file(), "Missing card dataset builder")
        spec = importlib.util.spec_from_file_location("card_reference_builder", SCRIPT)
        self.builder = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(self.builder)

    def test_preserves_energy_icons_and_empty_upgrade_branches(self):
        text = '(;Innate.<br>)Gain ([[File:GreenEnergy.png|20px|link=]];[[File:GreenEnergy.png|20px|link=]] [[File:GreenEnergy.png|20px|link=]]).'
        self.assertEqual(self.builder.render_description(text, False), "Gain [E].")
        self.assertEqual(self.builder.render_description(text, True), "Innate.\nGain [E] [E].")
        self.assertEqual(self.builder.render_description('Draw (3;4) cards.( Exhaust.;)', True), "Draw 4 cards.")

    def test_removes_decorative_icons_without_duplicating_keyword_text(self):
        text = 'Gain 5 <span class="keyword-trigger">[[File:Icon Block.png|18px|link=Block]] [[Block|<span title="">Block</span>]]</span>.'
        self.assertEqual(self.builder.render_description(text, False), "Gain 5 Block.")

    def test_rejects_unhandled_markup(self):
        with self.assertRaises(ValueError):
            self.builder.render_description("Gain {{unknown|Block}}", False)

    def test_external_links_render_as_labels_without_changing_energy_tokens(self):
        text = 'If the last card was a [https://slaythespire.wiki.gg/wiki/Cards_List?type=Skill Skill], gain [E].'
        self.assertEqual(self.builder.render_description(text, False), "If the last card was a Skill, gain [E].")
        self.assertEqual(self.builder.render_description("See [https://example.com].", False), "See https://example.com.")

    def test_targeting_distinguishes_mixed_and_random_payloads(self):
        target = self.builder.requires_enemy_target
        self.assertFalse(target("BOUNCING_FLASK", "SKILL", "Apply 3 Poison to a random enemy 3 times.", False))
        self.assertFalse(target("RAGNAROK", "ATTACK", "Deal 5 damage to a random enemy 5 times.", False))
        self.assertFalse(target("CLEAVE", "ATTACK", "Deal 8 damage to ALL enemies.", False))
        self.assertTrue(target("CATALYST", "SKILL", "Double an enemy's Poison. Exhaust.", False))
        self.assertTrue(target("LEG_SWEEP", "SKILL", "Apply 2 Weak. Gain 11 Block.", False))
        self.assertTrue(target("BLIND", "SKILL", "Apply 2 Weak.", False))
        self.assertFalse(target("BLIND", "SKILL", "Apply 2 Weak to ALL enemies.", True))


if __name__ == "__main__":
    unittest.main()
