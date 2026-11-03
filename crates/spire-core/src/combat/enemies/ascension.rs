//! M29 per-species ascension owner. Combat code reads the one combat-level
//! ascension (`State.ascension`) and asks this module for species values, so no
//! A0 literal that depends on ascension lives outside a species function.
//! M29a2 landed the plumbing; each species' chunk (M29b–e2) fills its ladder from
//! the reviewed spec sheet. M29b: the 17 Act 1 normal species. M29c: Act 1 elites and bosses,
//! the Slavers and the Act 2 elites.
use super::super::{FullPowerEntry, FullPowerId};
use crate::content::{EncounterId, EnemyId};

/// Typed rejection above an encounter's playable ceiling (behaviour rows pending in M32).
pub const ABOVE_CEILING: &str = "enemy behaviour above this ascension lands in M32";
/// Typed interim rejection for a species whose ascension values have not landed yet (M29b–e2).
pub const INTERIM: &str = "enemy setup facts unavailable for this act/ascension";
/// Ascension outside the game's range.
pub const OUT_OF_RANGE: &str = "enemy ascension must be within 0..=20";

/// Whether the species' numeric ascension values are implemented at `ascension`.
pub fn species_supports_ascension(id: EnemyId, ascension: u8) -> bool {
    use EnemyId::*;
    ascension == 0 || ascension <= 20 && matches!(id,
        Cultist | JawWorm | GremlinNob | SlaverBlue | FungiBeast
        // M29b: Act 1 normals.
        | AcidSlimeS | AcidSlimeM | AcidSlimeL | SpikeSlimeS | SpikeSlimeM | SpikeSlimeL | LouseRed | LouseGreen
        | Looter | GremlinMad | GremlinSneaky | GremlinFat | GremlinShield | GremlinWizard
        // M29c: Act 1 elites/bosses, Slavers, Act 2 elites.
        | Lagavulin | Sentry | Hexaghost | SlimeBoss | Guardian | SlaverRed | BookOfStabbing | GremlinLeader | Taskmaster)
}

/// Highest ascension whose behaviour the engine implements for the species
/// (inventory §3a: one below the lowest behaviour threshold pending in M32).
pub fn playable_ceiling(id: EnemyId) -> u8 {
    use EnemyId::*;
    match id {
        AcidSlimeS | AcidSlimeM | AcidSlimeL | SpikeSlimeM | SpikeSlimeL | LouseRed | LouseGreen
        | GremlinWizard | GremlinFat | SlaverRed | BanditLeader | Chosen | Mystic | ShelledParasite
        | SnakePlant | Snecko | Darkling | SpireGrowth => 16,
        BookOfStabbing | Taskmaster | SpireShield | SpireSpear => 17,
        Champ | BronzeAutomaton | Deca | TimeEater => 18,
        _ => 20,
    }
}

/// Species a combatant can create in combat (spawn or split), transitively.
pub fn spawn_closure(id: EnemyId) -> &'static [EnemyId] {
    use EnemyId::*;
    match id {
        SlimeBoss => &[AcidSlimeL, SpikeSlimeL, AcidSlimeM, SpikeSlimeM],
        AcidSlimeL => &[AcidSlimeM],
        SpikeSlimeL => &[SpikeSlimeM],
        GremlinLeader => &[GremlinMad, GremlinSneaky, GremlinFat, GremlinShield, GremlinWizard],
        Collector => &[TorchHead],
        BronzeAutomaton => &[BronzeOrb],
        Reptomancer => &[Dagger],
        _ => &[],
    }
}

/// Every species any composition draw of `encounter` can produce (erratum n6),
/// plus each one's spawn closure. Pure in `encounter`; independent of the RNG.
pub fn encounter_species(encounter: EncounterId) -> Vec<EnemyId> {
    use EnemyId::*;
    let drawn: Vec<EnemyId> = match encounter {
        EncounterId::GremlinGang => super::act2::gremlin_leader::POOL.to_vec(),
        EncounterId::GremlinLeader => {
            let mut ids = super::act2::gremlin_leader::POOL.to_vec();
            ids.push(GremlinLeader);
            ids
        },
        EncounterId::ExordiumThugs => vec![SpikeSlimeM, AcidSlimeM, LouseRed, LouseGreen, Cultist, Looter, SlaverBlue, SlaverRed],
        EncounterId::ExordiumWildlife => vec![FungiBeast, JawWorm, SpikeSlimeM, AcidSlimeM, LouseRed, LouseGreen],
        EncounterId::ThreeShapes | EncounterId::FourShapes => vec![Repulsor, Exploder, Spiker],
        EncounterId::SphereAndTwoShapes => vec![Spiker, Repulsor, Exploder, SphericGuardian],
        EncounterId::TwoLouse | EncounterId::ThreeLouse => vec![LouseRed, LouseGreen],
        EncounterId::SmallSlimes => vec![SpikeSlimeS, AcidSlimeM, AcidSlimeS, SpikeSlimeM],
        EncounterId::LargeSlime => vec![AcidSlimeL, SpikeSlimeL],
        EncounterId::LotsOfSlimes => vec![SpikeSlimeS, AcidSlimeS],
        _ => super::fixed_formation(encounter).unwrap_or_default(),
    };
    let mut set = Vec::new();
    for id in drawn.iter().flat_map(|id| std::iter::once(id).chain(spawn_closure(*id))) {
        if !set.contains(id) { set.push(*id); }
    }
    set
}

/// Minimum playable ceiling over the encounter's species set (n6).
pub fn encounter_ceiling(encounter: EncounterId) -> u8 {
    encounter_species(encounter).into_iter().map(playable_ceiling).min().unwrap_or(0)
}

/// Typed admission of `encounter` at `ascension`: `Err(ABOVE_CEILING)` above the
/// encounter ceiling, `Err(INTERIM)` while a species' values have not landed.
pub fn encounter_admission(encounter: EncounterId, ascension: u8) -> Result<(), &'static str> {
    if ascension == 0 { return Ok(()); }
    let species = encounter_species(encounter);
    if ascension > 20 { return Err(OUT_OF_RANGE); }
    if species.is_empty() { return Err(INTERIM); }
    if ascension > encounter_ceiling(encounter) { return Err(ABOVE_CEILING); }
    if species.iter().all(|id| species_supports_ascension(*id, ascension)) { Ok(()) } else { Err(INTERIM) }
}

/// Typed admission of one combatant and its spawn closure (entry admission and spawn/split
/// constructors): `ABOVE_CEILING` above the closure's lowest ceiling, else `INTERIM` until landed.
pub fn species_admission(id: EnemyId, ascension: u8) -> Result<(), &'static str> {
    if ascension == 0 { return Ok(()); }
    if ascension > 20 { return Err(OUT_OF_RANGE); }
    let closure = || std::iter::once(&id).chain(spawn_closure(id));
    // The ceiling is over the closure too (plan I3): a solo Slime Boss is bounded by its children.
    if closure().map(|id| playable_ceiling(*id)).any(|ceiling| ascension > ceiling) { return Err(ABOVE_CEILING); }
    if closure().all(|id| species_supports_ascension(*id, ascension)) { Ok(()) }
    else { Err(INTERIM) }
}

/// Louse Bite base range drawn at setup (inclusive).
pub fn louse_bite_range(ascension: u8) -> (i32, i32) { if ascension >= 2 { (6, 8) } else { (5, 7) } }
/// Louse Curl Up amount range drawn at setup (inclusive).
pub fn louse_curl_up_range(ascension: u8) -> (i32, i32) {
    if ascension >= 17 { (9, 12) } else if ascension >= 7 { (4, 8) } else { (3, 7) }
}
/// Darkling Nip base range drawn at setup (inclusive).
pub fn darkling_nip_range(_ascension: u8) -> (i32, i32) { (7, 11) }
/// Giant Head "It Is Time" base for the current countdown (`count` ≤ 0 once the countdown is spent).
pub fn giant_head_time_base(count: i8, _ascension: u8) -> i32 { 30 - 5 * i32::from(count.min(0)) }
/// Transient's first attack base.
pub fn transient_start_base(_ascension: u8) -> i32 { 30 }
/// Angry amount of a Mad Gremlin, at setup and when summoned.
pub fn gremlin_mad_angry(ascension: u8) -> i32 { if ascension >= 17 { 2 } else { 1 } }
/// Gold a thief steals per hit (and its Thievery amount).
pub fn thievery_gold(id: EnemyId, ascension: u8) -> u32 {
    // The Mugger's ladder lands in M29d.
    if id == EnemyId::Looter && ascension >= 17 { 20 } else { 15 }
}
/// Block a Shield Gremlin's Protect gives.
pub fn gremlin_shield_protect(ascension: u8) -> i32 {
    if ascension >= 17 { 11 } else if ascension >= 7 { 8 } else { 7 }
}
/// The Guardian's Mode Shift threshold at setup (memory quota and power amount).
pub fn guardian_mode_shift(ascension: u8) -> i32 {
    if ascension >= 19 { 40 } else if ascension >= 9 { 35 } else { 30 }
}
/// Awakened One's maximum HP after rebirth.
pub fn awakened_rebirth_max_hp(_ascension: u8) -> i32 { 300 }

fn power(id: &'static str, amount: i32) -> FullPowerEntry {
    FullPowerEntry { id: FullPowerId(id), state: super::super::PowerState { amount, skip_first: false, monster_source_grace: false } }
}

/// Pre-battle powers a formation member starts with (setup table; order is part of
/// the state and is kept). Louse Curl Up and Darkling Life Link are added by setup.
pub fn setup_powers(encounter: EncounterId, id: EnemyId, ascension: u8) -> Vec<FullPowerEntry> {
    let mut powers = Vec::new();
    if matches!(id, EnemyId::Deca | EnemyId::Donu) { powers.push(power("artifact", 2)); }
    if encounter == EncounterId::ShieldAndSpear { powers.push(power("artifact", 1)); powers.push(power("back_attack", 1)); }
    if id == EnemyId::TimeEater { powers.push(power("time_warp", 0)); }
    if id == EnemyId::AwakenedOne { for (name, amount) in [("curiosity", 1), ("regenerate", 10), ("unawakened", 1)] { powers.push(power(name, amount)); } }
    if encounter == EncounterId::Reptomancer && id != EnemyId::Reptomancer { powers.push(power("minion", 1)); }
    if id == EnemyId::GiantHead { powers.push(power("slow", 0)); }
    if id == EnemyId::WrithingMass { for (name, amount) in [("reactive", 1), ("malleable", 4)] { powers.push(power(name, amount)); } }
    if id == EnemyId::Transient { for (name, amount) in [("fading", 5), ("shifting", 1)] { powers.push(power(name, amount)); } }
    for (species, name, amount) in [(EnemyId::OrbWalker, "strength_up", 3), (EnemyId::Spiker, "thorns", 3), (EnemyId::Exploder, "explosive", 3)] {
        if id == species { powers.push(power(name, amount)); }
    }
    if id == EnemyId::BronzeAutomaton { powers.push(power("artifact", 3)); }
    if id == EnemyId::CorruptHeart { for (name, amount) in [("beat_of_death", 1), ("invincible", 300)] { powers.push(power(name, amount)); } }
    if id == EnemyId::BookOfStabbing { powers.push(power("painful_stabs", 1)); }
    if encounter == EncounterId::GremlinLeader && id != EnemyId::GremlinLeader { powers.push(power("minion", 1)); }
    if id == EnemyId::GremlinMad { powers.push(power("angry", gremlin_mad_angry(ascension))); }
    if id == EnemyId::SphericGuardian { for (name, amount) in [("artifact", 3), ("barricade", 1)] { powers.push(power(name, amount)); } }
    if id == EnemyId::Byrd { powers.push(power("flight", 3)); }
    if id == EnemyId::Guardian { powers.push(power("mode_shift", guardian_mode_shift(ascension))); }
    if id == EnemyId::ShelledParasite { powers.push(power("plated_armor", 14)); }
    if id == EnemyId::SnakePlant { powers.push(power("malleable", 3)); }
    if matches!(id, EnemyId::Looter | EnemyId::Mugger) { powers.push(power("thievery", thievery_gold(id, ascension) as i32)); }
    if id == EnemyId::FungiBeast { powers.push(power("spore_cloud", 2)); }
    if id == EnemyId::Sentry { powers.push(power("artifact", 1)); }
    powers
}

/// The Split power of the large Slimes and the Slime Boss (setup and split children).
pub fn split_powers(id: EnemyId) -> Vec<FullPowerEntry> {
    if matches!(id, EnemyId::AcidSlimeL | EnemyId::SpikeSlimeL | EnemyId::SlimeBoss) { vec![power("split", 1)] } else { vec![] }
}
