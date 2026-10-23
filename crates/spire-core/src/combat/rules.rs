//! Bounded slice calculations and exact two-stage enemy selection.
//! Source-grounded, not original-runtime validated; no history/intent mutation here.
use super::{EnemyKind, InputError, MoveId};
use super::rng::{self, Domain, RngError, StreamState};

fn bounded(value: i32, maximum: i32, name: &str) -> Result<(), InputError> {
    if (0..=maximum).contains(&value) {
        Ok(())
    } else {
        Err(InputError::Unsupported(format!("{name} outside supported numeric surface")))
    }
}

fn calculated(value: Option<i32>) -> Result<i32, InputError> {
    value.ok_or_else(|| InputError::Invariant("bounded arithmetic overflow".into()))
}

pub fn normal_damage(base: i32, strength: i32, vulnerable: bool) -> Result<i32, InputError> {
    bounded(base, 11, "base damage")?;
    bounded(strength, 999, "Strength")?;
    // Nonnegative base + Strength <= 1010; tripling <= 3030. Integer division
    // after multiplication is exactly floor(1.5*n), with output <= 1515.
    let total = calculated(base.checked_add(strength))?;
    Ok(if vulnerable { calculated(total.checked_mul(3))? / 2 } else { total })
}

/// NORMAL attack damage with native binary32 give/stance/receive ordering and
/// one final floor. The old helper remains unchanged for legacy callers.
pub fn normal_damage_modifiers(base: i32, strength: i32, weak: bool,
    wrath: bool, vulnerable: bool) -> Result<i32, InputError>
{
    normal_damage_modifiers_bounded(base, 11, strength, 0, 999, 1_010, 3_030,
        weak, wrath, vulnerable)
}

pub fn full_normal_damage_modifiers(base: i32, strength: i32, weak: bool,
    wrath: bool, vulnerable: bool) -> Result<i32, InputError>
{
    bounded(base, 10_004, "full base damage")?;
    if !(-999..=999).contains(&strength) {
        return Err(InputError::Unsupported("full Strength outside supported numeric surface".into()));
    }
    let total = calculated(base.checked_add(strength))?.max(0);
    bounded(total, 10_004, "full combined damage")?;
    let mut value = total as f32;
    if weak { value *= 0.75f32; }
    if wrath { value *= 2.0f32; }
    if vulnerable { value *= 1.5f32; }
    bounded(value.floor() as i32, 30_012, "full modified damage")?;
    Ok(value.max(0.0).floor() as i32)
}

fn normal_damage_modifiers_bounded(base: i32, maximum_base: i32, strength: i32,
    minimum_strength: i32, maximum_strength: i32, maximum_total: i32, maximum_output: i32,
    weak: bool, wrath: bool, vulnerable: bool) -> Result<i32, InputError>
{
    bounded(base, maximum_base, "base damage")?;
    if !(minimum_strength..=maximum_strength).contains(&strength) {
        return Err(InputError::Unsupported("Strength outside supported numeric surface".into()));
    }
    let total = calculated(base.checked_add(strength))?.max(0);
    bounded(total, maximum_total, "combined damage")?;
    let mut value = total as f32;
    if weak { value *= 0.75f32; }
    if wrath { value *= 2.0f32; }
    if vulnerable { value *= 1.5f32; }
    bounded(value.floor() as i32, maximum_output, "modified damage")?;
    Ok(value.max(0.0).floor() as i32)
}

pub fn block_after_add(current: i32, amount: i32) -> Result<i32, InputError> {
    bounded(current, 999, "block")?;
    bounded(amount, 8, "block gain")?;
    Ok(calculated(current.checked_add(amount))?.min(999)) // Sum <= 1007.
}

/// Bounded block addition for the full native runtime. Keep the legacy helper
/// above unchanged; native effect execution may supply larger card payloads.
pub fn full_block_after_add(current: i32, amount: i32) -> Result<i32, InputError> {
    bounded(current, 999, "block")?;
    bounded(amount, i32::MAX, "native block gain")?;
    Ok((i64::from(current) + i64::from(amount)).min(999) as i32)
}

/// Printed card Block: stat addition, binary32 Frail, then one final floor.
/// Direct Block gains (potions, orbs and enemies) do not call this calculation.
pub fn full_card_block(base: i32, dexterity: i32, frail: bool) -> Result<i32, InputError> {
    bounded(base, 1007, "card Block base")?;
    if !(-999..=999).contains(&dexterity) {
        return Err(InputError::Unsupported("Dexterity outside supported native surface".into()));
    }
    let mut value = (base + dexterity) as f32;
    if frail { value *= 0.75_f32; }
    Ok(value.max(0.0).floor() as i32)
}

pub fn hp_and_block_after_damage(hp: i32, block: i32, damage: i32) -> Result<(i32, i32), InputError> {
    bounded(hp, 999, "HP")?;
    bounded(block, 999, "block")?;
    bounded(damage, 1515, "damage")?;
    let absorbed = block.min(damage);
    // Remaining damage <= 1515; subtraction is >= -1515 before clamping.
    let remaining = calculated(damage.checked_sub(absorbed))?;
    Ok((calculated(hp.checked_sub(remaining))?.max(0), calculated(block.checked_sub(absorbed))?))
}

/// Full-runtime HP/block absorption. The legacy public helper retains its 1,515 cap.
pub fn full_hp_and_block_after_damage(hp: i32, block: i32, damage: i32)
    -> Result<(i32, i32), InputError>
{
    bounded(hp, 999, "HP")?;
    bounded(block, 999, "block")?;
    bounded(damage, 30_012, "full damage")?;
    let absorbed = block.min(damage);
    let remaining = calculated(damage.checked_sub(absorbed))?;
    Ok((calculated(hp.checked_sub(remaining))?.max(0),
        calculated(block.checked_sub(absorbed))?))
}

pub fn clamped_strength(current: i32, delta: i32) -> Result<i32, InputError> {
    bounded(current, 999, "Strength")?;
    bounded(delta, 3, "Strength gain")?;
    Ok(calculated(current.checked_add(delta))?.min(999)) // Sum <= 1002.
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ratio {
    pub numerator: u128,
    pub denominator: u128,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SelectionError {
    Invalid(InputError),
    Random(RngError),
}

impl From<InputError> for SelectionError {
    fn from(error: InputError) -> Self { Self::Invalid(error) }
}

impl From<RngError> for SelectionError {
    fn from(error: RngError) -> Self { Self::Random(error) }
}

fn validate_history(kind: EnemyKind, history: &[MoveId]) -> Result<(), InputError> {
    use MoveId::*;
    let valid = match kind {
        EnemyKind::Cultist => history.is_empty() ||
            (history[0] == Incantation && history[1..].iter().all(|id| *id == Attack)),
        EnemyKind::JawWorm => history.is_empty() ||
            (history[0] == Chomp && history.iter().all(|id| matches!(id, Chomp | Thrash | Bellow))),
    };
    if valid { Ok(()) } else { Err(InputError::Unsupported("enemy/history mismatch".into())) }
}

pub fn jaw_worm_fallback(history: &[MoveId], roll: u8) -> Result<Option<Ratio>, InputError> {
    validate_history(EnemyKind::JawWorm, history)?;
    bounded(i32::from(roll), 99, "AI bucket")?;
    use MoveId::*;
    let ratio = match roll {
        0..=24 if history.last() == Some(&Chomp) => Some((9, 16)),
        25..=54 if history.ends_with(&[Thrash, Thrash]) => Some((2994733, 8388608)),
        55..=99 if history.last() == Some(&Bellow) => Some((3489661, 8388608)),
        _ => None,
    };
    Ok(ratio.map(|(numerator, denominator)| Ratio { numerator, denominator }))
}

pub fn jaw_worm_move(history: &[MoveId], roll: u8, fallback: Option<bool>) -> Result<MoveId, InputError> {
    let required = jaw_worm_fallback(history, roll)?.is_some();
    if required != fallback.is_some() {
        return Err(InputError::Invariant("conditional coin presence does not match restriction".into()));
    }
    use MoveId::*;
    if history.is_empty() { return Ok(Chomp); }
    Ok(match (roll, fallback) {
        (0..=24, Some(true)) => Bellow,
        (0..=24, Some(false)) => Thrash,
        (25..=99, Some(true)) => Chomp,
        (25..=54, Some(false)) => Bellow,
        (55..=99, Some(false)) => Thrash,
        (0..=24, None) => Chomp,
        (25..=54, None) => Thrash,
        (55..=99, None) => Bellow,
        _ => unreachable!("bucket validated before selection"),
    })
}

// Selector-only seam, combat-private. No caller-configurable RNG backend.
pub(super) trait SelectionDraws {
    fn bucket(&mut self) -> Result<u8, RngError>;
    fn coin(&mut self, probability: Ratio) -> Result<bool, RngError>;
}

pub(super) fn select<D: SelectionDraws>(kind: EnemyKind, history: &[MoveId], draws: &mut D)
    -> Result<MoveId, SelectionError>
{
    validate_history(kind, history)?;
    match kind {
        EnemyKind::Cultist => Ok(if history.is_empty() { MoveId::Incantation } else { MoveId::Attack }),
        EnemyKind::JawWorm if history.is_empty() => Ok(MoveId::Chomp),
        EnemyKind::JawWorm => {
            let roll = draws.bucket()?;
            let fallback = match jaw_worm_fallback(history, roll)? {
                Some(probability) => Some(draws.coin(probability)?),
                None => None,
            };
            Ok(jaw_worm_move(history, roll, fallback)?)
        }
    }
}

struct NativeDraws {
    stream: StreamState,
}

impl SelectionDraws for NativeDraws {
    fn bucket(&mut self) -> Result<u8, RngError> {
        let (bucket, next) = rng::uniform_below(&self.stream, 100)?;
        self.stream = next;
        Ok(bucket as u8) // uniform_below(100) is in 0..=99.
    }

    fn coin(&mut self, probability: Ratio) -> Result<bool, RngError> {
        let (coin, next) = rng::bernoulli(&self.stream, probability.numerator, probability.denominator)?;
        self.stream = next;
        Ok(coin)
    }
}

pub fn choose_enemy_move(kind: EnemyKind, history: &[MoveId], stream: &StreamState)
    -> Result<(MoveId, StreamState), SelectionError>
{
    if !matches!(stream.id().domain(), Domain::CombatEnemyAi | Domain::RunCombatEnemyAi) {
        return Err(SelectionError::Invalid(InputError::Unsupported("selection requires enemy AI domain".into())));
    }
    let mut draws = NativeDraws { stream: stream.clone() };
    let chosen = select(kind, history, &mut draws)?;
    // Native RNG errors already carry the latest cursor, including rejection
    // consumption and a completed bucket followed by fallback exhaustion.
    Ok((chosen, draws.stream))
}
