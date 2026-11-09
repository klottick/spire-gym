//! Public intent category and planned hit count (M34a a1/a2).
//!
//! The category is the original game's `AbstractMonster.Intent` shown with a move, keyed per
//! (monster, move) with an ascension condition only where the game has one. The hit count is the
//! number of damage applications the selected move queues, from the same values the move work
//! reads (ascension helpers, AI memory). Both are pure functions of what move selection writes on
//! the actor, so the public view evaluates them; hidden intents publish neither.
use super::enemies::EnemyMemory;
use super::model::{ActorEnemyId, EnemyState};
use crate::content::EnemyId;

/// The 17 source intent categories, in the game's declaration order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IntentCategory {
    Attack, AttackBuff, AttackDebuff, AttackDefend, Buff, Debuff, StrongDebuff, Debug, Defend,
    DefendDebuff, DefendBuff, Escape, Magic, None, Sleep, Stun, Unknown,
}

impl IntentCategory {
    pub const ALL: [Self; 17] = [Self::Attack, Self::AttackBuff, Self::AttackDebuff, Self::AttackDefend,
        Self::Buff, Self::Debuff, Self::StrongDebuff, Self::Debug, Self::Defend, Self::DefendDebuff,
        Self::DefendBuff, Self::Escape, Self::Magic, Self::None, Self::Sleep, Self::Stun, Self::Unknown];

    pub const fn as_str(self) -> &'static str { match self {
        Self::Attack => "attack", Self::AttackBuff => "attack_buff", Self::AttackDebuff => "attack_debuff",
        Self::AttackDefend => "attack_defend", Self::Buff => "buff", Self::Debuff => "debuff",
        Self::StrongDebuff => "strong_debuff", Self::Debug => "debug", Self::Defend => "defend",
        Self::DefendDebuff => "defend_debuff", Self::DefendBuff => "defend_buff", Self::Escape => "escape",
        Self::Magic => "magic", Self::None => "none", Self::Sleep => "sleep", Self::Stun => "stun",
        Self::Unknown => "unknown",
    } }
}

pub(super) fn enemy_id(enemy: &EnemyState) -> EnemyId {
    match enemy.enemy_id { ActorEnemyId::Full(id) => id,
        ActorEnemyId::Legacy(id) => match id { super::EnemyKind::Cultist => EnemyId::Cultist,
            super::EnemyKind::JawWorm => EnemyId::JawWorm } }
}

/// How a (monster, move) pair's category depends on ascension.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CategoryRule {
    Always(IntentCategory),
    /// `below` under ascension `from`, `at_or_above` from it on (Darkling Harden A17, Deca Square A19).
    From { from: u8, below: IntentCategory, at_or_above: IntentCategory },
}

/// How many damage applications a (monster, move) pair queues.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HitsRule {
    NoDamage,
    Fixed(u32),
    /// `ascension::heart_blood_shots_hits`, the count the Heart's move work queues.
    HeartBloodShots,
    /// `ascension::spear_skewer_hits`, the count the Spear's move work queues.
    SpearSkewer,
    /// Book of Stabbing's `stab_count`, written at selection and read by its `BookStab` chain.
    BookStabs,
    /// `ascension::byrd_peck_hits`, the count Byrd's move work queues.
    ByrdPeck,
    /// Maw's selected-turn counter through `maw_nom_hits`, the count its `MawNom` chain makes.
    MawNom,
}

/// One census row: the engine pair, its source category rule and its hit rule.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IntentRow { pub enemy: EnemyId, pub intent: &'static str, pub category: CategoryRule, pub hits: HitsRule }

impl CategoryRule {
    pub const fn at(self, ascension: u8) -> IntentCategory { match self {
        Self::Always(category) => category,
        Self::From { from, below, at_or_above } => if ascension >= from { at_or_above } else { below },
    } }
}

/// The table row for (`enemy`, `intent`), if the census has one.
pub fn row(enemy: EnemyId, intent: &str) -> Option<&'static IntentRow> {
    INTENT_ROWS.iter().find(|row| row.enemy == enemy && row.intent == intent)
}

/// The category shown with `intent` on `enemy` at `ascension`; `None` only for a pair the table lacks.
pub fn category(enemy: EnemyId, intent: &str, ascension: u8) -> Option<IntentCategory> {
    row(enemy, intent).map(|row| row.category.at(ascension))
}

/// Damage applications the selected move will make; `None` for a move that deals no damage.
pub fn planned_hits(enemy: &EnemyState, ascension: u8) -> Option<u32> {
    hits_for(enemy_id(enemy), enemy.intent_id.as_str(), ascension, enemy.full_memory.as_ref())
}

/// [`planned_hits`] from the pair, the combat ascension and the actor's AI memory.
pub fn hits_for(enemy: EnemyId, intent: &str, ascension: u8, memory: Option<&EnemyMemory>) -> Option<u32> {
    match row(enemy, intent)?.hits {
        HitsRule::NoDamage => None,
        HitsRule::Fixed(n) => Some(n),
        HitsRule::HeartBloodShots => Some(super::enemies::ascension::heart_blood_shots_hits(ascension) as u32),
        HitsRule::SpearSkewer => Some(super::enemies::ascension::spear_skewer_hits(ascension) as u32),
        HitsRule::BookStabs => match memory { Some(EnemyMemory::BookOfStabbing { stab_count }) => Some(*stab_count), _ => None },
        HitsRule::ByrdPeck => Some(super::enemies::ascension::byrd_peck_hits(ascension) as u32),
        HitsRule::MawNom => match memory { Some(EnemyMemory::Maw { turns, .. }) => Some(super::enemies::act3::ordinary::maw_nom_hits(*turns)), _ => None },
    }
}

/// The Corrupt Heart's live Invincible cap; `None` for every other actor.
pub fn invincible_cap(enemy: &EnemyState, ascension: u8) -> Option<i32> {
    (enemy_id(enemy) == EnemyId::CorruptHeart).then(|| super::enemies::ascension::heart_invincible(ascension))
}

use CategoryRule::Always;
use IntentCategory as C;
use HitsRule as H;
const fn r(enemy: EnemyId, intent: &'static str, category: CategoryRule, hits: HitsRule) -> IntentRow {
    IntentRow { enemy, intent, category, hits }
}

/// The census table (M34a a1), one row per engine (monster, move) pair.
pub const INTENT_ROWS: &[IntentRow] = &[
    r(EnemyId::AcidSlimeL, "acid_lick", Always(C::Debuff), H::NoDamage),
    r(EnemyId::AcidSlimeL, "acid_spit", Always(C::AttackDebuff), H::Fixed(1)),
    r(EnemyId::AcidSlimeL, "acid_split", Always(C::Unknown), H::NoDamage),
    r(EnemyId::AcidSlimeL, "acid_tackle", Always(C::Attack), H::Fixed(1)),
    r(EnemyId::AcidSlimeM, "acid_lick", Always(C::Debuff), H::NoDamage),
    r(EnemyId::AcidSlimeM, "acid_spit", Always(C::AttackDebuff), H::Fixed(1)),
    r(EnemyId::AcidSlimeM, "acid_tackle", Always(C::Attack), H::Fixed(1)),
    r(EnemyId::AcidSlimeS, "acid_lick", Always(C::Debuff), H::NoDamage),
    r(EnemyId::AcidSlimeS, "acid_tackle", Always(C::Attack), H::Fixed(1)),
    r(EnemyId::Cultist, "attack", Always(C::Attack), H::Fixed(1)),
    r(EnemyId::Cultist, "incantation", Always(C::Buff), H::NoDamage),
    r(EnemyId::FungiBeast, "bite", Always(C::Attack), H::Fixed(1)),
    r(EnemyId::FungiBeast, "grow", Always(C::Buff), H::NoDamage),
    r(EnemyId::LouseGreen, "bite", Always(C::Attack), H::Fixed(1)),
    r(EnemyId::LouseGreen, "spit_web", Always(C::Debuff), H::NoDamage),
    r(EnemyId::LouseRed, "bite", Always(C::Attack), H::Fixed(1)),
    r(EnemyId::LouseRed, "grow", Always(C::Buff), H::NoDamage),
    r(EnemyId::GremlinNob, "bellow", Always(C::Buff), H::NoDamage),
    r(EnemyId::GremlinNob, "bull_rush", Always(C::Attack), H::Fixed(1)),
    r(EnemyId::GremlinNob, "skull_bash", Always(C::AttackDebuff), H::Fixed(1)),
    r(EnemyId::Guardian, "charging_up", Always(C::Defend), H::NoDamage),
    r(EnemyId::Guardian, "defensive_mode", Always(C::Buff), H::NoDamage),
    r(EnemyId::Guardian, "fierce_bash", Always(C::Attack), H::Fixed(1)),
    r(EnemyId::Guardian, "roll_attack", Always(C::Attack), H::Fixed(1)),
    r(EnemyId::Guardian, "twin_slam", Always(C::AttackBuff), H::Fixed(2)),
    r(EnemyId::Guardian, "vent_steam", Always(C::StrongDebuff), H::NoDamage),
    r(EnemyId::Guardian, "whirlwind", Always(C::Attack), H::Fixed(4)),
    r(EnemyId::Hexaghost, "activate", Always(C::Unknown), H::NoDamage),
    r(EnemyId::Hexaghost, "divider", Always(C::Attack), H::Fixed(6)),
    r(EnemyId::Hexaghost, "inferno", Always(C::AttackDebuff), H::Fixed(6)),
    r(EnemyId::Hexaghost, "inflame", Always(C::DefendBuff), H::NoDamage),
    r(EnemyId::Hexaghost, "sear", Always(C::AttackDebuff), H::Fixed(1)),
    r(EnemyId::Hexaghost, "tackle", Always(C::Attack), H::Fixed(2)),
    r(EnemyId::JawWorm, "bellow", Always(C::DefendBuff), H::NoDamage),
    r(EnemyId::JawWorm, "chomp", Always(C::Attack), H::Fixed(1)),
    r(EnemyId::JawWorm, "thrash", Always(C::AttackDefend), H::Fixed(1)),
    r(EnemyId::Lagavulin, "attack", Always(C::Attack), H::Fixed(1)),
    r(EnemyId::Lagavulin, "siphon", Always(C::StrongDebuff), H::NoDamage),
    r(EnemyId::Lagavulin, "sleep", Always(C::Sleep), H::NoDamage),
    r(EnemyId::Lagavulin, "stunned", Always(C::Stun), H::NoDamage),
    r(EnemyId::Looter, "looter_escape", Always(C::Escape), H::NoDamage),
    r(EnemyId::Looter, "looter_lunge", Always(C::Attack), H::Fixed(1)),
    r(EnemyId::Looter, "looter_mug", Always(C::Attack), H::Fixed(1)),
    r(EnemyId::Looter, "looter_smoke_bomb", Always(C::Defend), H::NoDamage),
    r(EnemyId::Mugger, "mugger_escape", Always(C::Escape), H::NoDamage),
    r(EnemyId::Mugger, "mugger_lunge", Always(C::Attack), H::Fixed(1)),
    r(EnemyId::Mugger, "mugger_mug", Always(C::Attack), H::Fixed(1)),
    r(EnemyId::Mugger, "mugger_smoke_bomb", Always(C::Defend), H::NoDamage),
    r(EnemyId::Sentry, "beam", Always(C::Attack), H::Fixed(1)),
    r(EnemyId::Sentry, "bolt", Always(C::Debuff), H::NoDamage),
    r(EnemyId::SlimeBoss, "prep_slam", Always(C::Unknown), H::NoDamage),
    r(EnemyId::SlimeBoss, "slam", Always(C::Attack), H::Fixed(1)),
    r(EnemyId::SlimeBoss, "split", Always(C::Unknown), H::NoDamage),
    r(EnemyId::SlimeBoss, "sticky", Always(C::StrongDebuff), H::NoDamage),
    r(EnemyId::SpikeSlimeL, "spike_flame_tackle", Always(C::AttackDebuff), H::Fixed(1)),
    r(EnemyId::SpikeSlimeL, "spike_lick", Always(C::Debuff), H::NoDamage),
    r(EnemyId::SpikeSlimeL, "spike_split", Always(C::Unknown), H::NoDamage),
    r(EnemyId::SpikeSlimeM, "spike_flame_tackle", Always(C::AttackDebuff), H::Fixed(1)),
    r(EnemyId::SpikeSlimeM, "spike_lick", Always(C::Debuff), H::NoDamage),
    r(EnemyId::SpikeSlimeS, "spike_tackle", Always(C::Attack), H::Fixed(1)),
    r(EnemyId::SphericGuardian, "sphere_activate", Always(C::Defend), H::NoDamage),
    r(EnemyId::SphericGuardian, "sphere_debuff", Always(C::AttackDebuff), H::Fixed(1)),
    r(EnemyId::SphericGuardian, "sphere_slam", Always(C::Attack), H::Fixed(2)),
    r(EnemyId::SphericGuardian, "sphere_harden", Always(C::AttackDefend), H::Fixed(1)),
    r(EnemyId::Byrd, "byrd_caw", Always(C::Buff), H::NoDamage),
    r(EnemyId::Byrd, "byrd_peck", Always(C::Attack), H::ByrdPeck),
    r(EnemyId::Byrd, "byrd_swoop", Always(C::Attack), H::Fixed(1)),
    r(EnemyId::Byrd, "byrd_headbutt", Always(C::Attack), H::Fixed(1)),
    r(EnemyId::Byrd, "byrd_go_airborne", Always(C::Unknown), H::NoDamage),
    r(EnemyId::Byrd, "byrd_stunned", Always(C::Stun), H::NoDamage),
    r(EnemyId::Chosen, "chosen_poke", Always(C::Attack), H::Fixed(2)),
    r(EnemyId::Chosen, "chosen_hex", Always(C::StrongDebuff), H::NoDamage),
    r(EnemyId::Chosen, "chosen_debilitate", Always(C::AttackDebuff), H::Fixed(1)),
    r(EnemyId::Chosen, "chosen_drain", Always(C::Debuff), H::NoDamage),
    r(EnemyId::Chosen, "chosen_zap", Always(C::Attack), H::Fixed(1)),
    r(EnemyId::Snecko, "snecko_glare", Always(C::StrongDebuff), H::NoDamage),
    r(EnemyId::Snecko, "snecko_bite", Always(C::Attack), H::Fixed(1)),
    r(EnemyId::Snecko, "snecko_tail_whip", Always(C::AttackDebuff), H::Fixed(1)),
    r(EnemyId::Centurion, "centurion_slash", Always(C::Attack), H::Fixed(1)),
    r(EnemyId::Centurion, "centurion_protect", Always(C::Defend), H::NoDamage),
    r(EnemyId::Centurion, "centurion_fury", Always(C::Attack), H::Fixed(3)),
    r(EnemyId::Mystic, "mystic_attack", Always(C::AttackDebuff), H::Fixed(1)),
    r(EnemyId::Mystic, "mystic_buff", Always(C::Buff), H::NoDamage),
    r(EnemyId::Mystic, "mystic_heal", Always(C::Buff), H::NoDamage),
    r(EnemyId::GremlinLeader, "leader_rally", Always(C::Unknown), H::NoDamage),
    r(EnemyId::GremlinLeader, "leader_stab", Always(C::Attack), H::Fixed(3)),
    r(EnemyId::GremlinLeader, "leader_encourage", Always(C::DefendBuff), H::NoDamage),
    r(EnemyId::GremlinMad, "gremlin_scratch", Always(C::Attack), H::Fixed(1)),
    r(EnemyId::GremlinSneaky, "gremlin_puncture", Always(C::Attack), H::Fixed(1)),
    r(EnemyId::GremlinFat, "gremlin_smash", Always(C::AttackDebuff), H::Fixed(1)),
    r(EnemyId::GremlinShield, "gremlin_protect", Always(C::Defend), H::NoDamage),
    r(EnemyId::GremlinShield, "gremlin_bash", Always(C::Attack), H::Fixed(1)),
    r(EnemyId::GremlinWizard, "gremlin_charge", Always(C::Unknown), H::NoDamage),
    r(EnemyId::GremlinWizard, "gremlin_blast", Always(C::Attack), H::Fixed(1)),
    r(EnemyId::SlaverBlue, "slaver_stab", Always(C::Attack), H::Fixed(1)),
    r(EnemyId::SlaverBlue, "slaver_rake", Always(C::AttackDebuff), H::Fixed(1)),
    r(EnemyId::SlaverRed, "slaver_stab", Always(C::Attack), H::Fixed(1)),
    r(EnemyId::SlaverRed, "slaver_entangle", Always(C::StrongDebuff), H::NoDamage),
    r(EnemyId::SlaverRed, "slaver_scrape", Always(C::AttackDebuff), H::Fixed(1)),
    r(EnemyId::Taskmaster, "taskmaster_whip", Always(C::AttackDebuff), H::Fixed(1)),
    r(EnemyId::BanditPointy, "pointy_special", Always(C::Attack), H::Fixed(2)),
    r(EnemyId::BanditLeader, "romeo_mock", Always(C::Unknown), H::NoDamage),
    r(EnemyId::BanditLeader, "romeo_agonizing_slash", Always(C::AttackDebuff), H::Fixed(1)),
    r(EnemyId::BanditLeader, "romeo_cross_slash", Always(C::Attack), H::Fixed(1)),
    r(EnemyId::BanditBear, "bear_hug", Always(C::StrongDebuff), H::NoDamage),
    r(EnemyId::BanditBear, "bear_lunge", Always(C::AttackDefend), H::Fixed(1)),
    r(EnemyId::BanditBear, "bear_maul", Always(C::Attack), H::Fixed(1)),
    r(EnemyId::BookOfStabbing, "book_multi", Always(C::Attack), H::BookStabs),
    r(EnemyId::BookOfStabbing, "book_big", Always(C::Attack), H::Fixed(1)),
    r(EnemyId::Champ, "champ_anger", Always(C::Buff), H::NoDamage),
    r(EnemyId::Champ, "champ_execute", Always(C::Attack), H::Fixed(2)),
    r(EnemyId::Champ, "champ_taunt", Always(C::Debuff), H::NoDamage),
    r(EnemyId::Champ, "champ_stance", Always(C::DefendBuff), H::NoDamage),
    r(EnemyId::Champ, "champ_gloat", Always(C::Buff), H::NoDamage),
    r(EnemyId::Champ, "champ_slap", Always(C::AttackDebuff), H::Fixed(1)),
    r(EnemyId::Champ, "champ_slash", Always(C::Attack), H::Fixed(1)),
    r(EnemyId::Collector, "collector_spawn", Always(C::Unknown), H::NoDamage),
    r(EnemyId::Collector, "collector_ultimate", Always(C::StrongDebuff), H::NoDamage),
    r(EnemyId::Collector, "collector_revive", Always(C::Unknown), H::NoDamage),
    r(EnemyId::Collector, "collector_rake", Always(C::Attack), H::Fixed(1)),
    r(EnemyId::Collector, "collector_buff", Always(C::DefendBuff), H::NoDamage),
    r(EnemyId::TorchHead, "torch_attack", Always(C::Attack), H::Fixed(1)),
    r(EnemyId::BronzeAutomaton, "automaton_spawn", Always(C::Unknown), H::NoDamage),
    r(EnemyId::BronzeAutomaton, "automaton_beam", Always(C::Attack), H::Fixed(1)),
    r(EnemyId::BronzeAutomaton, "automaton_stun", Always(C::Stun), H::NoDamage),
    r(EnemyId::BronzeAutomaton, "automaton_flail", Always(C::Attack), H::Fixed(2)),
    r(EnemyId::BronzeAutomaton, "automaton_boost", Always(C::DefendBuff), H::NoDamage),
    r(EnemyId::BronzeOrb, "orb_stasis", Always(C::StrongDebuff), H::NoDamage),
    r(EnemyId::BronzeOrb, "orb_support", Always(C::Defend), H::NoDamage),
    r(EnemyId::BronzeOrb, "orb_beam", Always(C::Attack), H::Fixed(1)),
    r(EnemyId::ShelledParasite, "shelled_parasite_fell", Always(C::AttackDebuff), H::Fixed(1)),
    r(EnemyId::ShelledParasite, "shelled_parasite_double_strike", Always(C::Attack), H::Fixed(2)),
    r(EnemyId::ShelledParasite, "shelled_parasite_life_suck", Always(C::AttackBuff), H::Fixed(1)),
    r(EnemyId::ShelledParasite, "shelled_parasite_stunned", Always(C::Stun), H::NoDamage),
    r(EnemyId::SnakePlant, "snake_plant_chomp_chomp", Always(C::Attack), H::Fixed(3)),
    r(EnemyId::SnakePlant, "snake_plant_enfeebling_spores", Always(C::StrongDebuff), H::NoDamage),
    r(EnemyId::OrbWalker, "orb_laser", Always(C::AttackDebuff), H::Fixed(1)),
    r(EnemyId::OrbWalker, "orb_claw", Always(C::Attack), H::Fixed(1)),
    r(EnemyId::Repulsor, "repulsor_attack", Always(C::Attack), H::Fixed(1)),
    r(EnemyId::Repulsor, "repulsor_daze", Always(C::Debuff), H::NoDamage),
    r(EnemyId::Exploder, "exploder_attack", Always(C::Attack), H::Fixed(1)),
    r(EnemyId::Exploder, "exploder_explode", Always(C::Unknown), H::NoDamage),
    r(EnemyId::Spiker, "spiker_attack", Always(C::Attack), H::Fixed(1)),
    r(EnemyId::Spiker, "spiker_buff", Always(C::Buff), H::NoDamage),
    r(EnemyId::Darkling, "darkling_harden", CategoryRule::From { from: 17, below: C::Defend, at_or_above: C::DefendBuff }, H::NoDamage),
    r(EnemyId::Darkling, "darkling_nip", Always(C::Attack), H::Fixed(1)),
    r(EnemyId::Darkling, "darkling_chomp", Always(C::Attack), H::Fixed(2)),
    r(EnemyId::Darkling, "darkling_reincarnate", Always(C::Buff), H::NoDamage),
    r(EnemyId::Darkling, "darkling_regrowing", Always(C::Unknown), H::NoDamage),
    r(EnemyId::WrithingMass, "writhing_multi", Always(C::Attack), H::Fixed(3)),
    r(EnemyId::WrithingMass, "writhing_block", Always(C::AttackDefend), H::Fixed(1)),
    r(EnemyId::WrithingMass, "writhing_debuff", Always(C::AttackDebuff), H::Fixed(1)),
    r(EnemyId::WrithingMass, "writhing_big", Always(C::Attack), H::Fixed(1)),
    r(EnemyId::WrithingMass, "writhing_parasite", Always(C::StrongDebuff), H::NoDamage),
    r(EnemyId::AwakenedOne, "awa_slash", Always(C::Attack), H::Fixed(1)),
    r(EnemyId::AwakenedOne, "awa_soul", Always(C::Attack), H::Fixed(4)),
    r(EnemyId::AwakenedOne, "awa_echo", Always(C::Attack), H::Fixed(1)),
    r(EnemyId::AwakenedOne, "awa_sludge", Always(C::AttackDebuff), H::Fixed(1)),
    r(EnemyId::AwakenedOne, "awa_tackle", Always(C::Attack), H::Fixed(3)),
    r(EnemyId::AwakenedOne, "awa_rebirth", Always(C::Unknown), H::NoDamage),
    r(EnemyId::TimeEater, "time_reverb", Always(C::Attack), H::Fixed(3)),
    r(EnemyId::TimeEater, "time_slam", Always(C::AttackDebuff), H::Fixed(1)),
    r(EnemyId::TimeEater, "time_ripple", Always(C::DefendDebuff), H::NoDamage),
    r(EnemyId::TimeEater, "time_haste", Always(C::Buff), H::NoDamage),
    r(EnemyId::SpireGrowth, "growth_quick", Always(C::Attack), H::Fixed(1)),
    r(EnemyId::SpireGrowth, "growth_constrict", Always(C::StrongDebuff), H::NoDamage),
    r(EnemyId::SpireGrowth, "growth_smash", Always(C::Attack), H::Fixed(1)),
    r(EnemyId::Transient, "transient_attack", Always(C::Attack), H::Fixed(1)),
    r(EnemyId::Maw, "maw_roar", Always(C::StrongDebuff), H::NoDamage),
    r(EnemyId::Maw, "maw_nom", Always(C::Attack), H::MawNom),
    r(EnemyId::Maw, "maw_drool", Always(C::Buff), H::NoDamage),
    r(EnemyId::Maw, "maw_slam", Always(C::Attack), H::Fixed(1)),
    r(EnemyId::GiantHead, "head_count", Always(C::Attack), H::Fixed(1)),
    r(EnemyId::GiantHead, "head_time", Always(C::Attack), H::Fixed(1)),
    r(EnemyId::GiantHead, "head_glare", Always(C::Debuff), H::NoDamage),
    r(EnemyId::Nemesis, "nemesis_tri", Always(C::Attack), H::Fixed(3)),
    r(EnemyId::Nemesis, "nemesis_scythe", Always(C::Attack), H::Fixed(1)),
    r(EnemyId::Nemesis, "nemesis_burn", Always(C::Debuff), H::NoDamage),
    r(EnemyId::Reptomancer, "repto_spawn", Always(C::Unknown), H::NoDamage),
    r(EnemyId::Reptomancer, "repto_snake", Always(C::AttackDebuff), H::Fixed(2)),
    r(EnemyId::Reptomancer, "repto_bite", Always(C::Attack), H::Fixed(1)),
    r(EnemyId::Dagger, "dagger_wound", Always(C::AttackDebuff), H::Fixed(1)),
    r(EnemyId::Dagger, "dagger_explode", Always(C::Attack), H::Fixed(1)),
    r(EnemyId::Deca, "deca_beam", Always(C::AttackDebuff), H::Fixed(2)),
    r(EnemyId::Deca, "deca_square", CategoryRule::From { from: 19, below: C::Defend, at_or_above: C::DefendBuff }, H::NoDamage),
    r(EnemyId::Donu, "donu_beam", Always(C::Attack), H::Fixed(2)),
    r(EnemyId::Donu, "donu_circle", Always(C::Buff), H::NoDamage),
    r(EnemyId::SpireShield, "shield_bash", Always(C::AttackDebuff), H::Fixed(1)),
    r(EnemyId::SpireShield, "shield_fortify", Always(C::Defend), H::NoDamage),
    r(EnemyId::SpireShield, "shield_smash", Always(C::AttackDefend), H::Fixed(1)),
    r(EnemyId::SpireSpear, "spear_burn_strike", Always(C::AttackDebuff), H::Fixed(2)),
    r(EnemyId::SpireSpear, "spear_strength", Always(C::Buff), H::NoDamage),
    r(EnemyId::SpireSpear, "spear_skewer", Always(C::Attack), H::SpearSkewer),
    r(EnemyId::CorruptHeart, "heart_debilitate", Always(C::StrongDebuff), H::NoDamage),
    r(EnemyId::CorruptHeart, "heart_blood_shots", Always(C::Attack), H::HeartBloodShots),
    r(EnemyId::CorruptHeart, "heart_echo", Always(C::Attack), H::Fixed(1)),
    r(EnemyId::CorruptHeart, "heart_buff", Always(C::Buff), H::NoDamage),
];
