# Act 1 source identity and numeric-domain joins

**Status: bounded source-join supplement; assignment .44 remains incomplete.** This is a research contract, not final native schema, runtime validation, whole-spec approval, or implementation authorization. The archive is the pinned working reference described in the consolidated specification; evidence below is static/source-record evidence only. No original game class or initializer was executed.

## 1. Identity and code assignment contract

A source identity is the exact owner-qualified source key, not localized display text, a Java enum ordinal, an allocation identity, nor a wire number. A native canonical key is stable, nonlocalized ASCII `snake_case` within its namespace. Source aliases are byte-sensitive and case-sensitive; no normalization, trimming, case-folding, punctuation rewrite, or “first matching alias” is permitted. Duplicate display labels in distinct namespaces/owners remain distinct identities. Dynamic per-instance handles and numeric wire allocation remain the native owner’s responsibility.

A catalog row has these normative columns: `(namespace, canonical_key, exact_source_identity, exact_aliases, lifecycle_owner, public_eligibility, evidence_grade)`. Each source identity and each `(namespace, owner, alias)` must resolve to exactly one canonical key. Unsupported or unproven identities fail closed. Catalog order is native canonical UTF-8 byte order, not historical host/JRE enumeration order. This is a selected native policy, not a source-order claim.

The finite namespace families required by this Act 1 envelope are cards, relics, potions, enemies, encounters, powers, events, rooms, rewards, choices/source-options, properties, intents, orbs, room objects, and public facts. Their reachability includes Ironclad A0, legally Note-reachable other-character cards, Prismatic Shard reward cards and dependent generated/control cards, transformed/generated cards, Act 1 routes and required Act 2 initialization. It does not include other-character starts/runs or Act 2 decisions. A generator must not widen this universe to every ID returned by an arbitrary runtime lookup.

## 2. Concrete identities whose source rows are closed in accepted records

The following tables separate content-kind IDs from encounter/factory IDs and native-only proposal labels. Exact source spellings remain owner-qualified; no localized display name, Java enum ordinal, or host enumeration order is used as a key. In class-alias columns, `—` denotes an exact class spelling identical to the source ID in that row. The enemy-kind table is limited to the local Act 1 universe, not a global all-acts monster namespace; move identifiers remain subordinate to their enemy owner.

### 2.1 Card source-key catalog — complete CardLibrary registrations

This independently authored table enumerates every row statically registered by the pinned `CardLibrary.initialize` red, green, blue, purple, colorless and curse registration methods. The canonical key is the stable class-qualified native key shown; source IDs are exact case-sensitive `AbstractCard.cardID` constructor constants, and the Java class spelling is retained as a separate exact alias. In the alias column, `—` means the class spelling equals the exact source ID shown. Registry owner is the source loader method, not a claim that every card belongs in every random pool. Legal pool eligibility additionally follows the instantiated card color/rarity and each source selector; event, copy, temporary, Note and potion routes keep their own acquisition path. Public emission is allowed only for a current card instance/offer whose identity the source checkpoint actually reveals; registering a kind never reveals hidden instance identity.

| Canonical key | Exact source ID (`cardID`) | Exact class alias | Source registry owner | Eligibility / emission |
|---|---|---|---|---|
| `card.anger` | `Anger` | — | `addRedCards` (red) | `CardLibrary.addRedCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.armaments` | `Armaments` | — | `addRedCards` (red) | `CardLibrary.addRedCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.barricade` | `Barricade` | — | `addRedCards` (red) | `CardLibrary.addRedCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.bash` | `Bash` | — | `addRedCards` (red) | `CardLibrary.addRedCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.battle_trance` | `Battle Trance` | `BattleTrance` | `addRedCards` (red) | `CardLibrary.addRedCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.berserk` | `Berserk` | — | `addRedCards` (red) | `CardLibrary.addRedCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.blood_for_blood` | `Blood for Blood` | `BloodForBlood` | `addRedCards` (red) | `CardLibrary.addRedCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.bloodletting` | `Bloodletting` | — | `addRedCards` (red) | `CardLibrary.addRedCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.bludgeon` | `Bludgeon` | — | `addRedCards` (red) | `CardLibrary.addRedCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.body_slam` | `Body Slam` | `BodySlam` | `addRedCards` (red) | `CardLibrary.addRedCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.brutality` | `Brutality` | — | `addRedCards` (red) | `CardLibrary.addRedCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.burning_pact` | `Burning Pact` | `BurningPact` | `addRedCards` (red) | `CardLibrary.addRedCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.carnage` | `Carnage` | — | `addRedCards` (red) | `CardLibrary.addRedCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.clash` | `Clash` | — | `addRedCards` (red) | `CardLibrary.addRedCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.cleave` | `Cleave` | — | `addRedCards` (red) | `CardLibrary.addRedCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.clothesline` | `Clothesline` | — | `addRedCards` (red) | `CardLibrary.addRedCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.combust` | `Combust` | — | `addRedCards` (red) | `CardLibrary.addRedCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.corruption` | `Corruption` | — | `addRedCards` (red) | `CardLibrary.addRedCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.dark_embrace` | `Dark Embrace` | `DarkEmbrace` | `addRedCards` (red) | `CardLibrary.addRedCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.defend_red` | `Defend_R` | `Defend_Red` | `addRedCards` (red) | `CardLibrary.addRedCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.demon_form` | `Demon Form` | `DemonForm` | `addRedCards` (red) | `CardLibrary.addRedCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.disarm` | `Disarm` | — | `addRedCards` (red) | `CardLibrary.addRedCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.double_tap` | `Double Tap` | `DoubleTap` | `addRedCards` (red) | `CardLibrary.addRedCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.dropkick` | `Dropkick` | — | `addRedCards` (red) | `CardLibrary.addRedCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.dual_wield` | `Dual Wield` | `DualWield` | `addRedCards` (red) | `CardLibrary.addRedCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.entrench` | `Entrench` | — | `addRedCards` (red) | `CardLibrary.addRedCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.evolve` | `Evolve` | — | `addRedCards` (red) | `CardLibrary.addRedCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.exhume` | `Exhume` | — | `addRedCards` (red) | `CardLibrary.addRedCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.feed` | `Feed` | — | `addRedCards` (red) | `CardLibrary.addRedCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.feel_no_pain` | `Feel No Pain` | `FeelNoPain` | `addRedCards` (red) | `CardLibrary.addRedCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.fiend_fire` | `Fiend Fire` | `FiendFire` | `addRedCards` (red) | `CardLibrary.addRedCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.fire_breathing` | `Fire Breathing` | `FireBreathing` | `addRedCards` (red) | `CardLibrary.addRedCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.flame_barrier` | `Flame Barrier` | `FlameBarrier` | `addRedCards` (red) | `CardLibrary.addRedCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.flex` | `Flex` | — | `addRedCards` (red) | `CardLibrary.addRedCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.ghostly_armor` | `Ghostly Armor` | `GhostlyArmor` | `addRedCards` (red) | `CardLibrary.addRedCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.havoc` | `Havoc` | — | `addRedCards` (red) | `CardLibrary.addRedCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.headbutt` | `Headbutt` | — | `addRedCards` (red) | `CardLibrary.addRedCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.heavy_blade` | `Heavy Blade` | `HeavyBlade` | `addRedCards` (red) | `CardLibrary.addRedCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.hemokinesis` | `Hemokinesis` | — | `addRedCards` (red) | `CardLibrary.addRedCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.immolate` | `Immolate` | — | `addRedCards` (red) | `CardLibrary.addRedCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.impervious` | `Impervious` | — | `addRedCards` (red) | `CardLibrary.addRedCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.infernal_blade` | `Infernal Blade` | `InfernalBlade` | `addRedCards` (red) | `CardLibrary.addRedCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.inflame` | `Inflame` | — | `addRedCards` (red) | `CardLibrary.addRedCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.intimidate` | `Intimidate` | — | `addRedCards` (red) | `CardLibrary.addRedCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.iron_wave` | `Iron Wave` | `IronWave` | `addRedCards` (red) | `CardLibrary.addRedCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.juggernaut` | `Juggernaut` | — | `addRedCards` (red) | `CardLibrary.addRedCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.limit_break` | `Limit Break` | `LimitBreak` | `addRedCards` (red) | `CardLibrary.addRedCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.metallicize` | `Metallicize` | — | `addRedCards` (red) | `CardLibrary.addRedCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.offering` | `Offering` | — | `addRedCards` (red) | `CardLibrary.addRedCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.perfected_strike` | `Perfected Strike` | `PerfectedStrike` | `addRedCards` (red) | `CardLibrary.addRedCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.pommel_strike` | `Pommel Strike` | `PommelStrike` | `addRedCards` (red) | `CardLibrary.addRedCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.power_through` | `Power Through` | `PowerThrough` | `addRedCards` (red) | `CardLibrary.addRedCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.pummel` | `Pummel` | — | `addRedCards` (red) | `CardLibrary.addRedCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.rage` | `Rage` | — | `addRedCards` (red) | `CardLibrary.addRedCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.rampage` | `Rampage` | — | `addRedCards` (red) | `CardLibrary.addRedCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.reaper` | `Reaper` | — | `addRedCards` (red) | `CardLibrary.addRedCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.reckless_charge` | `Reckless Charge` | `RecklessCharge` | `addRedCards` (red) | `CardLibrary.addRedCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.rupture` | `Rupture` | — | `addRedCards` (red) | `CardLibrary.addRedCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.searing_blow` | `Searing Blow` | `SearingBlow` | `addRedCards` (red) | `CardLibrary.addRedCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.second_wind` | `Second Wind` | `SecondWind` | `addRedCards` (red) | `CardLibrary.addRedCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.seeing_red` | `Seeing Red` | `SeeingRed` | `addRedCards` (red) | `CardLibrary.addRedCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.sentinel` | `Sentinel` | — | `addRedCards` (red) | `CardLibrary.addRedCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.sever_soul` | `Sever Soul` | `SeverSoul` | `addRedCards` (red) | `CardLibrary.addRedCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.shockwave` | `Shockwave` | — | `addRedCards` (red) | `CardLibrary.addRedCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.shrug_it_off` | `Shrug It Off` | `ShrugItOff` | `addRedCards` (red) | `CardLibrary.addRedCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.spot_weakness` | `Spot Weakness` | `SpotWeakness` | `addRedCards` (red) | `CardLibrary.addRedCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.strike_red` | `Strike_R` | `Strike_Red` | `addRedCards` (red) | `CardLibrary.addRedCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.sword_boomerang` | `Sword Boomerang` | `SwordBoomerang` | `addRedCards` (red) | `CardLibrary.addRedCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.thunder_clap` | `Thunderclap` | `ThunderClap` | `addRedCards` (red) | `CardLibrary.addRedCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.true_grit` | `True Grit` | `TrueGrit` | `addRedCards` (red) | `CardLibrary.addRedCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.twin_strike` | `Twin Strike` | `TwinStrike` | `addRedCards` (red) | `CardLibrary.addRedCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.uppercut` | `Uppercut` | — | `addRedCards` (red) | `CardLibrary.addRedCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.warcry` | `Warcry` | — | `addRedCards` (red) | `CardLibrary.addRedCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.whirlwind` | `Whirlwind` | — | `addRedCards` (red) | `CardLibrary.addRedCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.wild_strike` | `Wild Strike` | `WildStrike` | `addRedCards` (red) | `CardLibrary.addRedCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.accuracy` | `Accuracy` | — | `addGreenCards` (green) | `CardLibrary.addGreenCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.acrobatics` | `Acrobatics` | — | `addGreenCards` (green) | `CardLibrary.addGreenCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.adrenaline` | `Adrenaline` | — | `addGreenCards` (green) | `CardLibrary.addGreenCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.after_image` | `After Image` | `AfterImage` | `addGreenCards` (green) | `CardLibrary.addGreenCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.alchemize` | `Venomology` | `Alchemize` | `addGreenCards` (green) | `CardLibrary.addGreenCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.all_out_attack` | `All Out Attack` | `AllOutAttack` | `addGreenCards` (green) | `CardLibrary.addGreenCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.a_thousand_cuts` | `A Thousand Cuts` | `AThousandCuts` | `addGreenCards` (green) | `CardLibrary.addGreenCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.backflip` | `Backflip` | — | `addGreenCards` (green) | `CardLibrary.addGreenCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.backstab` | `Backstab` | — | `addGreenCards` (green) | `CardLibrary.addGreenCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.bane` | `Bane` | — | `addGreenCards` (green) | `CardLibrary.addGreenCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.blade_dance` | `Blade Dance` | `BladeDance` | `addGreenCards` (green) | `CardLibrary.addGreenCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.blur` | `Blur` | — | `addGreenCards` (green) | `CardLibrary.addGreenCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.bouncing_flask` | `Bouncing Flask` | `BouncingFlask` | `addGreenCards` (green) | `CardLibrary.addGreenCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.bullet_time` | `Bullet Time` | `BulletTime` | `addGreenCards` (green) | `CardLibrary.addGreenCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.burst` | `Burst` | — | `addGreenCards` (green) | `CardLibrary.addGreenCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.calculated_gamble` | `Calculated Gamble` | `CalculatedGamble` | `addGreenCards` (green) | `CardLibrary.addGreenCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.caltrops` | `Caltrops` | — | `addGreenCards` (green) | `CardLibrary.addGreenCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.catalyst` | `Catalyst` | — | `addGreenCards` (green) | `CardLibrary.addGreenCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.choke` | `Choke` | — | `addGreenCards` (green) | `CardLibrary.addGreenCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.cloak_and_dagger` | `Cloak And Dagger` | `CloakAndDagger` | `addGreenCards` (green) | `CardLibrary.addGreenCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.concentrate` | `Concentrate` | — | `addGreenCards` (green) | `CardLibrary.addGreenCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.corpse_explosion` | `Corpse Explosion` | `CorpseExplosion` | `addGreenCards` (green) | `CardLibrary.addGreenCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.crippling_poison` | `Crippling Poison` | `CripplingPoison` | `addGreenCards` (green) | `CardLibrary.addGreenCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.dagger_spray` | `Dagger Spray` | `DaggerSpray` | `addGreenCards` (green) | `CardLibrary.addGreenCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.dagger_throw` | `Dagger Throw` | `DaggerThrow` | `addGreenCards` (green) | `CardLibrary.addGreenCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.dash` | `Dash` | — | `addGreenCards` (green) | `CardLibrary.addGreenCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.deadly_poison` | `Deadly Poison` | `DeadlyPoison` | `addGreenCards` (green) | `CardLibrary.addGreenCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.defend_green` | `Defend_G` | `Defend_Green` | `addGreenCards` (green) | `CardLibrary.addGreenCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.deflect` | `Deflect` | — | `addGreenCards` (green) | `CardLibrary.addGreenCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.die_die_die` | `Die Die Die` | `DieDieDie` | `addGreenCards` (green) | `CardLibrary.addGreenCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.distraction` | `Distraction` | — | `addGreenCards` (green) | `CardLibrary.addGreenCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.dodge_and_roll` | `Dodge and Roll` | `DodgeAndRoll` | `addGreenCards` (green) | `CardLibrary.addGreenCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.doppelganger` | `Doppelganger` | — | `addGreenCards` (green) | `CardLibrary.addGreenCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.endless_agony` | `Endless Agony` | `EndlessAgony` | `addGreenCards` (green) | `CardLibrary.addGreenCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.envenom` | `Envenom` | — | `addGreenCards` (green) | `CardLibrary.addGreenCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.escape_plan` | `Escape Plan` | `EscapePlan` | `addGreenCards` (green) | `CardLibrary.addGreenCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.eviscerate` | `Eviscerate` | — | `addGreenCards` (green) | `CardLibrary.addGreenCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.expertise` | `Expertise` | — | `addGreenCards` (green) | `CardLibrary.addGreenCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.finisher` | `Finisher` | — | `addGreenCards` (green) | `CardLibrary.addGreenCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.flechettes` | `Flechettes` | — | `addGreenCards` (green) | `CardLibrary.addGreenCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.flying_knee` | `Flying Knee` | `FlyingKnee` | `addGreenCards` (green) | `CardLibrary.addGreenCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.footwork` | `Footwork` | — | `addGreenCards` (green) | `CardLibrary.addGreenCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.glass_knife` | `Glass Knife` | `GlassKnife` | `addGreenCards` (green) | `CardLibrary.addGreenCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.grand_finale` | `Grand Finale` | `GrandFinale` | `addGreenCards` (green) | `CardLibrary.addGreenCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.heel_hook` | `Heel Hook` | `HeelHook` | `addGreenCards` (green) | `CardLibrary.addGreenCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.infinite_blades` | `Infinite Blades` | `InfiniteBlades` | `addGreenCards` (green) | `CardLibrary.addGreenCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.leg_sweep` | `Leg Sweep` | `LegSweep` | `addGreenCards` (green) | `CardLibrary.addGreenCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.malaise` | `Malaise` | — | `addGreenCards` (green) | `CardLibrary.addGreenCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.masterful_stab` | `Masterful Stab` | `MasterfulStab` | `addGreenCards` (green) | `CardLibrary.addGreenCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.neutralize` | `Neutralize` | — | `addGreenCards` (green) | `CardLibrary.addGreenCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.nightmare` | `Night Terror` | `Nightmare` | `addGreenCards` (green) | `CardLibrary.addGreenCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.noxious_fumes` | `Noxious Fumes` | `NoxiousFumes` | `addGreenCards` (green) | `CardLibrary.addGreenCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.outmaneuver` | `Outmaneuver` | — | `addGreenCards` (green) | `CardLibrary.addGreenCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.phantasmal_killer` | `Phantasmal Killer` | `PhantasmalKiller` | `addGreenCards` (green) | `CardLibrary.addGreenCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.piercing_wail` | `PiercingWail` | — | `addGreenCards` (green) | `CardLibrary.addGreenCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.poisoned_stab` | `Poisoned Stab` | `PoisonedStab` | `addGreenCards` (green) | `CardLibrary.addGreenCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.predator` | `Predator` | — | `addGreenCards` (green) | `CardLibrary.addGreenCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.prepared` | `Prepared` | — | `addGreenCards` (green) | `CardLibrary.addGreenCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.quick_slash` | `Quick Slash` | `QuickSlash` | `addGreenCards` (green) | `CardLibrary.addGreenCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.reflex` | `Reflex` | — | `addGreenCards` (green) | `CardLibrary.addGreenCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.riddle_with_holes` | `Riddle With Holes` | `RiddleWithHoles` | `addGreenCards` (green) | `CardLibrary.addGreenCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.setup` | `Setup` | — | `addGreenCards` (green) | `CardLibrary.addGreenCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.skewer` | `Skewer` | — | `addGreenCards` (green) | `CardLibrary.addGreenCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.slice` | `Slice` | — | `addGreenCards` (green) | `CardLibrary.addGreenCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.storm_of_steel` | `Storm of Steel` | `StormOfSteel` | `addGreenCards` (green) | `CardLibrary.addGreenCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.strike_green` | `Strike_G` | `Strike_Green` | `addGreenCards` (green) | `CardLibrary.addGreenCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.sucker_punch` | `Sucker Punch` | `SuckerPunch` | `addGreenCards` (green) | `CardLibrary.addGreenCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.survivor` | `Survivor` | — | `addGreenCards` (green) | `CardLibrary.addGreenCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.tactician` | `Tactician` | — | `addGreenCards` (green) | `CardLibrary.addGreenCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.terror` | `Terror` | — | `addGreenCards` (green) | `CardLibrary.addGreenCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.tools_of_the_trade` | `Tools of the Trade` | `ToolsOfTheTrade` | `addGreenCards` (green) | `CardLibrary.addGreenCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.sneaky_strike` | `Underhanded Strike` | `SneakyStrike` | `addGreenCards` (green) | `CardLibrary.addGreenCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.unload` | `Unload` | — | `addGreenCards` (green) | `CardLibrary.addGreenCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.well_laid_plans` | `Well Laid Plans` | `WellLaidPlans` | `addGreenCards` (green) | `CardLibrary.addGreenCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.wraith_form` | `Wraith Form v2` | `WraithForm` | `addGreenCards` (green) | `CardLibrary.addGreenCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.aggregate` | `Aggregate` | — | `addBlueCards` (blue) | `CardLibrary.addBlueCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.all_for_one` | `All For One` | `AllForOne` | `addBlueCards` (blue) | `CardLibrary.addBlueCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.amplify` | `Amplify` | — | `addBlueCards` (blue) | `CardLibrary.addBlueCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.auto_shields` | `Auto Shields` | `AutoShields` | `addBlueCards` (blue) | `CardLibrary.addBlueCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.ball_lightning` | `Ball Lightning` | `BallLightning` | `addBlueCards` (blue) | `CardLibrary.addBlueCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.barrage` | `Barrage` | — | `addBlueCards` (blue) | `CardLibrary.addBlueCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.beam_cell` | `Beam Cell` | `BeamCell` | `addBlueCards` (blue) | `CardLibrary.addBlueCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.biased_cognition` | `Biased Cognition` | `BiasedCognition` | `addBlueCards` (blue) | `CardLibrary.addBlueCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.blizzard` | `Blizzard` | — | `addBlueCards` (blue) | `CardLibrary.addBlueCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.boot_sequence` | `BootSequence` | — | `addBlueCards` (blue) | `CardLibrary.addBlueCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.buffer` | `Buffer` | — | `addBlueCards` (blue) | `CardLibrary.addBlueCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.capacitor` | `Capacitor` | — | `addBlueCards` (blue) | `CardLibrary.addBlueCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.chaos` | `Chaos` | — | `addBlueCards` (blue) | `CardLibrary.addBlueCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.chill` | `Chill` | — | `addBlueCards` (blue) | `CardLibrary.addBlueCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.claw` | `Gash` | `Claw` | `addBlueCards` (blue) | `CardLibrary.addBlueCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.cold_snap` | `Cold Snap` | `ColdSnap` | `addBlueCards` (blue) | `CardLibrary.addBlueCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.compile_driver` | `Compile Driver` | `CompileDriver` | `addBlueCards` (blue) | `CardLibrary.addBlueCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.conserve_battery` | `Conserve Battery` | `ConserveBattery` | `addBlueCards` (blue) | `CardLibrary.addBlueCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.consume` | `Consume` | — | `addBlueCards` (blue) | `CardLibrary.addBlueCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.coolheaded` | `Coolheaded` | — | `addBlueCards` (blue) | `CardLibrary.addBlueCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.core_surge` | `Core Surge` | `CoreSurge` | `addBlueCards` (blue) | `CardLibrary.addBlueCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.creative_ai` | `Creative AI` | `CreativeAI` | `addBlueCards` (blue) | `CardLibrary.addBlueCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.darkness` | `Darkness` | — | `addBlueCards` (blue) | `CardLibrary.addBlueCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.defend_blue` | `Defend_B` | `Defend_Blue` | `addBlueCards` (blue) | `CardLibrary.addBlueCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.defragment` | `Defragment` | — | `addBlueCards` (blue) | `CardLibrary.addBlueCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.doom_and_gloom` | `Doom and Gloom` | `DoomAndGloom` | `addBlueCards` (blue) | `CardLibrary.addBlueCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.double_energy` | `Double Energy` | `DoubleEnergy` | `addBlueCards` (blue) | `CardLibrary.addBlueCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.dualcast` | `Dualcast` | — | `addBlueCards` (blue) | `CardLibrary.addBlueCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.echo_form` | `Echo Form` | `EchoForm` | `addBlueCards` (blue) | `CardLibrary.addBlueCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.electrodynamics` | `Electrodynamics` | — | `addBlueCards` (blue) | `CardLibrary.addBlueCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.fission` | `Fission` | — | `addBlueCards` (blue) | `CardLibrary.addBlueCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.force_field` | `Force Field` | `ForceField` | `addBlueCards` (blue) | `CardLibrary.addBlueCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.ftl` | `FTL` | — | `addBlueCards` (blue) | `CardLibrary.addBlueCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.fusion` | `Fusion` | — | `addBlueCards` (blue) | `CardLibrary.addBlueCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.genetic_algorithm` | `Genetic Algorithm` | `GeneticAlgorithm` | `addBlueCards` (blue) | `CardLibrary.addBlueCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.glacier` | `Glacier` | — | `addBlueCards` (blue) | `CardLibrary.addBlueCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.go_for_the_eyes` | `Go for the Eyes` | `GoForTheEyes` | `addBlueCards` (blue) | `CardLibrary.addBlueCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.heatsinks` | `Heatsinks` | — | `addBlueCards` (blue) | `CardLibrary.addBlueCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.hello_world` | `Hello World` | `HelloWorld` | `addBlueCards` (blue) | `CardLibrary.addBlueCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.hologram` | `Hologram` | — | `addBlueCards` (blue) | `CardLibrary.addBlueCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.hyperbeam` | `Hyperbeam` | — | `addBlueCards` (blue) | `CardLibrary.addBlueCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.leap` | `Leap` | — | `addBlueCards` (blue) | `CardLibrary.addBlueCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.lock_on` | `Lockon` | `LockOn` | `addBlueCards` (blue) | `CardLibrary.addBlueCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.loop` | `Loop` | — | `addBlueCards` (blue) | `CardLibrary.addBlueCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.machine_learning` | `Machine Learning` | `MachineLearning` | `addBlueCards` (blue) | `CardLibrary.addBlueCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.melter` | `Melter` | — | `addBlueCards` (blue) | `CardLibrary.addBlueCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.meteor_strike` | `Meteor Strike` | `MeteorStrike` | `addBlueCards` (blue) | `CardLibrary.addBlueCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.multi_cast` | `Multi-Cast` | `MultiCast` | `addBlueCards` (blue) | `CardLibrary.addBlueCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.overclock` | `Steam Power` | `Overclock` | `addBlueCards` (blue) | `CardLibrary.addBlueCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.rainbow` | `Rainbow` | — | `addBlueCards` (blue) | `CardLibrary.addBlueCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.reboot` | `Reboot` | — | `addBlueCards` (blue) | `CardLibrary.addBlueCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.rebound` | `Rebound` | — | `addBlueCards` (blue) | `CardLibrary.addBlueCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.recursion` | `Redo` | `Recursion` | `addBlueCards` (blue) | `CardLibrary.addBlueCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.recycle` | `Recycle` | — | `addBlueCards` (blue) | `CardLibrary.addBlueCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.reinforced_body` | `Reinforced Body` | `ReinforcedBody` | `addBlueCards` (blue) | `CardLibrary.addBlueCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.reprogram` | `Reprogram` | — | `addBlueCards` (blue) | `CardLibrary.addBlueCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.rip_and_tear` | `Rip and Tear` | `RipAndTear` | `addBlueCards` (blue) | `CardLibrary.addBlueCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.scrape` | `Scrape` | — | `addBlueCards` (blue) | `CardLibrary.addBlueCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.seek` | `Seek` | — | `addBlueCards` (blue) | `CardLibrary.addBlueCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.self_repair` | `Self Repair` | `SelfRepair` | `addBlueCards` (blue) | `CardLibrary.addBlueCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.skim` | `Skim` | — | `addBlueCards` (blue) | `CardLibrary.addBlueCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.stack` | `Stack` | — | `addBlueCards` (blue) | `CardLibrary.addBlueCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.static_discharge` | `Static Discharge` | `StaticDischarge` | `addBlueCards` (blue) | `CardLibrary.addBlueCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.steam_barrier` | `Steam` | `SteamBarrier` | `addBlueCards` (blue) | `CardLibrary.addBlueCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.storm` | `Storm` | — | `addBlueCards` (blue) | `CardLibrary.addBlueCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.streamline` | `Streamline` | — | `addBlueCards` (blue) | `CardLibrary.addBlueCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.strike_blue` | `Strike_B` | `Strike_Blue` | `addBlueCards` (blue) | `CardLibrary.addBlueCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.sunder` | `Sunder` | — | `addBlueCards` (blue) | `CardLibrary.addBlueCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.sweeping_beam` | `Sweeping Beam` | `SweepingBeam` | `addBlueCards` (blue) | `CardLibrary.addBlueCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.tempest` | `Tempest` | — | `addBlueCards` (blue) | `CardLibrary.addBlueCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.thunder_strike` | `Thunder Strike` | `ThunderStrike` | `addBlueCards` (blue) | `CardLibrary.addBlueCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.turbo` | `Turbo` | — | `addBlueCards` (blue) | `CardLibrary.addBlueCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.equilibrium` | `Undo` | `Equilibrium` | `addBlueCards` (blue) | `CardLibrary.addBlueCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.white_noise` | `White Noise` | `WhiteNoise` | `addBlueCards` (blue) | `CardLibrary.addBlueCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.zap` | `Zap` | — | `addBlueCards` (blue) | `CardLibrary.addBlueCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.alpha` | `Alpha` | — | `addPurpleCards` (purple) | `CardLibrary.addPurpleCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.battle_hymn` | `BattleHymn` | — | `addPurpleCards` (purple) | `CardLibrary.addPurpleCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.blasphemy` | `Blasphemy` | — | `addPurpleCards` (purple) | `CardLibrary.addPurpleCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.bowling_bash` | `BowlingBash` | — | `addPurpleCards` (purple) | `CardLibrary.addPurpleCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.brilliance` | `Brilliance` | — | `addPurpleCards` (purple) | `CardLibrary.addPurpleCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.carve_reality` | `CarveReality` | — | `addPurpleCards` (purple) | `CardLibrary.addPurpleCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.collect` | `Collect` | — | `addPurpleCards` (purple) | `CardLibrary.addPurpleCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.conclude` | `Conclude` | — | `addPurpleCards` (purple) | `CardLibrary.addPurpleCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.conjure_blade` | `ConjureBlade` | — | `addPurpleCards` (purple) | `CardLibrary.addPurpleCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.consecrate` | `Consecrate` | — | `addPurpleCards` (purple) | `CardLibrary.addPurpleCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.crescendo` | `Crescendo` | — | `addPurpleCards` (purple) | `CardLibrary.addPurpleCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.crush_joints` | `CrushJoints` | — | `addPurpleCards` (purple) | `CardLibrary.addPurpleCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.cut_through_fate` | `CutThroughFate` | — | `addPurpleCards` (purple) | `CardLibrary.addPurpleCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.deceive_reality` | `DeceiveReality` | — | `addPurpleCards` (purple) | `CardLibrary.addPurpleCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.defend_watcher` | `Defend_P` | `Defend_Watcher` | `addPurpleCards` (purple) | `CardLibrary.addPurpleCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.deus_ex_machina` | `DeusExMachina` | — | `addPurpleCards` (purple) | `CardLibrary.addPurpleCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.deva_form` | `DevaForm` | — | `addPurpleCards` (purple) | `CardLibrary.addPurpleCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.devotion` | `Devotion` | — | `addPurpleCards` (purple) | `CardLibrary.addPurpleCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.empty_body` | `EmptyBody` | — | `addPurpleCards` (purple) | `CardLibrary.addPurpleCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.empty_fist` | `EmptyFist` | — | `addPurpleCards` (purple) | `CardLibrary.addPurpleCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.empty_mind` | `EmptyMind` | — | `addPurpleCards` (purple) | `CardLibrary.addPurpleCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.eruption` | `Eruption` | — | `addPurpleCards` (purple) | `CardLibrary.addPurpleCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.establishment` | `Establishment` | — | `addPurpleCards` (purple) | `CardLibrary.addPurpleCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.evaluate` | `Evaluate` | — | `addPurpleCards` (purple) | `CardLibrary.addPurpleCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.fasting` | `Fasting2` | `Fasting` | `addPurpleCards` (purple) | `CardLibrary.addPurpleCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.fear_no_evil` | `FearNoEvil` | — | `addPurpleCards` (purple) | `CardLibrary.addPurpleCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.flurry_of_blows` | `FlurryOfBlows` | — | `addPurpleCards` (purple) | `CardLibrary.addPurpleCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.flying_sleeves` | `FlyingSleeves` | — | `addPurpleCards` (purple) | `CardLibrary.addPurpleCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.follow_up` | `FollowUp` | — | `addPurpleCards` (purple) | `CardLibrary.addPurpleCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.foreign_influence` | `ForeignInfluence` | — | `addPurpleCards` (purple) | `CardLibrary.addPurpleCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.foresight` | `Wireheading` | `Foresight` | `addPurpleCards` (purple) | `CardLibrary.addPurpleCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.halt` | `Halt` | — | `addPurpleCards` (purple) | `CardLibrary.addPurpleCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.indignation` | `Indignation` | — | `addPurpleCards` (purple) | `CardLibrary.addPurpleCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.inner_peace` | `InnerPeace` | — | `addPurpleCards` (purple) | `CardLibrary.addPurpleCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.judgement` | `Judgement` | — | `addPurpleCards` (purple) | `CardLibrary.addPurpleCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.just_lucky` | `JustLucky` | — | `addPurpleCards` (purple) | `CardLibrary.addPurpleCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.lesson_learned` | `LessonLearned` | — | `addPurpleCards` (purple) | `CardLibrary.addPurpleCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.like_water` | `LikeWater` | — | `addPurpleCards` (purple) | `CardLibrary.addPurpleCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.master_reality` | `MasterReality` | — | `addPurpleCards` (purple) | `CardLibrary.addPurpleCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.meditate` | `Meditate` | — | `addPurpleCards` (purple) | `CardLibrary.addPurpleCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.mental_fortress` | `MentalFortress` | — | `addPurpleCards` (purple) | `CardLibrary.addPurpleCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.nirvana` | `Nirvana` | — | `addPurpleCards` (purple) | `CardLibrary.addPurpleCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.omniscience` | `Omniscience` | — | `addPurpleCards` (purple) | `CardLibrary.addPurpleCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.perseverance` | `Perseverance` | — | `addPurpleCards` (purple) | `CardLibrary.addPurpleCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.pray` | `Pray` | — | `addPurpleCards` (purple) | `CardLibrary.addPurpleCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.pressure_points` | `PathToVictory` | `PressurePoints` | `addPurpleCards` (purple) | `CardLibrary.addPurpleCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.prostrate` | `Prostrate` | — | `addPurpleCards` (purple) | `CardLibrary.addPurpleCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.protect` | `Protect` | — | `addPurpleCards` (purple) | `CardLibrary.addPurpleCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.ragnarok` | `Ragnarok` | — | `addPurpleCards` (purple) | `CardLibrary.addPurpleCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.reach_heaven` | `ReachHeaven` | — | `addPurpleCards` (purple) | `CardLibrary.addPurpleCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.rushdown` | `Adaptation` | `Rushdown` | `addPurpleCards` (purple) | `CardLibrary.addPurpleCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.sanctity` | `Sanctity` | — | `addPurpleCards` (purple) | `CardLibrary.addPurpleCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.sands_of_time` | `SandsOfTime` | — | `addPurpleCards` (purple) | `CardLibrary.addPurpleCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.sash_whip` | `SashWhip` | — | `addPurpleCards` (purple) | `CardLibrary.addPurpleCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.scrawl` | `Scrawl` | — | `addPurpleCards` (purple) | `CardLibrary.addPurpleCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.signature_move` | `SignatureMove` | — | `addPurpleCards` (purple) | `CardLibrary.addPurpleCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.simmering_fury` | `Vengeance` | `SimmeringFury` | `addPurpleCards` (purple) | `CardLibrary.addPurpleCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.spirit_shield` | `SpiritShield` | — | `addPurpleCards` (purple) | `CardLibrary.addPurpleCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.strike_purple` | `Strike_P` | `Strike_Purple` | `addPurpleCards` (purple) | `CardLibrary.addPurpleCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.study` | `Study` | — | `addPurpleCards` (purple) | `CardLibrary.addPurpleCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.swivel` | `Swivel` | — | `addPurpleCards` (purple) | `CardLibrary.addPurpleCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.talk_to_the_hand` | `TalkToTheHand` | — | `addPurpleCards` (purple) | `CardLibrary.addPurpleCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.tantrum` | `Tantrum` | — | `addPurpleCards` (purple) | `CardLibrary.addPurpleCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.third_eye` | `ThirdEye` | — | `addPurpleCards` (purple) | `CardLibrary.addPurpleCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.tranquility` | `ClearTheMind` | `Tranquility` | `addPurpleCards` (purple) | `CardLibrary.addPurpleCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.vault` | `Vault` | — | `addPurpleCards` (purple) | `CardLibrary.addPurpleCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.vigilance` | `Vigilance` | — | `addPurpleCards` (purple) | `CardLibrary.addPurpleCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.wallop` | `Wallop` | — | `addPurpleCards` (purple) | `CardLibrary.addPurpleCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.wave_of_the_hand` | `WaveOfTheHand` | — | `addPurpleCards` (purple) | `CardLibrary.addPurpleCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.weave` | `Weave` | — | `addPurpleCards` (purple) | `CardLibrary.addPurpleCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.wheel_kick` | `WheelKick` | — | `addPurpleCards` (purple) | `CardLibrary.addPurpleCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.windmill_strike` | `WindmillStrike` | — | `addPurpleCards` (purple) | `CardLibrary.addPurpleCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.wish` | `Wish` | — | `addPurpleCards` (purple) | `CardLibrary.addPurpleCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.worship` | `Worship` | — | `addPurpleCards` (purple) | `CardLibrary.addPurpleCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.wreath_of_flame` | `WreathOfFlame` | — | `addPurpleCards` (purple) | `CardLibrary.addPurpleCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.apotheosis` | `Apotheosis` | — | `addColorlessCards` (colorless) | `CardLibrary.addColorlessCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.bandage_up` | `Bandage Up` | `BandageUp` | `addColorlessCards` (colorless) | `CardLibrary.addColorlessCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.blind` | `Blind` | — | `addColorlessCards` (colorless) | `CardLibrary.addColorlessCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.chrysalis` | `Chrysalis` | — | `addColorlessCards` (colorless) | `CardLibrary.addColorlessCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.dark_shackles` | `Dark Shackles` | `DarkShackles` | `addColorlessCards` (colorless) | `CardLibrary.addColorlessCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.deep_breath` | `Deep Breath` | `DeepBreath` | `addColorlessCards` (colorless) | `CardLibrary.addColorlessCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.discovery` | `Discovery` | — | `addColorlessCards` (colorless) | `CardLibrary.addColorlessCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.dramatic_entrance` | `Dramatic Entrance` | `DramaticEntrance` | `addColorlessCards` (colorless) | `CardLibrary.addColorlessCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.enlightenment` | `Enlightenment` | — | `addColorlessCards` (colorless) | `CardLibrary.addColorlessCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.finesse` | `Finesse` | — | `addColorlessCards` (colorless) | `CardLibrary.addColorlessCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.flash_of_steel` | `Flash of Steel` | `FlashOfSteel` | `addColorlessCards` (colorless) | `CardLibrary.addColorlessCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.forethought` | `Forethought` | — | `addColorlessCards` (colorless) | `CardLibrary.addColorlessCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.good_instincts` | `Good Instincts` | `GoodInstincts` | `addColorlessCards` (colorless) | `CardLibrary.addColorlessCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.hand_of_greed` | `HandOfGreed` | — | `addColorlessCards` (colorless) | `CardLibrary.addColorlessCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.impatience` | `Impatience` | — | `addColorlessCards` (colorless) | `CardLibrary.addColorlessCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.jack_of_all_trades` | `Jack Of All Trades` | `JackOfAllTrades` | `addColorlessCards` (colorless) | `CardLibrary.addColorlessCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.madness` | `Madness` | — | `addColorlessCards` (colorless) | `CardLibrary.addColorlessCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.magnetism` | `Magnetism` | — | `addColorlessCards` (colorless) | `CardLibrary.addColorlessCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.master_of_strategy` | `Master of Strategy` | `MasterOfStrategy` | `addColorlessCards` (colorless) | `CardLibrary.addColorlessCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.mayhem` | `Mayhem` | — | `addColorlessCards` (colorless) | `CardLibrary.addColorlessCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.metamorphosis` | `Metamorphosis` | — | `addColorlessCards` (colorless) | `CardLibrary.addColorlessCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.mind_blast` | `Mind Blast` | `MindBlast` | `addColorlessCards` (colorless) | `CardLibrary.addColorlessCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.panacea` | `Panacea` | — | `addColorlessCards` (colorless) | `CardLibrary.addColorlessCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.panache` | `Panache` | — | `addColorlessCards` (colorless) | `CardLibrary.addColorlessCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.panic_button` | `PanicButton` | — | `addColorlessCards` (colorless) | `CardLibrary.addColorlessCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.purity` | `Purity` | — | `addColorlessCards` (colorless) | `CardLibrary.addColorlessCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.sadistic_nature` | `Sadistic Nature` | `SadisticNature` | `addColorlessCards` (colorless) | `CardLibrary.addColorlessCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.secret_technique` | `Secret Technique` | `SecretTechnique` | `addColorlessCards` (colorless) | `CardLibrary.addColorlessCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.secret_weapon` | `Secret Weapon` | `SecretWeapon` | `addColorlessCards` (colorless) | `CardLibrary.addColorlessCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.swift_strike` | `Swift Strike` | `SwiftStrike` | `addColorlessCards` (colorless) | `CardLibrary.addColorlessCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.the_bomb` | `The Bomb` | `TheBomb` | `addColorlessCards` (colorless) | `CardLibrary.addColorlessCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.thinking_ahead` | `Thinking Ahead` | `ThinkingAhead` | `addColorlessCards` (colorless) | `CardLibrary.addColorlessCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.transmutation` | `Transmutation` | — | `addColorlessCards` (colorless) | `CardLibrary.addColorlessCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.trip` | `Trip` | — | `addColorlessCards` (colorless) | `CardLibrary.addColorlessCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.violence` | `Violence` | — | `addColorlessCards` (colorless) | `CardLibrary.addColorlessCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.burn` | `Burn` | — | `addColorlessCards` (colorless) | `CardLibrary.addColorlessCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.dazed` | `Dazed` | — | `addColorlessCards` (colorless) | `CardLibrary.addColorlessCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.slimed` | `Slimed` | — | `addColorlessCards` (colorless) | `CardLibrary.addColorlessCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.void_card` | `Void` | `VoidCard` | `addColorlessCards` (colorless) | `CardLibrary.addColorlessCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.wound` | `Wound` | — | `addColorlessCards` (colorless) | `CardLibrary.addColorlessCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.apparition` | `Ghostly` | `Apparition` | `addColorlessCards` (colorless) | `CardLibrary.addColorlessCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.beta` | `Beta` | — | `addColorlessCards` (colorless) | `CardLibrary.addColorlessCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.bite` | `Bite` | — | `addColorlessCards` (colorless) | `CardLibrary.addColorlessCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.jax` | `J.A.X.` | `JAX` | `addColorlessCards` (colorless) | `CardLibrary.addColorlessCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.insight` | `Insight` | — | `addColorlessCards` (colorless) | `CardLibrary.addColorlessCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.miracle` | `Miracle` | — | `addColorlessCards` (colorless) | `CardLibrary.addColorlessCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.omega` | `Omega` | — | `addColorlessCards` (colorless) | `CardLibrary.addColorlessCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.ritual_dagger` | `RitualDagger` | — | `addColorlessCards` (colorless) | `CardLibrary.addColorlessCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.safety` | `Safety` | — | `addColorlessCards` (colorless) | `CardLibrary.addColorlessCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.shiv` | `Shiv` | — | `addColorlessCards` (colorless) | `CardLibrary.addColorlessCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.smite` | `Smite` | — | `addColorlessCards` (colorless) | `CardLibrary.addColorlessCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.through_violence` | `ThroughViolence` | — | `addColorlessCards` (colorless) | `CardLibrary.addColorlessCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.become_almighty` | `BecomeAlmighty` | — | `addColorlessCards` (colorless) | `CardLibrary.addColorlessCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.fame_and_fortune` | `FameAndFortune` | — | `addColorlessCards` (colorless) | `CardLibrary.addColorlessCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.live_forever` | `LiveForever` | — | `addColorlessCards` (colorless) | `CardLibrary.addColorlessCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.expunger` | `Expunger` | — | `addColorlessCards` (colorless) | `CardLibrary.addColorlessCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.ascenders_bane` | `AscendersBane` | — | `addCurseCards` (curse) | `CardLibrary.addCurseCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.curse_of_the_bell` | `CurseOfTheBell` | — | `addCurseCards` (curse) | `CardLibrary.addCurseCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.clumsy` | `Clumsy` | — | `addCurseCards` (curse) | `CardLibrary.addCurseCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.decay` | `Decay` | — | `addCurseCards` (curse) | `CardLibrary.addCurseCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.doubt` | `Doubt` | — | `addCurseCards` (curse) | `CardLibrary.addCurseCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.injury` | `Injury` | — | `addCurseCards` (curse) | `CardLibrary.addCurseCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.necronomicurse` | `Necronomicurse` | — | `addCurseCards` (curse) | `CardLibrary.addCurseCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.normality` | `Normality` | — | `addCurseCards` (curse) | `CardLibrary.addCurseCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.pain` | `Pain` | — | `addCurseCards` (curse) | `CardLibrary.addCurseCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.parasite` | `Parasite` | — | `addCurseCards` (curse) | `CardLibrary.addCurseCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.pride` | `Pride` | — | `addCurseCards` (curse) | `CardLibrary.addCurseCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.regret` | `Regret` | — | `addCurseCards` (curse) | `CardLibrary.addCurseCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.shame` | `Shame` | — | `addCurseCards` (curse) | `CardLibrary.addCurseCards`; route/pool filtered; emit entitled visible instance/offer only |
| `card.writhe` | `Writhe` | — | `addCurseCards` (curse) | `CardLibrary.addCurseCards`; route/pool filtered; emit entitled visible instance/offer only |

The catalogue has 370 unique registered class identities and 370 unique exact constructor IDs in this static inventory. Basic class variants remain separate: e.g. `Strike_R`, `Strike_G`, `Strike_B` and `Strike_Purple` are four different IDs, not display-name aliases. The special dependents registered through the colorless loader retain their exact class/ID keys; they are not removed because ordinary rarity pools omit them. This complete loader set is joined with constructor identity, while the precise legal producer/selector relation is namespace-owner metadata rather than an assumption from loader membership.

### 2.2 Relic identity, alias, pool-owner, and eligibility catalog

This table reproduces the 141 candidate identities in the accepted Act 1 relic reconciliation and binds each exact constructor ID and Java class alias to the static `RelicLibrary.initialize` registration set. `RelicLibrary` is the identity/lookup registry; the pool/source column is the actual candidate-tier/source eligibility stated in the accepted record. This is not a claim that every registered relic is in the Act 1 candidate set: the retained library has 178 registrations, including other-character, Act 2–4, boss, shop, event and special identities outside this Act 1 candidate table. A relic key may be emitted only for an entitled visible inventory/offer/reward instance, and hidden future offers are not exposed.

| Canonical key | Exact source ID | Exact class alias | Registry / lifecycle owner | Tier and actual source eligibility |
|---|---|---|---|---|
| `relic.burning_blood` | `Burning Blood` | `BurningBlood` | `RelicLibrary.initialize` / `RelicLibrary.add` | STARTER; Ironclad-red; Ironclad starting relic; removed when an active starter is not kept; emit only entitled visible live instance/offer |
| `relic.akabeko` | `Akabeko` | — | `RelicLibrary.initialize` / `RelicLibrary.add` | COMMON; shared; Shared or Ironclad-red common relic pool; emit only entitled visible live instance/offer |
| `relic.anchor` | `Anchor` | — | `RelicLibrary.initialize` / `RelicLibrary.add` | COMMON; shared; Shared or Ironclad-red common relic pool; emit only entitled visible live instance/offer |
| `relic.ancient_tea_set` | `Ancient Tea Set` | `AncientTeaSet` | `RelicLibrary.initialize` / `RelicLibrary.add` | COMMON; shared; Shared or Ironclad-red common relic pool; emit only entitled visible live instance/offer |
| `relic.art_of_war` | `Art of War` | `ArtOfWar` | `RelicLibrary.initialize` / `RelicLibrary.add` | COMMON; shared; Shared or Ironclad-red common relic pool; emit only entitled visible live instance/offer |
| `relic.bag_of_marbles` | `Bag of Marbles` | `BagOfMarbles` | `RelicLibrary.initialize` / `RelicLibrary.add` | COMMON; shared; Shared or Ironclad-red common relic pool; emit only entitled visible live instance/offer |
| `relic.bag_of_preparation` | `Bag of Preparation` | `BagOfPreparation` | `RelicLibrary.initialize` / `RelicLibrary.add` | COMMON; shared; Shared or Ironclad-red common relic pool; emit only entitled visible live instance/offer |
| `relic.blood_vial` | `Blood Vial` | `BloodVial` | `RelicLibrary.initialize` / `RelicLibrary.add` | COMMON; shared; Shared or Ironclad-red common relic pool; emit only entitled visible live instance/offer |
| `relic.bronze_scales` | `Bronze Scales` | `BronzeScales` | `RelicLibrary.initialize` / `RelicLibrary.add` | COMMON; shared; Shared or Ironclad-red common relic pool; emit only entitled visible live instance/offer |
| `relic.centennial_puzzle` | `Centennial Puzzle` | `CentennialPuzzle` | `RelicLibrary.initialize` / `RelicLibrary.add` | COMMON; shared; Shared or Ironclad-red common relic pool; emit only entitled visible live instance/offer |
| `relic.ceramic_fish` | `CeramicFish` | — | `RelicLibrary.initialize` / `RelicLibrary.add` | COMMON; shared; Shared or Ironclad-red common relic pool; emit only entitled visible live instance/offer |
| `relic.dream_catcher` | `Dream Catcher` | `DreamCatcher` | `RelicLibrary.initialize` / `RelicLibrary.add` | COMMON; shared; Shared or Ironclad-red common relic pool; emit only entitled visible live instance/offer |
| `relic.happy_flower` | `Happy Flower` | `HappyFlower` | `RelicLibrary.initialize` / `RelicLibrary.add` | COMMON; shared; Shared or Ironclad-red common relic pool; emit only entitled visible live instance/offer |
| `relic.juzu_bracelet` | `Juzu Bracelet` | `JuzuBracelet` | `RelicLibrary.initialize` / `RelicLibrary.add` | COMMON; shared; Shared or Ironclad-red common relic pool; emit only entitled visible live instance/offer |
| `relic.lantern` | `Lantern` | — | `RelicLibrary.initialize` / `RelicLibrary.add` | COMMON; shared; Shared or Ironclad-red common relic pool; emit only entitled visible live instance/offer |
| `relic.maw_bank` | `MawBank` | — | `RelicLibrary.initialize` / `RelicLibrary.add` | COMMON; shared; Shared or Ironclad-red common relic pool; emit only entitled visible live instance/offer |
| `relic.meal_ticket` | `MealTicket` | — | `RelicLibrary.initialize` / `RelicLibrary.add` | COMMON; shared; Shared or Ironclad-red common relic pool; emit only entitled visible live instance/offer |
| `relic.nunchaku` | `Nunchaku` | — | `RelicLibrary.initialize` / `RelicLibrary.add` | COMMON; shared; Shared or Ironclad-red common relic pool; emit only entitled visible live instance/offer |
| `relic.oddly_smooth_stone` | `Oddly Smooth Stone` | `OddlySmoothStone` | `RelicLibrary.initialize` / `RelicLibrary.add` | COMMON; shared; Shared or Ironclad-red common relic pool; emit only entitled visible live instance/offer |
| `relic.omamori` | `Omamori` | — | `RelicLibrary.initialize` / `RelicLibrary.add` | COMMON; shared; Shared or Ironclad-red common relic pool; emit only entitled visible live instance/offer |
| `relic.orichalcum` | `Orichalcum` | — | `RelicLibrary.initialize` / `RelicLibrary.add` | COMMON; shared; Shared or Ironclad-red common relic pool; emit only entitled visible live instance/offer |
| `relic.pen_nib` | `Pen Nib` | `PenNib` | `RelicLibrary.initialize` / `RelicLibrary.add` | COMMON; shared; Shared or Ironclad-red common relic pool; emit only entitled visible live instance/offer |
| `relic.potion_belt` | `Potion Belt` | `PotionBelt` | `RelicLibrary.initialize` / `RelicLibrary.add` | COMMON; shared; Shared or Ironclad-red common relic pool; emit only entitled visible live instance/offer |
| `relic.preserved_insect` | `PreservedInsect` | — | `RelicLibrary.initialize` / `RelicLibrary.add` | COMMON; shared; Shared or Ironclad-red common relic pool; emit only entitled visible live instance/offer |
| `relic.red_skull` | `Red Skull` | `RedSkull` | `RelicLibrary.initialize` / `RelicLibrary.add` | COMMON; Ironclad-red; Shared or Ironclad-red common relic pool; emit only entitled visible live instance/offer |
| `relic.regal_pillow` | `Regal Pillow` | `RegalPillow` | `RelicLibrary.initialize` / `RelicLibrary.add` | COMMON; shared; Shared or Ironclad-red common relic pool; emit only entitled visible live instance/offer |
| `relic.smiling_mask` | `Smiling Mask` | `SmilingMask` | `RelicLibrary.initialize` / `RelicLibrary.add` | COMMON; shared; Shared or Ironclad-red common relic pool; emit only entitled visible live instance/offer |
| `relic.strawberry` | `Strawberry` | — | `RelicLibrary.initialize` / `RelicLibrary.add` | COMMON; shared; Shared or Ironclad-red common relic pool; emit only entitled visible live instance/offer |
| `relic.boot` | `Boot` | — | `RelicLibrary.initialize` / `RelicLibrary.add` | COMMON; shared; Shared or Ironclad-red common relic pool; emit only entitled visible live instance/offer |
| `relic.tiny_chest` | `Tiny Chest` | `TinyChest` | `RelicLibrary.initialize` / `RelicLibrary.add` | COMMON; shared; Shared or Ironclad-red common relic pool; emit only entitled visible live instance/offer |
| `relic.toy_ornithopter` | `Toy Ornithopter` | `ToyOrnithopter` | `RelicLibrary.initialize` / `RelicLibrary.add` | COMMON; shared; Shared or Ironclad-red common relic pool; emit only entitled visible live instance/offer |
| `relic.vajra` | `Vajra` | — | `RelicLibrary.initialize` / `RelicLibrary.add` | COMMON; shared; Shared or Ironclad-red common relic pool; emit only entitled visible live instance/offer |
| `relic.war_paint` | `War Paint` | `WarPaint` | `RelicLibrary.initialize` / `RelicLibrary.add` | COMMON; shared; Shared or Ironclad-red common relic pool; emit only entitled visible live instance/offer |
| `relic.whetstone` | `Whetstone` | — | `RelicLibrary.initialize` / `RelicLibrary.add` | COMMON; shared; Shared or Ironclad-red common relic pool; emit only entitled visible live instance/offer |
| `relic.blue_candle` | `Blue Candle` | `BlueCandle` | `RelicLibrary.initialize` / `RelicLibrary.add` | UNCOMMON; shared; Shared or Ironclad-red uncommon relic pool; emit only entitled visible live instance/offer |
| `relic.bottled_flame` | `Bottled Flame` | `BottledFlame` | `RelicLibrary.initialize` / `RelicLibrary.add` | UNCOMMON; shared; Shared or Ironclad-red uncommon relic pool; emit only entitled visible live instance/offer |
| `relic.bottled_lightning` | `Bottled Lightning` | `BottledLightning` | `RelicLibrary.initialize` / `RelicLibrary.add` | UNCOMMON; shared; Shared or Ironclad-red uncommon relic pool; emit only entitled visible live instance/offer |
| `relic.bottled_tornado` | `Bottled Tornado` | `BottledTornado` | `RelicLibrary.initialize` / `RelicLibrary.add` | UNCOMMON; shared; Shared or Ironclad-red uncommon relic pool; emit only entitled visible live instance/offer |
| `relic.darkstone_periapt` | `Darkstone Periapt` | `DarkstonePeriapt` | `RelicLibrary.initialize` / `RelicLibrary.add` | UNCOMMON; shared; Shared or Ironclad-red uncommon relic pool; emit only entitled visible live instance/offer |
| `relic.eternal_feather` | `Eternal Feather` | `EternalFeather` | `RelicLibrary.initialize` / `RelicLibrary.add` | UNCOMMON; shared; Shared or Ironclad-red uncommon relic pool; emit only entitled visible live instance/offer |
| `relic.frozen_egg2` | `Frozen Egg 2` | `FrozenEgg2` | `RelicLibrary.initialize` / `RelicLibrary.add` | UNCOMMON; shared; Shared or Ironclad-red uncommon relic pool; emit only entitled visible live instance/offer |
| `relic.gremlin_horn` | `Gremlin Horn` | `GremlinHorn` | `RelicLibrary.initialize` / `RelicLibrary.add` | UNCOMMON; shared; Shared or Ironclad-red uncommon relic pool; emit only entitled visible live instance/offer |
| `relic.horn_cleat` | `HornCleat` | — | `RelicLibrary.initialize` / `RelicLibrary.add` | UNCOMMON; shared; Shared or Ironclad-red uncommon relic pool; emit only entitled visible live instance/offer |
| `relic.ink_bottle` | `InkBottle` | — | `RelicLibrary.initialize` / `RelicLibrary.add` | UNCOMMON; shared; Shared or Ironclad-red uncommon relic pool; emit only entitled visible live instance/offer |
| `relic.kunai` | `Kunai` | — | `RelicLibrary.initialize` / `RelicLibrary.add` | UNCOMMON; shared; Shared or Ironclad-red uncommon relic pool; emit only entitled visible live instance/offer |
| `relic.letter_opener` | `Letter Opener` | `LetterOpener` | `RelicLibrary.initialize` / `RelicLibrary.add` | UNCOMMON; shared; Shared or Ironclad-red uncommon relic pool; emit only entitled visible live instance/offer |
| `relic.matryoshka` | `Matryoshka` | — | `RelicLibrary.initialize` / `RelicLibrary.add` | UNCOMMON; shared; Shared or Ironclad-red uncommon relic pool; emit only entitled visible live instance/offer |
| `relic.meat_on_the_bone` | `Meat on the Bone` | `MeatOnTheBone` | `RelicLibrary.initialize` / `RelicLibrary.add` | UNCOMMON; shared; Shared or Ironclad-red uncommon relic pool; emit only entitled visible live instance/offer |
| `relic.mercury_hourglass` | `Mercury Hourglass` | `MercuryHourglass` | `RelicLibrary.initialize` / `RelicLibrary.add` | UNCOMMON; shared; Shared or Ironclad-red uncommon relic pool; emit only entitled visible live instance/offer |
| `relic.molten_egg2` | `Molten Egg 2` | `MoltenEgg2` | `RelicLibrary.initialize` / `RelicLibrary.add` | UNCOMMON; shared; Shared or Ironclad-red uncommon relic pool; emit only entitled visible live instance/offer |
| `relic.mummified_hand` | `Mummified Hand` | `MummifiedHand` | `RelicLibrary.initialize` / `RelicLibrary.add` | UNCOMMON; shared; Shared or Ironclad-red uncommon relic pool; emit only entitled visible live instance/offer |
| `relic.ornamental_fan` | `Ornamental Fan` | `OrnamentalFan` | `RelicLibrary.initialize` / `RelicLibrary.add` | UNCOMMON; shared; Shared or Ironclad-red uncommon relic pool; emit only entitled visible live instance/offer |
| `relic.pantograph` | `Pantograph` | — | `RelicLibrary.initialize` / `RelicLibrary.add` | UNCOMMON; shared; Shared or Ironclad-red uncommon relic pool; emit only entitled visible live instance/offer |
| `relic.paper_frog` | `Paper Frog` | `PaperFrog` | `RelicLibrary.initialize` / `RelicLibrary.add` | UNCOMMON; Ironclad-red; Shared or Ironclad-red uncommon relic pool; emit only entitled visible live instance/offer |
| `relic.pear` | `Pear` | — | `RelicLibrary.initialize` / `RelicLibrary.add` | UNCOMMON; shared; Shared or Ironclad-red uncommon relic pool; emit only entitled visible live instance/offer |
| `relic.question_card` | `Question Card` | `QuestionCard` | `RelicLibrary.initialize` / `RelicLibrary.add` | UNCOMMON; shared; Shared or Ironclad-red uncommon relic pool; emit only entitled visible live instance/offer |
| `relic.self_forming_clay` | `Self Forming Clay` | `SelfFormingClay` | `RelicLibrary.initialize` / `RelicLibrary.add` | UNCOMMON; Ironclad-red; Shared or Ironclad-red uncommon relic pool; emit only entitled visible live instance/offer |
| `relic.shuriken` | `Shuriken` | — | `RelicLibrary.initialize` / `RelicLibrary.add` | UNCOMMON; shared; Shared or Ironclad-red uncommon relic pool; emit only entitled visible live instance/offer |
| `relic.singing_bowl` | `Singing Bowl` | `SingingBowl` | `RelicLibrary.initialize` / `RelicLibrary.add` | UNCOMMON; shared; Shared or Ironclad-red uncommon relic pool; emit only entitled visible live instance/offer |
| `relic.strike_dummy` | `StrikeDummy` | — | `RelicLibrary.initialize` / `RelicLibrary.add` | UNCOMMON; shared; Shared or Ironclad-red uncommon relic pool; emit only entitled visible live instance/offer |
| `relic.sundial` | `Sundial` | — | `RelicLibrary.initialize` / `RelicLibrary.add` | UNCOMMON; shared; Shared or Ironclad-red uncommon relic pool; emit only entitled visible live instance/offer |
| `relic.courier` | `The Courier` | `Courier` | `RelicLibrary.initialize` / `RelicLibrary.add` | UNCOMMON; shared; Shared or Ironclad-red uncommon relic pool; emit only entitled visible live instance/offer |
| `relic.toxic_egg2` | `Toxic Egg 2` | `ToxicEgg2` | `RelicLibrary.initialize` / `RelicLibrary.add` | UNCOMMON; shared; Shared or Ironclad-red uncommon relic pool; emit only entitled visible live instance/offer |
| `relic.white_beast` | `White Beast Statue` | `WhiteBeast` | `RelicLibrary.initialize` / `RelicLibrary.add` | UNCOMMON; shared; Shared or Ironclad-red uncommon relic pool; emit only entitled visible live instance/offer |
| `relic.bird_faced_urn` | `Bird Faced Urn` | `BirdFacedUrn` | `RelicLibrary.initialize` / `RelicLibrary.add` | RARE; shared; Shared or Ironclad-red rare relic pool; emit only entitled visible live instance/offer |
| `relic.calipers` | `Calipers` | — | `RelicLibrary.initialize` / `RelicLibrary.add` | RARE; shared; Shared or Ironclad-red rare relic pool; emit only entitled visible live instance/offer |
| `relic.captains_wheel` | `CaptainsWheel` | — | `RelicLibrary.initialize` / `RelicLibrary.add` | RARE; shared; Shared or Ironclad-red rare relic pool; emit only entitled visible live instance/offer |
| `relic.champions_belt` | `Champion Belt` | `ChampionsBelt` | `RelicLibrary.initialize` / `RelicLibrary.add` | RARE; Ironclad-red; Shared or Ironclad-red rare relic pool; emit only entitled visible live instance/offer |
| `relic.charons_ashes` | `Charon's Ashes` | `CharonsAshes` | `RelicLibrary.initialize` / `RelicLibrary.add` | RARE; Ironclad-red; Shared or Ironclad-red rare relic pool; emit only entitled visible live instance/offer |
| `relic.dead_branch` | `Dead Branch` | `DeadBranch` | `RelicLibrary.initialize` / `RelicLibrary.add` | RARE; shared; Shared or Ironclad-red rare relic pool; emit only entitled visible live instance/offer |
| `relic.du_vu_doll` | `Du-Vu Doll` | `DuVuDoll` | `RelicLibrary.initialize` / `RelicLibrary.add` | RARE; shared; Shared or Ironclad-red rare relic pool; emit only entitled visible live instance/offer |
| `relic.fossilized_helix` | `FossilizedHelix` | — | `RelicLibrary.initialize` / `RelicLibrary.add` | RARE; shared; Shared or Ironclad-red rare relic pool; emit only entitled visible live instance/offer |
| `relic.gambling_chip` | `Gambling Chip` | `GamblingChip` | `RelicLibrary.initialize` / `RelicLibrary.add` | RARE; shared; Shared or Ironclad-red rare relic pool; emit only entitled visible live instance/offer |
| `relic.ginger` | `Ginger` | — | `RelicLibrary.initialize` / `RelicLibrary.add` | RARE; shared; Shared or Ironclad-red rare relic pool; emit only entitled visible live instance/offer |
| `relic.girya` | `Girya` | — | `RelicLibrary.initialize` / `RelicLibrary.add` | RARE; shared; Shared or Ironclad-red rare relic pool; emit only entitled visible live instance/offer |
| `relic.ice_cream` | `Ice Cream` | `IceCream` | `RelicLibrary.initialize` / `RelicLibrary.add` | RARE; shared; Shared or Ironclad-red rare relic pool; emit only entitled visible live instance/offer |
| `relic.incense_burner` | `Incense Burner` | `IncenseBurner` | `RelicLibrary.initialize` / `RelicLibrary.add` | RARE; shared; Shared or Ironclad-red rare relic pool; emit only entitled visible live instance/offer |
| `relic.lizard_tail` | `Lizard Tail` | `LizardTail` | `RelicLibrary.initialize` / `RelicLibrary.add` | RARE; shared; Shared or Ironclad-red rare relic pool; emit only entitled visible live instance/offer |
| `relic.magic_flower` | `Magic Flower` | `MagicFlower` | `RelicLibrary.initialize` / `RelicLibrary.add` | RARE; Ironclad-red; Shared or Ironclad-red rare relic pool; emit only entitled visible live instance/offer |
| `relic.mango` | `Mango` | — | `RelicLibrary.initialize` / `RelicLibrary.add` | RARE; shared; Shared or Ironclad-red rare relic pool; emit only entitled visible live instance/offer |
| `relic.old_coin` | `Old Coin` | `OldCoin` | `RelicLibrary.initialize` / `RelicLibrary.add` | RARE; shared; Shared or Ironclad-red rare relic pool; emit only entitled visible live instance/offer |
| `relic.peace_pipe` | `Peace Pipe` | `PeacePipe` | `RelicLibrary.initialize` / `RelicLibrary.add` | RARE; shared; Shared or Ironclad-red rare relic pool; emit only entitled visible live instance/offer |
| `relic.pocketwatch` | `Pocketwatch` | — | `RelicLibrary.initialize` / `RelicLibrary.add` | RARE; shared; Shared or Ironclad-red rare relic pool; emit only entitled visible live instance/offer |
| `relic.prayer_wheel` | `Prayer Wheel` | `PrayerWheel` | `RelicLibrary.initialize` / `RelicLibrary.add` | RARE; shared; Shared or Ironclad-red rare relic pool; emit only entitled visible live instance/offer |
| `relic.shovel` | `Shovel` | — | `RelicLibrary.initialize` / `RelicLibrary.add` | RARE; shared; Shared or Ironclad-red rare relic pool; emit only entitled visible live instance/offer |
| `relic.stone_calendar` | `StoneCalendar` | — | `RelicLibrary.initialize` / `RelicLibrary.add` | RARE; shared; Shared or Ironclad-red rare relic pool; emit only entitled visible live instance/offer |
| `relic.thread_and_needle` | `Thread and Needle` | `ThreadAndNeedle` | `RelicLibrary.initialize` / `RelicLibrary.add` | RARE; shared; Shared or Ironclad-red rare relic pool; emit only entitled visible live instance/offer |
| `relic.torii` | `Torii` | — | `RelicLibrary.initialize` / `RelicLibrary.add` | RARE; shared; Shared or Ironclad-red rare relic pool; emit only entitled visible live instance/offer |
| `relic.tungsten_rod` | `TungstenRod` | — | `RelicLibrary.initialize` / `RelicLibrary.add` | RARE; shared; Shared or Ironclad-red rare relic pool; emit only entitled visible live instance/offer |
| `relic.turnip` | `Turnip` | — | `RelicLibrary.initialize` / `RelicLibrary.add` | RARE; shared; Shared or Ironclad-red rare relic pool; emit only entitled visible live instance/offer |
| `relic.unceasing_top` | `Unceasing Top` | `UnceasingTop` | `RelicLibrary.initialize` / `RelicLibrary.add` | RARE; shared; Shared or Ironclad-red rare relic pool; emit only entitled visible live instance/offer |
| `relic.wing_boots` | `WingedGreaves` | `WingBoots` | `RelicLibrary.initialize` / `RelicLibrary.add` | RARE; shared; Shared or Ironclad-red rare relic pool; emit only entitled visible live instance/offer |
| `relic.astrolabe` | `Astrolabe` | — | `RelicLibrary.initialize` / `RelicLibrary.add` | BOSS; shared; Shared or Ironclad-red boss relic pool; eligible source predicates still apply; emit only entitled visible live instance/offer |
| `relic.black_blood` | `Black Blood` | `BlackBlood` | `RelicLibrary.initialize` / `RelicLibrary.add` | BOSS; Ironclad-red; Shared or Ironclad-red boss relic pool; eligible source predicates still apply; emit only entitled visible live instance/offer |
| `relic.black_star` | `Black Star` | `BlackStar` | `RelicLibrary.initialize` / `RelicLibrary.add` | BOSS; shared; Shared or Ironclad-red boss relic pool; eligible source predicates still apply; emit only entitled visible live instance/offer |
| `relic.busted_crown` | `Busted Crown` | `BustedCrown` | `RelicLibrary.initialize` / `RelicLibrary.add` | BOSS; shared; Shared or Ironclad-red boss relic pool; eligible source predicates still apply; emit only entitled visible live instance/offer |
| `relic.calling_bell` | `Calling Bell` | `CallingBell` | `RelicLibrary.initialize` / `RelicLibrary.add` | BOSS; shared; Shared or Ironclad-red boss relic pool; eligible source predicates still apply; emit only entitled visible live instance/offer |
| `relic.coffee_dripper` | `Coffee Dripper` | `CoffeeDripper` | `RelicLibrary.initialize` / `RelicLibrary.add` | BOSS; shared; Shared or Ironclad-red boss relic pool; eligible source predicates still apply; emit only entitled visible live instance/offer |
| `relic.cursed_key` | `Cursed Key` | `CursedKey` | `RelicLibrary.initialize` / `RelicLibrary.add` | BOSS; shared; Shared or Ironclad-red boss relic pool; eligible source predicates still apply; emit only entitled visible live instance/offer |
| `relic.ectoplasm` | `Ectoplasm` | — | `RelicLibrary.initialize` / `RelicLibrary.add` | BOSS; shared; Shared or Ironclad-red boss relic pool; eligible source predicates still apply; emit only entitled visible live instance/offer |
| `relic.empty_cage` | `Empty Cage` | `EmptyCage` | `RelicLibrary.initialize` / `RelicLibrary.add` | BOSS; shared; Shared or Ironclad-red boss relic pool; eligible source predicates still apply; emit only entitled visible live instance/offer |
| `relic.fusion_hammer` | `Fusion Hammer` | `FusionHammer` | `RelicLibrary.initialize` / `RelicLibrary.add` | BOSS; shared; Shared or Ironclad-red boss relic pool; eligible source predicates still apply; emit only entitled visible live instance/offer |
| `relic.mark_of_pain` | `Mark of Pain` | `MarkOfPain` | `RelicLibrary.initialize` / `RelicLibrary.add` | BOSS; Ironclad-red; Shared or Ironclad-red boss relic pool; eligible source predicates still apply; emit only entitled visible live instance/offer |
| `relic.pandoras_box` | `Pandora's Box` | `PandorasBox` | `RelicLibrary.initialize` / `RelicLibrary.add` | BOSS; shared; Shared or Ironclad-red boss relic pool; eligible source predicates still apply; emit only entitled visible live instance/offer |
| `relic.philosopher_stone` | `Philosopher's Stone` | `PhilosopherStone` | `RelicLibrary.initialize` / `RelicLibrary.add` | BOSS; shared; Shared or Ironclad-red boss relic pool; eligible source predicates still apply; emit only entitled visible live instance/offer |
| `relic.runic_cube` | `Runic Cube` | `RunicCube` | `RelicLibrary.initialize` / `RelicLibrary.add` | BOSS; Ironclad-red; Shared or Ironclad-red boss relic pool; eligible source predicates still apply; emit only entitled visible live instance/offer |
| `relic.runic_dome` | `Runic Dome` | `RunicDome` | `RelicLibrary.initialize` / `RelicLibrary.add` | BOSS; shared; Shared or Ironclad-red boss relic pool; eligible source predicates still apply; emit only entitled visible live instance/offer |
| `relic.runic_pyramid` | `Runic Pyramid` | `RunicPyramid` | `RelicLibrary.initialize` / `RelicLibrary.add` | BOSS; shared; Shared or Ironclad-red boss relic pool; eligible source predicates still apply; emit only entitled visible live instance/offer |
| `relic.sacred_bark` | `SacredBark` | — | `RelicLibrary.initialize` / `RelicLibrary.add` | BOSS; shared; Shared or Ironclad-red boss relic pool; eligible source predicates still apply; emit only entitled visible live instance/offer |
| `relic.slavers_collar` | `SlaversCollar` | — | `RelicLibrary.initialize` / `RelicLibrary.add` | BOSS; shared; Shared or Ironclad-red boss relic pool; eligible source predicates still apply; emit only entitled visible live instance/offer |
| `relic.snecko_eye` | `Snecko Eye` | `SneckoEye` | `RelicLibrary.initialize` / `RelicLibrary.add` | BOSS; shared; Shared or Ironclad-red boss relic pool; eligible source predicates still apply; emit only entitled visible live instance/offer |
| `relic.sozu` | `Sozu` | — | `RelicLibrary.initialize` / `RelicLibrary.add` | BOSS; shared; Shared or Ironclad-red boss relic pool; eligible source predicates still apply; emit only entitled visible live instance/offer |
| `relic.tiny_house` | `Tiny House` | `TinyHouse` | `RelicLibrary.initialize` / `RelicLibrary.add` | BOSS; shared; Shared or Ironclad-red boss relic pool; eligible source predicates still apply; emit only entitled visible live instance/offer |
| `relic.velvet_choker` | `Velvet Choker` | `VelvetChoker` | `RelicLibrary.initialize` / `RelicLibrary.add` | BOSS; shared; Shared or Ironclad-red boss relic pool; eligible source predicates still apply; emit only entitled visible live instance/offer |
| `relic.brimstone` | `Brimstone` | — | `RelicLibrary.initialize` / `RelicLibrary.add` | SHOP; Ironclad-red; Shared or Ironclad-red shop relic pool; shop-specific exclusions still apply; emit only entitled visible live instance/offer |
| `relic.cauldron` | `Cauldron` | — | `RelicLibrary.initialize` / `RelicLibrary.add` | SHOP; shared; Shared or Ironclad-red shop relic pool; shop-specific exclusions still apply; emit only entitled visible live instance/offer |
| `relic.chemical_x` | `Chemical X` | `ChemicalX` | `RelicLibrary.initialize` / `RelicLibrary.add` | SHOP; shared; Shared or Ironclad-red shop relic pool; shop-specific exclusions still apply; emit only entitled visible live instance/offer |
| `relic.clockwork_souvenir` | `ClockworkSouvenir` | — | `RelicLibrary.initialize` / `RelicLibrary.add` | SHOP; shared; Shared or Ironclad-red shop relic pool; shop-specific exclusions still apply; emit only entitled visible live instance/offer |
| `relic.dollys_mirror` | `DollysMirror` | — | `RelicLibrary.initialize` / `RelicLibrary.add` | SHOP; shared; Shared or Ironclad-red shop relic pool; shop-specific exclusions still apply; emit only entitled visible live instance/offer |
| `relic.frozen_eye` | `Frozen Eye` | `FrozenEye` | `RelicLibrary.initialize` / `RelicLibrary.add` | SHOP; shared; Shared or Ironclad-red shop relic pool; shop-specific exclusions still apply; emit only entitled visible live instance/offer |
| `relic.hand_drill` | `HandDrill` | — | `RelicLibrary.initialize` / `RelicLibrary.add` | SHOP; shared; Shared or Ironclad-red shop relic pool; shop-specific exclusions still apply; emit only entitled visible live instance/offer |
| `relic.waffle` | `Lee's Waffle` | `Waffle` | `RelicLibrary.initialize` / `RelicLibrary.add` | SHOP; shared; Shared or Ironclad-red shop relic pool; shop-specific exclusions still apply; emit only entitled visible live instance/offer |
| `relic.medical_kit` | `Medical Kit` | `MedicalKit` | `RelicLibrary.initialize` / `RelicLibrary.add` | SHOP; shared; Shared or Ironclad-red shop relic pool; shop-specific exclusions still apply; emit only entitled visible live instance/offer |
| `relic.membership_card` | `Membership Card` | `MembershipCard` | `RelicLibrary.initialize` / `RelicLibrary.add` | SHOP; shared; Shared or Ironclad-red shop relic pool; shop-specific exclusions still apply; emit only entitled visible live instance/offer |
| `relic.orange_pellets` | `OrangePellets` | — | `RelicLibrary.initialize` / `RelicLibrary.add` | SHOP; shared; Shared or Ironclad-red shop relic pool; shop-specific exclusions still apply; emit only entitled visible live instance/offer |
| `relic.orrery` | `Orrery` | — | `RelicLibrary.initialize` / `RelicLibrary.add` | SHOP; shared; Shared or Ironclad-red shop relic pool; shop-specific exclusions still apply; emit only entitled visible live instance/offer |
| `relic.prismatic_shard` | `PrismaticShard` | — | `RelicLibrary.initialize` / `RelicLibrary.add` | SHOP; shared; Shared or Ironclad-red shop relic pool; shop-specific exclusions still apply; emit only entitled visible live instance/offer |
| `relic.sling` | `Sling` | — | `RelicLibrary.initialize` / `RelicLibrary.add` | SHOP; shared; Shared or Ironclad-red shop relic pool; shop-specific exclusions still apply; emit only entitled visible live instance/offer |
| `relic.strange_spoon` | `Strange Spoon` | `StrangeSpoon` | `RelicLibrary.initialize` / `RelicLibrary.add` | SHOP; shared; Shared or Ironclad-red shop relic pool; shop-specific exclusions still apply; emit only entitled visible live instance/offer |
| `relic.abacus` | `TheAbacus` | `Abacus` | `RelicLibrary.initialize` / `RelicLibrary.add` | SHOP; shared; Shared or Ironclad-red shop relic pool; shop-specific exclusions still apply; emit only entitled visible live instance/offer |
| `relic.toolbox` | `Toolbox` | — | `RelicLibrary.initialize` / `RelicLibrary.add` | SHOP; shared; Shared or Ironclad-red shop relic pool; shop-specific exclusions still apply; emit only entitled visible live instance/offer |
| `relic.neows_lament` | `NeowsBlessing` | `NeowsLament` | `RelicLibrary.initialize` / `RelicLibrary.add` | SPECIAL; shared-special; Neow opening blessing offer; emit only entitled visible live instance/offer |
| `relic.golden_idol` | `Golden Idol` | `GoldenIdol` | `RelicLibrary.initialize` / `RelicLibrary.add` | SPECIAL; shared-special; Golden Idol Act 1 event; emit only entitled visible live instance/offer |
| `relic.odd_mushroom` | `Odd Mushroom` | `OddMushroom` | `RelicLibrary.initialize` / `RelicLibrary.add` | SPECIAL; shared-special; Hypnotizing Colored Mushrooms event; emit only entitled visible live instance/offer |
| `relic.warped_tongs` | `WarpedTongs` | — | `RelicLibrary.initialize` / `RelicLibrary.add` | SPECIAL; shared-special; Ominous Forge event; emit only entitled visible live instance/offer |
| `relic.spirit_poop` | `Spirit Poop` | `SpiritPoop` | `RelicLibrary.initialize` / `RelicLibrary.add` | SPECIAL; shared-special; Bonfire Spirits event; emit only entitled visible live instance/offer |
| `relic.face_of_cleric` | `FaceOfCleric` | — | `RelicLibrary.initialize` / `RelicLibrary.add` | SPECIAL; shared-special; Face Trader event; emit only entitled visible live instance/offer |
| `relic.cultist_mask` | `CultistMask` | — | `RelicLibrary.initialize` / `RelicLibrary.add` | SPECIAL; shared-special; Face Trader event; emit only entitled visible live instance/offer |
| `relic.gremlin_mask` | `GremlinMask` | — | `RelicLibrary.initialize` / `RelicLibrary.add` | SPECIAL; shared-special; Face Trader event; emit only entitled visible live instance/offer |
| `relic.nloths_mask` | `NlothsMask` | — | `RelicLibrary.initialize` / `RelicLibrary.add` | SPECIAL; shared-special; Face Trader event; emit only entitled visible live instance/offer |
| `relic.ssserpent_head` | `SsserpentHead` | — | `RelicLibrary.initialize` / `RelicLibrary.add` | SPECIAL; shared-special; Face Trader event (one of five possible faces); emit only entitled visible live instance/offer |

The source-facing aliases are exact and case-sensitive; display labels are explanatory only and are not identity aliases. `STARTER`, shared, character-red, boss, shop and special/event routes remain distinct eligibility classes. Possession does not imply offerability, and `canSpawn`/event-specific predicates do not create a generic emission entitlement.

### 2.3 Ironclad potion factory identity and eligibility catalog

This is the exact 33-ID Ironclad potion list from the retained `PotionHelper.getPotions` source list, joined to the `getPotion` factory case and to the accepted tier/potion record. The three Ironclad-only factory IDs are BloodPotion, ElixirPotion and HeartOfIron; the nine exclusive IDs for the other three characters are not in the Ironclad set. The registry owner does not grant an offer or disclose a hidden potion; an identity is emitted only for a public live belt slot or revealed offer/reward. Potion source law is route-specific: the accepted potion research specifies rarity-first and flat helpers, and Fruit Juice is excluded from limited generation while remaining in the unrestricted pool.

| Canonical key | Exact factory/source ID | Exact class alias | Factory owner | Tier / eligibility |
|---|---|---|---|---|
| `potion.blockpotion` | `Block Potion` | `BlockPotion` | `PotionHelper.getPotions(IRONCLAD,false)` → `getPotion` | COMMON; common rarity; generate only at an entitled drop/reward/offer or eligible use; emit identity only when its live slot/offer is public |
| `potion.dexteritypotion` | `Dexterity Potion` | `DexterityPotion` | `PotionHelper.getPotions(IRONCLAD,false)` → `getPotion` | COMMON; common rarity; generate only at an entitled drop/reward/offer or eligible use; emit identity only when its live slot/offer is public |
| `potion.energypotion` | `Energy Potion` | `EnergyPotion` | `PotionHelper.getPotions(IRONCLAD,false)` → `getPotion` | COMMON; common rarity; generate only at an entitled drop/reward/offer or eligible use; emit identity only when its live slot/offer is public |
| `potion.explosivepotion` | `Explosive Potion` | `ExplosivePotion` | `PotionHelper.getPotions(IRONCLAD,false)` → `getPotion` | COMMON; common rarity; generate only at an entitled drop/reward/offer or eligible use; emit identity only when its live slot/offer is public |
| `potion.firepotion` | `Fire Potion` | `FirePotion` | `PotionHelper.getPotions(IRONCLAD,false)` → `getPotion` | COMMON; common rarity; generate only at an entitled drop/reward/offer or eligible use; emit identity only when its live slot/offer is public |
| `potion.strengthpotion` | `Strength Potion` | `StrengthPotion` | `PotionHelper.getPotions(IRONCLAD,false)` → `getPotion` | COMMON; common rarity; generate only at an entitled drop/reward/offer or eligible use; emit identity only when its live slot/offer is public |
| `potion.swiftpotion` | `Swift Potion` | `SwiftPotion` | `PotionHelper.getPotions(IRONCLAD,false)` → `getPotion` | COMMON; common rarity; generate only at an entitled drop/reward/offer or eligible use; emit identity only when its live slot/offer is public |
| `potion.weakenpotion` | `Weak Potion` | `WeakenPotion` | `PotionHelper.getPotions(IRONCLAD,false)` → `getPotion` | COMMON; common rarity; generate only at an entitled drop/reward/offer or eligible use; emit identity only when its live slot/offer is public |
| `potion.fearpotion` | `FearPotion` | — | `PotionHelper.getPotions(IRONCLAD,false)` → `getPotion` | COMMON; common rarity; generate only at an entitled drop/reward/offer or eligible use; emit identity only when its live slot/offer is public |
| `potion.attackpotion` | `AttackPotion` | — | `PotionHelper.getPotions(IRONCLAD,false)` → `getPotion` | COMMON; common rarity; generate only at an entitled drop/reward/offer or eligible use; emit identity only when its live slot/offer is public |
| `potion.skillpotion` | `SkillPotion` | — | `PotionHelper.getPotions(IRONCLAD,false)` → `getPotion` | COMMON; common rarity; generate only at an entitled drop/reward/offer or eligible use; emit identity only when its live slot/offer is public |
| `potion.powerpotion` | `PowerPotion` | — | `PotionHelper.getPotions(IRONCLAD,false)` → `getPotion` | COMMON; common rarity; generate only at an entitled drop/reward/offer or eligible use; emit identity only when its live slot/offer is public |
| `potion.colorlesspotion` | `ColorlessPotion` | — | `PotionHelper.getPotions(IRONCLAD,false)` → `getPotion` | COMMON; common rarity; generate only at an entitled drop/reward/offer or eligible use; emit identity only when its live slot/offer is public |
| `potion.steroidpotion` | `SteroidPotion` | — | `PotionHelper.getPotions(IRONCLAD,false)` → `getPotion` | COMMON; common rarity; generate only at an entitled drop/reward/offer or eligible use; emit identity only when its live slot/offer is public |
| `potion.speedpotion` | `SpeedPotion` | — | `PotionHelper.getPotions(IRONCLAD,false)` → `getPotion` | COMMON; common rarity; generate only at an entitled drop/reward/offer or eligible use; emit identity only when its live slot/offer is public |
| `potion.blessingoftheforge` | `BlessingOfTheForge` | — | `PotionHelper.getPotions(IRONCLAD,false)` → `getPotion` | COMMON; common rarity; generate only at an entitled drop/reward/offer or eligible use; emit identity only when its live slot/offer is public |
| `potion.bloodpotion` | `BloodPotion` | — | `PotionHelper.getPotions(IRONCLAD,false)` → `getPotion` | COMMON; common rarity; generate only at an entitled drop/reward/offer or eligible use; emit identity only when its live slot/offer is public |
| `potion.regenpotion` | `Regen Potion` | `RegenPotion` | `PotionHelper.getPotions(IRONCLAD,false)` → `getPotion` | UNCOMMON; uncommon rarity; generate only at an entitled drop/reward/offer or eligible use; emit identity only when its live slot/offer is public |
| `potion.ancientpotion` | `Ancient Potion` | `AncientPotion` | `PotionHelper.getPotions(IRONCLAD,false)` → `getPotion` | UNCOMMON; uncommon rarity; generate only at an entitled drop/reward/offer or eligible use; emit identity only when its live slot/offer is public |
| `potion.liquidbronze` | `LiquidBronze` | — | `PotionHelper.getPotions(IRONCLAD,false)` → `getPotion` | UNCOMMON; uncommon rarity; generate only at an entitled drop/reward/offer or eligible use; emit identity only when its live slot/offer is public |
| `potion.gamblersbrew` | `GamblersBrew` | — | `PotionHelper.getPotions(IRONCLAD,false)` → `getPotion` | UNCOMMON; uncommon rarity; generate only at an entitled drop/reward/offer or eligible use; emit identity only when its live slot/offer is public |
| `potion.essenceofsteel` | `EssenceOfSteel` | — | `PotionHelper.getPotions(IRONCLAD,false)` → `getPotion` | UNCOMMON; uncommon rarity; generate only at an entitled drop/reward/offer or eligible use; emit identity only when its live slot/offer is public |
| `potion.duplicationpotion` | `DuplicationPotion` | — | `PotionHelper.getPotions(IRONCLAD,false)` → `getPotion` | UNCOMMON; uncommon rarity; generate only at an entitled drop/reward/offer or eligible use; emit identity only when its live slot/offer is public |
| `potion.distilledchaospotion` | `DistilledChaos` | `DistilledChaosPotion` | `PotionHelper.getPotions(IRONCLAD,false)` → `getPotion` | UNCOMMON; uncommon rarity; generate only at an entitled drop/reward/offer or eligible use; emit identity only when its live slot/offer is public |
| `potion.liquidmemories` | `LiquidMemories` | — | `PotionHelper.getPotions(IRONCLAD,false)` → `getPotion` | UNCOMMON; uncommon rarity; generate only at an entitled drop/reward/offer or eligible use; emit identity only when its live slot/offer is public |
| `potion.elixir` | `ElixirPotion` | `Elixir` | `PotionHelper.getPotions(IRONCLAD,false)` → `getPotion` | UNCOMMON; uncommon rarity; generate only at an entitled drop/reward/offer or eligible use; emit identity only when its live slot/offer is public |
| `potion.cultistpotion` | `CultistPotion` | — | `PotionHelper.getPotions(IRONCLAD,false)` → `getPotion` | RARE; rare rarity; generate only at an entitled drop/reward/offer or eligible use; emit identity only when its live slot/offer is public |
| `potion.fruitjuice` | `Fruit Juice` | `FruitJuice` | `PotionHelper.getPotions(IRONCLAD,false)` → `getPotion` | RARE; rare rarity; generate only at an entitled drop/reward/offer or eligible use; emit identity only when its live slot/offer is public |
| `potion.sneckooil` | `SneckoOil` | — | `PotionHelper.getPotions(IRONCLAD,false)` → `getPotion` | RARE; rare rarity; generate only at an entitled drop/reward/offer or eligible use; emit identity only when its live slot/offer is public |
| `potion.fairypotion` | `FairyPotion` | — | `PotionHelper.getPotions(IRONCLAD,false)` → `getPotion` | RARE; rare rarity; generate only at an entitled drop/reward/offer or eligible use; emit identity only when its live slot/offer is public |
| `potion.smokebomb` | `SmokeBomb` | — | `PotionHelper.getPotions(IRONCLAD,false)` → `getPotion` | RARE; rare rarity; generate only at an entitled drop/reward/offer or eligible use; emit identity only when its live slot/offer is public |
| `potion.entropicbrew` | `EntropicBrew` | — | `PotionHelper.getPotions(IRONCLAD,false)` → `getPotion` | RARE; rare rarity; generate only at an entitled drop/reward/offer or eligible use; emit identity only when its live slot/offer is public |
| `potion.heartofiron` | `HeartOfIron` | — | `PotionHelper.getPotions(IRONCLAD,false)` → `getPotion` | RARE; rare rarity; generate only at an entitled drop/reward/offer or eligible use; emit identity only when its live slot/offer is public |

Factory IDs and class aliases are exact case-sensitive source names. `getPotions` membership is the constructor identity set, not an assertion that every producer emits every entry: rarity filters, the fixed player-class authority and Fruit Juice limited-helper exclusion continue to apply. `Entropic Brew` uses the actual class/use phase and must not be inferred from potion ID alone.

### 2.4 Act 1 event-kind identities

These are the 25 event identities in the accepted Act 1 event records, joined to the exact `EventHelper.getEvent` switch key and constructed Java class alias. The event registry includes events outside this Act 1 set; only these 25 accepted Act 1 records are listed here. Public emission is current-event scoped: emit an event key only for the active revealed event; emit a choice only when that event exposes the corresponding current option. An event-kind registry row does not authorize future-room or hidden choice disclosure.

| Canonical key | Exact source event key | Exact class alias | Lifecycle/selection owner | Public eligibility |
|---|---|---|---|---|
| `event.big_fish` | `Big Fish` | `BigFish` | `EventHelper.getEvent` | Act 1 route only; active revealed event/current visible choices |
| `event.dead_adventurer` | `Dead Adventurer` | `DeadAdventurer` | `EventHelper.getEvent` | Act 1 route only; active revealed event/current visible choices |
| `event.golden_idol` | `Golden Idol` | `GoldenIdolEvent` | `EventHelper.getEvent` | Act 1 route only; active revealed event/current visible choices |
| `event.mushrooms` | `Mushrooms` | `Mushrooms` | `EventHelper.getEvent` | Act 1 route only; active revealed event/current visible choices |
| `event.living_wall` | `Living Wall` | `LivingWall` | `EventHelper.getEvent` | Act 1 route only; active revealed event/current visible choices |
| `event.scrap_ooze` | `Scrap Ooze` | `ScrapOoze` | `EventHelper.getEvent` | Act 1 route only; active revealed event/current visible choices |
| `event.shining_light` | `Shining Light` | `ShiningLight` | `EventHelper.getEvent` | Act 1 route only; active revealed event/current visible choices |
| `event.the_cleric` | `The Cleric` | `Cleric` | `EventHelper.getEvent` | Act 1 route only; active revealed event/current visible choices |
| `event.liars_game` | `Liars Game` | `Sssserpent` | `EventHelper.getEvent` | Act 1 route only; active revealed event/current visible choices |
| `event.golden_wing` | `Golden Wing` | `GoldenWing` | `EventHelper.getEvent` | Act 1 route only; active revealed event/current visible choices |
| `event.world_of_goop` | `World of Goop` | `GoopPuddle` | `EventHelper.getEvent` | Act 1 route only; active revealed event/current visible choices |
| `event.golden_shrine` | `Golden Shrine` | `GoldShrine` | `EventHelper.getEvent` | Act 1 route only; active revealed event/current visible choices |
| `event.match_and_keep` | `Match and Keep!` | `GremlinMatchGame` | `EventHelper.getEvent` | Act 1 route only; active revealed event/current visible choices |
| `event.purifier` | `Purifier` | `PurificationShrine` | `EventHelper.getEvent` | Act 1 route only; active revealed event/current visible choices |
| `event.transmorgrifier` | `Transmorgrifier` | `Transmogrifier` | `EventHelper.getEvent` | Act 1 route only; active revealed event/current visible choices |
| `event.upgrade_shrine` | `Upgrade Shrine` | `UpgradeShrine` | `EventHelper.getEvent` | Act 1 route only; active revealed event/current visible choices |
| `event.wheel_of_change` | `Wheel of Change` | `GremlinWheelGame` | `EventHelper.getEvent` | Act 1 route only; active revealed event/current visible choices |
| `event.accursed_blacksmith` | `Accursed Blacksmith` | `AccursedBlacksmith` | `EventHelper.getEvent` | Act 1 route only; active revealed event/current visible choices |
| `event.bonfire_elementals` | `Bonfire Elementals` | `Bonfire` | `EventHelper.getEvent` | Act 1 route only; active revealed event/current visible choices |
| `event.face_trader` | `FaceTrader` | `FaceTrader` | `EventHelper.getEvent` | Act 1 route only; active revealed event/current visible choices |
| `event.fountain_of_cleansing` | `Fountain of Cleansing` | `FountainOfCurseRemoval` | `EventHelper.getEvent` | Act 1 route only; active revealed event/current visible choices |
| `event.lab` | `Lab` | `Lab` | `EventHelper.getEvent` | Act 1 route only; active revealed event/current visible choices |
| `event.note_for_yourself` | `NoteForYourself` | `NoteForYourself` | `EventHelper.getEvent` | Act 1 route only; active revealed event/current visible choices |
| `event.we_meet_again` | `WeMeetAgain` | `WeMeetAgain` | `EventHelper.getEvent` | Act 1 route only; active revealed event/current visible choices |
| `event.the_woman_in_blue` | `The Woman in Blue` | `WomanInBlue` | `EventHelper.getEvent` | Act 1 route only; active revealed event/current visible choices |

Event choices are not globally numbered or keyed by localized labels. The selection owner is the concrete event class and its current option set. Choice-site exact keys and the full source-options set remain residual; do not infer choice IDs from row order or UI text.

### 2.5 Encounter identities and route-qualified factories

The source encounter ID is the exact `MonsterHelper` string constant. An encounter key is a factory/formation identity, never a monster kind: repeated actors retain separate actor identities under the same monster-kind key. These rows cover the scoped ordinary Act 1 families, three bosses, and the two event-specific factory variants named by accepted records. The same Nob/Sentries keys may be reached through Dead Adventurer, but that event-owned combat does not consume ordinary elite queue state. Emit an encounter key only at an entitled active public formation/checkpoint; never reveal future queued encounter identities.

| Canonical key | Exact source encounter key | Exact source constant alias | Factory/route owner | Public eligibility |
|---|---|---|---|---|
| `encounter.blue_slaver` | `Blue Slaver` | `BLUE_SLAVER_ENC` | `MonsterHelper.BLUE_SLAVER_ENC`; ordinary strong | Active current formation only; future queue hidden |
| `encounter.cultist` | `Cultist` | `CULTIST_ENC` | `MonsterHelper.CULTIST_ENC`; ordinary weak | Active current formation only; future queue hidden |
| `encounter.jaw_worm` | `Jaw Worm` | `JAW_WORM_ENC` | `MonsterHelper.JAW_WORM_ENC`; ordinary weak | Active current formation only; future queue hidden |
| `encounter.looter` | `Looter` | `LOOTER_ENC` | `MonsterHelper.LOOTER_ENC`; ordinary strong | Active current formation only; future queue hidden |
| `encounter.2_louse` | `2 Louse` | `TWO_LOUSE_ENC` | `MonsterHelper.TWO_LOUSE_ENC`; ordinary weak | Active current formation only; future queue hidden |
| `encounter.small_slimes` | `Small Slimes` | `SMALL_SLIMES_ENC` | `MonsterHelper.SMALL_SLIMES_ENC`; ordinary weak | Active current formation only; future queue hidden |
| `encounter.gremlin_gang` | `Gremlin Gang` | `GREMLIN_GANG_ENC` | `MonsterHelper.GREMLIN_GANG_ENC`; ordinary strong | Active current formation only; future queue hidden |
| `encounter.red_slaver` | `Red Slaver` | `RED_SLAVER_ENC` | `MonsterHelper.RED_SLAVER_ENC`; ordinary strong | Active current formation only; future queue hidden |
| `encounter.large_slime` | `Large Slime` | `LARGE_SLIME_ENC` | `MonsterHelper.LARGE_SLIME_ENC`; ordinary strong | Active current formation only; future queue hidden |
| `encounter.exordium_thugs` | `Exordium Thugs` | `LVL_1_THUGS_ENC` | `MonsterHelper.LVL_1_THUGS_ENC`; ordinary strong | Active current formation only; future queue hidden |
| `encounter.exordium_wildlife` | `Exordium Wildlife` | `LVL_1_WILDLIFE_ENC` | `MonsterHelper.LVL_1_WILDLIFE_ENC`; ordinary strong | Active current formation only; future queue hidden |
| `encounter.3_louse` | `3 Louse` | `THREE_LOUSE_ENC` | `MonsterHelper.THREE_LOUSE_ENC`; ordinary strong | Active current formation only; future queue hidden |
| `encounter.2_fungi_beasts` | `2 Fungi Beasts` | `TWO_FUNGI_ENC` | `MonsterHelper.TWO_FUNGI_ENC`; ordinary strong | Active current formation only; future queue hidden |
| `encounter.lots_of_slimes` | `Lots of Slimes` | `LOTS_OF_SLIMES_ENC` | `MonsterHelper.LOTS_OF_SLIMES_ENC`; ordinary strong | Active current formation only; future queue hidden |
| `encounter.gremlin_nob` | `Gremlin Nob` | `GREMLIN_NOB_ENC` | `MonsterHelper.GREMLIN_NOB_ENC`; ordinary elite; also Dead Adventurer event | Active current formation only; future queue hidden |
| `encounter.lagavulin` | `Lagavulin` | `LAGAVULIN_ENC` | `MonsterHelper.LAGAVULIN_ENC`; ordinary elite | Active current formation only; future queue hidden |
| `encounter.3_sentries` | `3 Sentries` | `THREE_SENTRY_ENC` | `MonsterHelper.THREE_SENTRY_ENC`; ordinary elite; also Dead Adventurer event | Active current formation only; future queue hidden |
| `encounter.the_guardian` | `The Guardian` | `GUARDIAN_ENC` | `MonsterHelper.GUARDIAN_ENC`; Act1 boss | Active current formation only; future queue hidden |
| `encounter.hexaghost` | `Hexaghost` | `HEXAGHOST_ENC` | `MonsterHelper.HEXAGHOST_ENC`; Act1 boss | Active current formation only; future queue hidden |
| `encounter.slime_boss` | `Slime Boss` | `SLIME_BOSS_ENC` | `MonsterHelper.SLIME_BOSS_ENC`; Act1 boss | Active current formation only; future queue hidden |
| `encounter.lagavulin_event` | `Lagavulin Event` | `LAGAVULIN_EVENT_ENC` | `MonsterHelper.LAGAVULIN_EVENT_ENC`; Dead Adventurer event-only variant | Active current formation only; future queue hidden |
| `encounter.the_mushroom_lair` | `The Mushroom Lair` | `MUSHROOMS_EVENT_ENC` | `MonsterHelper.MUSHROOMS_EVENT_ENC`; Mushrooms event combat | Active current formation only; future queue hidden |

Exact ordered member composition and route selection laws are in the accepted enemy/encounter records; these canonical keys do not replace formation member identities or per-actor state.
### 2.6 Monster-kind identity catalog

This 25-row local enemy-kind catalog binds exact case-sensitive static class `ID` constants to the exact class alias and current encounter/use owner. Entries are sourced from unique class entries in the pinned archive; class ID/alias are not a display-name substitute. Split/summon dependents remain monster kinds; their encounter key is separately catalogued above. Emit a monster key only for an actor in the public current formation, and emit it with actor-local state/reference; a future encounter queue does not reveal a monster kind.

| Canonical key | Exact source ID | Exact class alias | Owner / route | Public eligibility |
|---|---|---|---|---|
| `monster.cultist` | `Cultist` | `Cultist` | Act1 encounter/elite/boss factory as listed in accepted records | Current public formation actor only |
| `monster.jaw_worm` | `JawWorm` | `JawWorm` | Act1 encounter/elite/boss factory as listed in accepted records | Current public formation actor only |
| `monster.red_louse` | `FuzzyLouseNormal` | `LouseNormal` | Act1 encounter/elite/boss factory as listed in accepted records | Current public formation actor only |
| `monster.green_louse` | `FuzzyLouseDefensive` | `LouseDefensive` | Act1 encounter/elite/boss factory as listed in accepted records | Current public formation actor only |
| `monster.acid_slime_s` | `AcidSlime_S` | `AcidSlime_S` | Act1 encounter | Current public formation actor only |
| `monster.acid_slime_m` | `AcidSlime_M` | `AcidSlime_M` | Act1 encounter/split | Current public formation actor only |
| `monster.acid_slime_l` | `AcidSlime_L` | `AcidSlime_L` | Act1 encounter/split or Slime Boss summon | Current public formation actor only |
| `monster.spike_slime_s` | `SpikeSlime_S` | `SpikeSlime_S` | Act1 encounter/elite/boss factory as listed in accepted records | Current public formation actor only |
| `monster.spike_slime_m` | `SpikeSlime_M` | `SpikeSlime_M` | Act1 encounter/split | Current public formation actor only |
| `monster.spike_slime_l` | `SpikeSlime_L` | `SpikeSlime_L` | Act1 encounter/elite/boss factory as listed in accepted records | Current public formation actor only |
| `monster.blue_slaver` | `SlaverBlue` | `SlaverBlue` | Act1 encounter/elite/boss factory as listed in accepted records | Current public formation actor only |
| `monster.red_slaver` | `SlaverRed` | `SlaverRed` | Act1 encounter/elite/boss factory as listed in accepted records | Current public formation actor only |
| `monster.looter` | `Looter` | `Looter` | Act1 encounter/elite/boss factory as listed in accepted records | Current public formation actor only |
| `monster.fungi_beast` | `FungiBeast` | `FungiBeast` | Act1 encounter/elite/boss factory as listed in accepted records | Current public formation actor only |
| `monster.fat_gremlin` | `GremlinFat` | `GremlinFat` | Act1 encounter/elite/boss factory as listed in accepted records | Current public formation actor only |
| `monster.mad_gremlin` | `GremlinWarrior` | `GremlinWarrior` | Act1 encounter/elite/boss factory as listed in accepted records | Current public formation actor only |
| `monster.sneaky_gremlin` | `GremlinThief` | `GremlinThief` | Act1 encounter/elite/boss factory as listed in accepted records | Current public formation actor only |
| `monster.shield_gremlin` | `GremlinTsundere` | `GremlinTsundere` | Act1 encounter/elite/boss factory as listed in accepted records | Current public formation actor only |
| `monster.gremlin_wizard` | `GremlinWizard` | `GremlinWizard` | Act1 encounter/elite/boss factory as listed in accepted records | Current public formation actor only |
| `monster.gremlin_nob` | `GremlinNob` | `GremlinNob` | Act1 elite or Dead Adventurer event | Current public formation actor only |
| `monster.lagavulin` | `Lagavulin` | `Lagavulin` | Act1 encounter/elite/boss factory as listed in accepted records | Current public formation actor only |
| `monster.sentry` | `Sentry` | `Sentry` | Act1 encounter or event combat | Current public formation actor only |
| `monster.slime_boss` | `SlimeBoss` | `SlimeBoss` | Act1 encounter/elite/boss factory as listed in accepted records | Current public formation actor only |
| `monster.hexaghost` | `Hexaghost` | `Hexaghost` | Act1 encounter/elite/boss factory as listed in accepted records | Current public formation actor only |
| `monster.the_guardian` | `TheGuardian` | `TheGuardian` | Act1 encounter/elite/boss factory as listed in accepted records | Current public formation actor only |

The two Louse class aliases intentionally differ from their IDs: `LouseNormal`→`FuzzyLouseNormal` and `LouseDefensive`→`FuzzyLouseDefensive`. Monster move bytes remain local to each class and are not part of this kind catalog.

### 2.7 Property-code proposal vocabulary (native-only; not source identities)

The following labels and numbers are the finite **native proposal** in accepted E29, not Java/source property IDs and not independently proven source-owner bindings. Codes are version-local proposal values, not wire numbers assigned here. The source-owner/member and reveal predicate must still be supplied before emission. Code 41 has no source owner proof and is excluded/unemittable; code 40 is usable only for separately registered owner/subkey pairs and is not an escape hatch.

| Proposed code | Native-only proposal key | Source alias/owner status |
|---:|---|---|
| 1 | `base_damage` | No source property ID; owner/member join required |
| 2 | `displayed_damage` | No source property ID; owner/member join required |
| 3 | `base_block` | No source property ID; owner/member join required |
| 4 | `displayed_block` | No source property ID; owner/member join required |
| 5 | `magic` | No source property ID; owner/member join required |
| 6 | `captured_x` | No source property ID; exact producer required |
| 7 | `cost_until_played` | No source property ID; owner/member join required |
| 8 | `cost_combat` | No source property ID; owner/member join required |
| 9 | `cost_free_once` | No source property ID; owner/member join required |
| 10 | `autoplay` | No source property ID; owner/member join required |
| 11 | `ignore_total` | No source property ID; owner/member join required |
| 12 | `purge` | No source property ID; owner/member join required |
| 13 | `public_source_site` | Native proposal only; exact producer/source-site join required |
| 14 | `public_copy_origin` | Native proposal only; exact copy source/lifetime join required |
| 15 | `retained_duration` | No source property ID; owner/member join required |
| 16 | `delayed_duration` | No source property ID; owner/member join required |
| 17 | `uses` | No source property ID; owner/member join required |
| 18 | `gold_price` | No source property ID; exact settled-price owner required |
| 19 | `hp_price` | No source property ID; exact payment/owner required |
| 20 | `max_hp_price` | No source property ID; exact payment/owner required |
| 21 | `selected_order` | Native proposal only; exact source selection owner required |
| 22 | `can_skip` | No source property ID; current option owner required |
| 23 | `can_confirm` | No source property ID; current option owner required |
| 24 | `scry_count` | No source property ID; current owner/checkpoint required |
| 25 | `public_discard_count` | Native proposal only; reveal checkpoint required |
| 26 | `public_hp_loss_count` | Native proposal only; reveal checkpoint required |
| 27 | `public_play_count` | Native proposal only; reveal checkpoint required |
| 28 | `public_power_count` | Native proposal only; reveal checkpoint required |
| 29 | `mantra` | No source property ID; owner/member join required |
| 30 | `orb_slots` | Native proposal only; owner/member join required |
| 31 | `focus` | No source property ID; owner/member join required |
| 32 | `kill_growth` | Native proposal only; exact source producer required |
| 33 | `potion_capacity` | No source property ID; owner/member join required |
| 34 | `key_kind` | Native proposal only; keep node marker distinct from acquired key |
| 35 | `burning_buff_public` | Native proposal only; current public predicate required |
| 36 | `source_option_stage` | Native proposal only; event/option owner join required |
| 37 | `draw_order_known` | Native knowledge proposal; reveal condition required |
| 38 | `outgoing_card_public` | Native proposal only; Note owner/checkpoint required |
| 39 | `upgrades` | No source property ID; owner/member join required |
| 40 | `source_visible_counter` | No generic owner; only individually registered owner/subkey pairs |
| 41 | `reserved_unemittable_pending_actual_public_owner_proof` | **No source owner proved; excluded and unemittable** |
| 42 | `intent_visible` | Native proposal only; owner/relic visibility condition required |
| 43 | `payment_status` | Native proposal only; exact payment owner required |
| 44 | `transform_input_kind` | Native proposal only; transform source required |
| 45 | `public_turn_cost_modifier` | Native proposal only; owner/member join required |

This list freezes only the referenced native proposal vocabulary. It does not turn any label into a source ID or prove source-set equality, current visibility, or emission entitlement.


## 3. Numeric-domain/member join rules

The source primitive domain, reachable values, local helper caps, selected native representation, and public visibility are separate facts. Direct Java `int` members have source representability `[-2^31, 2^31-1]`; where Java `int` arithmetic is executed, overflow wraps at that operation in source behavior. A wider native carrier must sign-extend and preserve the resulting value; it must not narrow an out-of-range input. A Java `byte` intent/move discriminator is signed `[-128,127]` and must be decoded under its owning monster before it becomes a semantic key. Boolean fields are exactly `0/1`. Collection sizes/indexes derive from source `int`, but imply no fixed content limit. Allocation/carrier failure is administrative, never a reason to clip a legal source value, shorten a menu, omit a row, or fabricate defeat.

| Source owner/member or derived field | Read/write/default/reset and numeric law | Visibility and mapping |
|---|---|---|
| `MapRoomNode.x`, `.y` | Source signed `int`; read as coordinates, no clipping. Defaults/construction and map writers remain map-owner-specific. | Public for exposed map nodes. Map node identity is a row reference, not serialized object identity. |
| `MapRoomNode.hasEmeraldKey` | Boolean node marker set by burning-elite map selection and read at elite prebattle/room reward paths. It means the node’s burning designation. | Conditional on public map-observation policy. **Not** `Settings.hasEmeraldKey`, the separately acquired player key. |
| `Settings.hasRubyKey`, `.hasSapphireKey`, `.hasEmeraldKey` | Boolean acquired-key state; writes at their respective acquisition choices; independent from map marker. | Public only when the actual run-key state is entitled. Never alias the burning-node marker to this property. |
| `AbstractCard.cardID` | Exact source string identity; constructor/transform/copy chooses kind. Aliases are exact and namespace-scoped. | Key resolves to a canonical card key; no localized display name or class-name fallback. |
| `AbstractCard.timesUpgraded` | Signed `int`; constructor starts at owner-specified value; upgrade writers increment, with Java overflow at that operation. Searing Blow and specialized constructors/copies have nonuniform construction/replay behavior. No generic maximum. | Public for entitled card instances; instance value, not a kind-global counter. |
| `AbstractCard.misc` | Signed `int`, including owner-defined negative sentinels and state; constructor/default, reset and mutator laws vary per class. Java arithmetic wraps where performed. | Conditional: mask when card instance’s relevant state is not legitimately visible. Preserve instance/receiver identity. |
| `AbstractCard.cost` | Signed `int`. `-1` is X, `-2` is unplayable, `0` is an ordinary cost; other negative values are not silently reclassified. Constructor/copy/upgrade and mutators are class/operation-specific. | Entitlement-gated; base cost and its derived kind are masked together if hidden. Never substitute sentinel with zero. |
| `AbstractCard.costForTurn` | Distinct signed `int`, mutable separately from base cost; setter guards and ordered writers govern. ResetAttributes restores turn cost from mutable combat cost, not printed base cost. | Independently entitlement-gated; never derive this from `cost_kind`, free flag, or base cost. |
| Cost/free mutations | `setCostForTurn` negative ordinary requests clamp to zero only under its own guard; negative X/unplayable states are preserved. Positive mutable combat cost can independently set `freeToPlayOnce`. Confusion/randomize-hand, Enlightenment, Madness and Corruption use different branches. Eviscerate copy mutates the source; Blood for Blood copy updates the returned copy. | Expose separate base cost, turn cost, free flag and captured-X semantics only at their own public checkpoints. No generic total precedence rule. |
| `AbstractCreature.currentHealth`, `maxHealth`, `currentBlock`, `gold` | Signed `int` members. Constructor/start defaults are owner-specific; damage/heal/maxHP/payment methods mutate at their semantic effect boundary. Do not infer universal nonnegative range from normal UI. `addBlock`’s processed result caps current Block at 999 at that helper only. | Public run/player fields; actor fields only for public actors. No universal 999 resource cap. |
| Strength amount | StrengthPower constructor/stack/reduce clamps result to `[-999,999]` on those paths; addition/subtraction precedes clamp. Other powers/counters do not inherit this cap. | Owner-qualified power rows; preserve signed amount and owner, no generic counter saturation. |
| `EnergyManager.energy`, `energyMaster` | Signed `int` members. `addEnergy` helper cap at 999 applies only on that method path; direct/setter paths remain distinct. | Current energy and energy master are separate fields and visibility. No 999 input clamp. |
| `AbstractRelic.counter`, `usedUp` | `counter` is signed `int`, potentially owner-defined negative/special values; constructor and per-relic reset/write behavior differ. `usedUp` is boolean. No global bound. | Relic identity and owner inventory position are public; counter is conditional on that relic’s source-visible predicate. |
| `AbstractPotion.ID`, `slot`, `potency` / `getPotency()` | ID is exact factory identity; slot is signed source `int`; potency is signed `int` and the accessor result is authoritative. Sacred Bark multiplication follows source accessor semantics, including Java overflow at the source arithmetic boundary. | Slot/occupied state public; identity and potency only for an occupied, entitled slot. No generic positive potency assumption. |
| `AbstractMonster.intent`, `nextMove`, `intentDmg`, `intentMultiAmt` | `intent` object plus signed-byte local move ID are owner-qualified; payloads are signed `int`. Intent update/constructor and payload calculation order are monster-specific; float-derived payloads use their source binary32 boundary before narrowing. | Mask hidden/suppressed intent and derived payload together. Never emit byte ordinals as global intent codes. |
| `AbstractPower.ID`, `.owner`, `.amount` | Exact string identity, object owner reference, signed `int` amount; constructors, stacking, decrement and removal are power-specific. | Public power key and owner-qualified order; amount conditional, never a universal saturating count. |
| `AbstractOrb.ID`, `.passiveAmount`, `.evokeAmount` | Exact orb ID and signed `int` values after applicable focus/lock-on hooks. Calculation stages may be binary32 before source narrowing. Base class has no common `counter` member. | Actual occupied slot order; no synthetic counter or absent-orb rows. |
| `ShopScreen` settled prices | Generated discounts/jitter use float intermediates and source-specific rounding order; once settled, shop card/relic/potion price fields and `actualPurgeCost` are signed `int` paid/displayed amounts. Preserve source settlement and integer overflow behavior. | A public price requires a registered owner/member and entitled live offer. The carrier is settled signed-i32, not float. |
| Other event/reward/composite prices | No generic price owner or property may be inferred from price-like display text or intermediate float. Exact member, settlement, payer, write point, and reveal checkpoint required per owner. | Do not emit until that owner proof exists. Property 41 is reserved and unemittable absent actual owner/emission proof. |
| Float intermediates | Binary32 operands/operation ordering/rounding boundary are source-method-specific. Do not replace with binary64/exact-rational arithmetic or round only at a later observation. Preserve NaN/infinity/signed-zero policy only where source field permits. | Raw intermediate values are not public prices; native administrative overflow must not truncate a settled legal value. |
| `property_code=41` | No proven actual owner/member/emission path in accepted evidence. | Exclude it from public emitted property rows and from the native consumer’s recognized/emittable property set. Do not serialize “unknown property 41” as an extension. |

These are source facts only at their stated member/helper/owner boundaries. They do not imply every possible input is naturally reachable. The native implementation may choose a wider safe representation but may not clamp source state. Carrier i64 and signed-i32 settled price are representation choices, not source behavior.

## 4. Visibility, ownership, lifetime, and absent values

Every emitted field requires a complete `(source owner, source member or exact producer, read checkpoint, writer/mutator set, constructor/default/reset, lifetime, visibility predicate, destination)` join. The source object pointer remains private; instance references become slot-local typed refs. Current container order is an observation only when the container is public at that checkpoint. Hidden draw order is not a stable catalog order; Frozen Eye may reveal current draw order, whereas Runic Dome suppresses enemy intent. A face-down board location may be public while its value remains unknown. An absent row, a present-but-unknown field, and known zero are distinct states.

Fixed members already identified in the accepted source crosswalk include run act/floor/ascension/turn; player class/resources/energy/slots/orbs/keys/stance; map coordinates/room/taken/node marker; card identity/cost/upgrade/misc/flags; relic identity/counter/used state; potion identity/slot/potency; actor resources/intent payload; power identity/owner/amount; and orb identity/passive/evoke amounts. Those keys are not a complete source-member inventory: per-owner reader/writer/default/reset and public predicates remain required, especially subclass-only properties, offers, room objects, action tuples and relation endpoints.

The accepted property vocabulary is finite and version-local. Property 41’s stated reserved label has no owner proof and therefore has no emission path. Property 40 is not an open extension bag: only separately registered owner/subkey pairs may use it. Public facts/history are environment knowledge ledgers, not permission to reveal private RNG/history fields. All hidden-state and owner rules are fail-closed.

## 5. Numeric rounding and failure boundaries

* Integer arithmetic follows the actual source operation sequence. Java signed-int add/subtract/multiply wraps modulo `2^32` at that operation; no neighboring helper’s clamp is generalized.
* Float-driven source calculations retain binary32 precision and operation order, then round/narrow only where the source does. Card damage/Block, potion percentages, chances and price generation are not universal float fields.
* In particular, settled shop price is already an integer source result; do not expose or recompute it from a float wire value. Preserve ties/rounding order and integer discount application as the relevant source owner defines them.
* `AbstractCreature.addBlock` and energy helper bounds are exact helper-local contracts, not field-wide caps. Strength’s `[-999,999]` is limited to the named StrengthPower constructor/stack/reduce paths.
* A legal source value outside an elected carrier’s exact range, nonrepresentable conversion, failed checked allocation, or arithmetic necessary for representation faults administratively with provenance. It never becomes clipping, omission, a fabricated game outcome, retry, or a legal-input rejection.

## 6. Actual equality evidence and remaining obligations

Accepted E29 reports a 146-cell candidate crosswalk and a 131-card bounded alias table; neither is proof that all source-owned fields or legal identities are equal to a complete emitted set. This continuation now publishes concrete static identity rows for 370 `CardLibrary` registrations, the 141 accepted Act 1 relic candidates, the 33 Ironclad potion factory IDs, 25 Act 1 enemy kinds, 22 scoped encounter keys, and 25 Act 1 event factory IDs. Private checks compare the card table against source loader calls and exact class-file constants; relic rows against the accepted candidate ledger and `RelicLibrary` registration/class IDs; potion rows against the Ironclad factory list/cases; event rows against `EventHelper.getEvent`; encounter names against retained source constants; and enemy IDs against pinned class-file constants. These are bounded namespace checks, not proof of all legal producer/use sites or all required source namespaces. Static declarations alone also do not prove a source member is read, written, reset, owner-qualified, or public. The complete expected set still must be derived from actual in-scope reachability and compared against independently maintained delivered rows; a checker built from the same authored list on both sides is circular and invalid.

For that reason this document does **not** claim full catalog or numeric closure. The identity-catalog work remains incomplete; these concrete joins are still open:

1. **Cards:** the 370-row list is the static `CardLibrary` registration universe, not yet the exact legal producer set. Every reachable producer/selector route (ordinary rewards, Note/Shard/off-color, generated/dependent, transformed/copied/event/potion and Status/Curse/control identities) still needs explicit owner-qualified inclusion/exclusion and emission checkpoints; current per-row route/pool text is not a substitute for those joins.
2. **Powers:** complete literal source IDs/classes and the actual player/monster/event/relic/card/action creation, application, visibility and removal owners are not catalogued or reconciled.
3. **Events and choices:** the event-kind IDs/classes are joined, but the finite actual source option keys/indexes and owner-specific eligible/current-choice emission rows are not. Rooms, reward identities/origins, room objects and reward/choice transitions also lack complete source-key catalogs.
4. **Other source namespaces:** complete orb IDs, intent keys/payload ownership, public-fact/history keys and source effect-site identities/use-set equality are absent. Source-local move bytes remain subordinate to monster kind; they are not global intent codes.
5. **Properties:** accepted codes 1–45 are a native proposal vocabulary, not source IDs. Code 41 is excluded and unemittable pending owner proof; code 40 is not an open bag. The mapping from every other property/subkey to exact source owner/member, source option/producer, and public reveal checkpoint is not complete.
6. **Ownership/emission equality:** the bounded row checks do not establish all producer/read/write/use sites, route eligibility, hidden-state predicates or full namespace source-set equality. The current card row selector language and broad live-offer disclosure conditions are intentionally qualified, not asserted as proven source law.
7. **Numeric/member joins:** this continuation does not expand those joins. Per-member domain/read-write/default/reset/rounding/overflow/visibility completion has been split to the disjoint `.46` numeric-members author; the qualified facts above remain as originally evidenced. This document still does not close the full source-field inventory or whole observation-set equality.

The `.44` task is therefore **not complete**. Do not treat this document as an API/codec, final source registry, native implementation input without these gaps, or a statement that prior accepted evidence proves them. Native owner may consume only the concrete facts above as qualified source inputs; native numeric wire IDs/instance IDs remain its responsibility.

## 7. Accepted public evidence locators

These links are normative source-record references and their own evidence grades/limitations apply: [consolidated Act 1 specification §§12–13 and B07/B09/B10/B11](../superpowers/specs/2026-10-19-ironclad-a0-act1-consolidated-design.md); [semantic/native contract closure (E29)](2026-10-19-act1-semantic-contract-closure.md); [Ironclad, colorless, Status, Curse and dependent cards](2026-10-19-act1-content-red-colorless.md); [Silent cards](2026-10-19-act1-content-silent.md); [Defect cards/orbs](2026-10-19-act1-content-defect.md); [Watcher cards/choices](2026-10-19-act1-content-watcher.md); [enemy/encounter records](2026-10-19-act1-content-enemies.md); [events and Neow](2026-10-19-act1-content-event-neow.md); [economy, rewards, and potions](2026-10-19-act1-content-economy-potions.md); [relic source records](2026-10-18-act1-relics-closure.md). They are public inputs, not private evidence dependencies for the normative rules stated above.
