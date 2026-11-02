# Shared power and intent vocabulary (spire-85d)

Every observation encoder publishes the same power and intent vocabulary. The
engine owns the registry; encoders only derive from it.

- `crates/spire-core/src/combat/public_ids.rs` holds `PUBLIC_POWERS` and
  `PUBLIC_INTENTS`. These list every power and intent the public combat view
  can publish, including the masked intent `hidden`. In debug builds, the
  `full_player` and `full_enemies` view builders (`combat/view.rs`) assert that
  each published name is registered. So any core, env or fuzz test that reaches
  an unregistered producer fails loudly.
- `crates/spire-env/src/public_vocab.rs` is the only spire-env owner. Its
  `install` overwrites the `power` and `intent` vocabularies in both descriptor
  builders, `run_schema::vocabularies` and `run_schema_m10::spec_from_vocabulary`.
  Its `power` and `intent` lookups are the only encoder lookups. Row 0 is
  padding; row `i + 1` is registry entry `i`.
- Per-milestone power and intent lists have been removed: the M9 and M10 tables,
  the M12–M25 unions, the policy-view closure lists, and the M20 player-power
  admission list. No encoder can now reject a registered power or intent.
- `public_vocab` tests walk the full registry through every encoder (M9, M10,
  policy Run-v3, combat policy view-v1/v2, M12–M25 Run, M16 Defect combat and M21
  Watcher combat). `native_fault_regressions.rs` replays the retained M10 Flex
  (`lose_strength`) and M22 burning-elite Gremlin Nob (`regeneration`) prefixes
  at the session level.

Limit (review F3): completeness is enforced dynamically. The debug view guard
fires only on paths a test, fuzz or debug run actually reaches; there is no
structural per-producer check. A release build that publishes an unregistered
name still fails loudly, with an encoder `Input("unregistered public power")`
or `Input("unregistered public intent")` error surfaced as a delivery fault.
It never aliases the name to padding or corrupts the observation. A debug-build
native fuzz pass, or a check per producer table, would make completeness
durable.

When a producer adds a power or intent, append it to the registry. Every
descriptor and fingerprint below changes with it.

## CHANGES — 2026-11-02 descriptor fingerprints

All power and intent vocabularies became the shared registry tables: 137 powers
and 184 intents, plus padding. Every Run and combat-view descriptor fingerprint
changed. Power and intent indices changed for every schema except the leading
M9 prefix. Retrain or re-derive any consumer that embeds these indices.

| Schema | Schema ID | Old fingerprint | New fingerprint |
| --- | --- | --- | --- |
| M9 Run-v1 | `spire-gym-run-observation-v1` | `0893bce86e4b3abf11aa384c501503a91ae37f688ec605ab40b814ad365cb999` | `270163f700c7ca19556898d14c22ca8d641b0ef6f5c757d80d0ca7983989981c` |
| M10 Run-v2 | `spire-gym-run-observation-v2` | `7664874a7a483d1f2e0439ea7b319d1a4be4b9d276d0cb0f908654feda9393db` | `dccc7624c45898bc33e3fa682b620fd78ce70fe558d6db7624a888a7ed068c21` |
| M10 policy Run-v3 | `spire-gym-run-observation-v3` | `30905db1b7a520482d3ea8aa2e63ddb7f78b7c081de89d87268151f29cf4c7f8` | `28d92d89c437eea7fdc260240585bca466dee1fd74b9b3eb38513d2b34b0be64` |
| Combat policy view-v1 | `spire-gym-combat-policy-view-v1` | `02009686282c592f971eece3d5853eb54f2a482dde9dae1d0cf380ecdaf8fd78` | `08bb0e2244325c5ae33caea9b3c0f78ceda8cc79ea9100e77618b9d7a8d880db` |
| Combat policy view-v2 | `spire-gym-combat-policy-view-v2` | `02433813e1297244060d9852f342c6eb4c9c4d62f3dde9763ff82191867eceec` | `a27083bad27ffdd1fafea4c8de2b1580edad3c9d10200695694e15858b6122f7` |
| M12 Silent Act1 | `spire-gym-m12-silent-act1-observation-v1` | `366c572af8141146995e849acc737a33484623790e2b3d661e290fadfb6c9fac` | `ae0b995eade57ef1dccc39bd9cad6064d8bd33a1cfbc37a312e4b76ac01ceb20` |
| M13 Silent Act2 | `spire-gym-m13-silent-act2-observation-v1` | `3f1e21d50106b8ca71d7cd67a2de7bfa4074455dbbb459629d8e2ac24cf39cf8` | `cd7f78f7bc8ae8566f727f431f8a8d840303cdcb556173b31bf169cea8c34ecf` |
| M14 Silent Act3 | `spire-gym-m14-silent-act3-observation-v1` | `50a685721e5db56ed7a5332160f3747087f96ce738da020415b1427159377aee` | `72e75d8357ebbbba2b905f381262a99a4385c670d9fa20fa8cd008f4b90fb827` |
| M15 Silent Heart | `spire-gym-m15-silent-heart-observation-v1` | `254fb75b404fc558070468c6397324e68f962f23b33aca819b54fecf142ea11a` | `ab9fd38b68e7bd276526e136caf58fbbf6f0ef5916eb67fcba20fd91b54004f1` |
| M16 Defect combat | `spire-gym-m16-defect-combat-observation-v1` | `7bf5a854cd5fb4d05b1645960c1c39d1e23f937e1e8edda02aad13ba904a0183` | `754e496dadc127be2f8409a88f9e85393b629919e449e5c459a712a288b5ed58` |
| M17 Defect Act1 | `spire-gym-m17-defect-act1-observation-v1` | `2176f835eca1384b2070f3d7d001158ef1fead23be5d6d7eea138b57f6bff999` | `eeb45fcedd185710aeb0e5cb58ca619cc5927861cebda39b1612ff1500887a92` |
| M18 Defect Act2 | `spire-gym-m18-defect-act2-observation-v1` | `e097a8202f7cf7433f0ce57a39807bb74d370fba75a9aa84af65d726269564bb` | `6a4bb8526357f0264ae0cad8ccaad8015a0a3d360a678d236d94afe21a4ac967` |
| M19 Defect Act3 | `spire-gym-m19-defect-act3-observation-v1` | `149092e11432e182920b5f07e0728dac45e880296815d9b84fba9b6180c2817b` | `b2aa0b85a83a51171524a7487ab7cb3ed450e00811bd7f00c472e358a0220b15` |
| M20 Defect Heart | `spire-gym-m20-defect-heart-observation-v1` | `8c03211b4a3c589cfb55ba1078ddd447a444f3f53d8e9bfdab9ab85ab80e512d` | `3f11018081cfe0a2e9e69c87d57a57bd4043df2496c6d2c539dd74b0bf20fb56` |
| M21 Watcher combat | `spire-gym-m21-watcher-combat-observation-v1` | `271bd249d8da1a9db5e3e8372af8c3b8df3b7b8266bb0e3663744d52085e2c2c` | `99f84de7f3eb8a7dab49b3662c8ccb425a7b60e2ea9244938302b390458c890c` |
| M22 Watcher Act1 | `spire-gym-m22-watcher-act1-observation-v1` | `5d1c77658141b2dfefbe95e17b85899ad6b77fb370e402bcbbe771146821e087` | `2e4bcade98a3ab7beb402fca027aaed460bae6a26ff545754e3fa3b7e9e6b9bb` |
| M23 Watcher Act2 | `spire-gym-m23-watcher-act2-observation-v1` | `5d5524ff7f60575dea528d42808f7a0a5e67c41bfd0e3b36f7f83b663c9f9d7e` | `5b5f5ed8438b94d14bdb2cfa362229ddcb54df0a1091ba3ccb6ecfbdb87ff8bb` |
| M24 Watcher Act3 | `spire-gym-m24-watcher-act3-observation-v1` | `453b433051801b5cfbfd40bedbd70a8d2d53590e1ebffb4969c8cb17bc5b7564` | `3116abae686d6bc3aca5585c9d9fb87c89956543ac0213d324b2aa92ad9239b0` |
| M25 Watcher Heart | `spire-gym-m25-watcher-heart-observation-v1` | `d1197df6ed3d786ffe0413ec88beeb33db44873c8032f6c67d926a109af93e78` | `09355f51be8b906ecfe1bfd3465f400107ec387d4d5ebe1964f80a6810e77550` |

- M21 Watcher combat view at `combat_policy_capacities()`: `e7e49e88d50c947fde064a764153a5846ba3dc6c8298116f03e8126383b51e34` → `bbb3ad92bc04542773435bf30194a6ef7ac533807931832628bb41d7b47af861` (the table row above uses `RunCapacities::m10_default()`).
- M16 Defect combat at `combat_policy_capacities()`: `2b53f59fac1899e231300914b0b88aabc4e26add3492584de479b60e5fafce0a` → `6092fd366225add31b35bde2fb0f8984f943773f8399404d89de410ddf7618d7`.

Schema IDs are unchanged; only descriptor bytes changed. Diagnostic-only test
descriptors (M20, M22–M25 diagnostic specs) change with their parents. Dated
milestone profiles and reconciliation records keep the fingerprints accepted at
their time. Combat-v1 (`schema.rs`, legacy three-power slice) has no power or
intent name vocabulary and is unchanged.
