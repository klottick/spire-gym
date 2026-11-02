"""spire-90f card metadata facade: renderer precedence and native descriptor access."""

from spire_lab import card_metadata

FIXTURE = {
    "schema_id": card_metadata.SCHEMA_ID,
    "cards": [
        {"id": "strike", "color": "red", "rarity": "basic", "type": "attack", "max_upgrades": 1, "faces": [
            {"upgrades": 0, "cost": 1, "target": "enemy", "keywords": []},
            {"upgrades": 1, "cost": 1, "target": "enemy", "keywords": []},
        ]},
        {"id": "searing_blow", "color": "red", "rarity": "uncommon", "type": "attack", "max_upgrades": None, "faces": [
            {"upgrades": 0, "cost": 2, "target": "enemy", "keywords": []},
            {"upgrades": 1, "cost": 2, "target": "enemy", "keywords": []},
        ]},
        {"id": "become_almighty", "color": "colorless", "rarity": "special", "type": "power", "max_upgrades": 0, "faces": []},
    ],
}


def test_renderer_uses_metadata_and_prefers_observed_public_values():
    assert card_metadata.render_card(FIXTURE, "strike") == (
        "strike: red basic attack, cost 1, targets an enemy, keywords none")
    assert "cost 0 (printed 1)" in card_metadata.render_card(FIXTURE, "strike", 1, observed_cost=0)
    assert card_metadata.render_card(FIXTURE, "searing_blow", 3).endswith("later upgrades not described statically")
    assert card_metadata.render_card(FIXTURE, "become_almighty").endswith("not a physical card")
    assert card_metadata.render_card(FIXTURE, "missing") == "missing: unknown card"


def test_native_descriptor_matches_the_rust_pin():
    # Requires an extension built from this source; see the delivery receipt.
    metadata = card_metadata.load()
    assert metadata["fingerprint"] == "3e922792b73e10ce2214d1e97a8bbcd46af1dad1320b85d41c2310c6b855071c"
    rows = card_metadata.by_id(metadata)
    assert len(rows) == 369 and rows["strike"]["faces"][0]["cost"] == 1
    assert "strike: red basic attack, cost 1" in card_metadata.render_card(metadata, "strike")
