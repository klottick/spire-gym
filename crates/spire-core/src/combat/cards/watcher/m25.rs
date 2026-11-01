//! Complete-Watcher (M25) construction admission on the shared physical card owner.
//! Costs and values are the design §5.1/§5.2 literals (base → upgraded). The finite
//! M21 kinds keep their accepted initialization byte-for-byte.
use super::*;

/// Source cost marker for an X-cost card (costForTurn -1).
const X_COST: i32 = -1;

pub(super) fn m25_cost(id: crate::content::CardId, up: bool) -> Option<i32> {
    use crate::content::CardId::*;
    Some(match id {
        Consecrate | Halt | ForeignInfluence | Insight | ThroughViolence => 0,
        Crescendo | Tranquility | Scrawl => if up { 0 } else { 1 },
        BowlingBash | CrushJoints | EmptyBody | Evaluate | FlyingSleeves | FollowUp | PressurePoints | SashWhip
        | Conclude | EmptyMind | FearNoEvil | Indignation | InnerPeace | Pray | Sanctity | SimmeringFury
        | TalkToTheHand | Tantrum | WaveOfTheHand | WreathOfFlame | Alpha | Blasphemy | Judgment | Expunger => 1,
        Protect | Fasting | ReachHeaven | SignatureMove | Swivel | Wallop | WheelKick | LessonLearned | SpiritShield => 2,
        Study | Beta => if up { 1 } else { 2 },
        DevaForm | Ragnarok | Wish | Omega => 3,
        Vault => if up { 2 } else { 3 },
        Omniscience => if up { 3 } else { 4 },
        Collect | ConjureBlade => X_COST,
        // Deus Ex Machina is unplayable: no cost on either face.
        _ => return None,
    })
}

/// Admit a card under the complete-Watcher capability. Curses keep the shared
/// constructor's unplayable fields (M24's rule); the finite M21 kinds use the
/// accepted M21 initializer; the M25 additions get their Watcher fields here.
pub(in crate::combat) fn initialize_m25_profile(card: &mut LiveCard) -> Result<(), InputError> {
    use crate::content::CardId::*;
    if crate::content::Catalog::standard().card(card.id).is_some_and(|def| def.card_type == crate::content::CardType::Curse) { return Ok(()) }
    if crate::content::watcher::supports_generated(card.id) { return initialize_profile(card) }
    if !crate::content::watcher::supports_m25_generated(card.id) || card.upgrades > 1 {
        return Err(InputError::Unsupported("card outside complete M25 construction profile".into()))
    }
    let up = card.upgrades > 0;
    card.local.m21 = true;
    let cost = m25_cost(card.id, up);
    card.local.mutable_combat_cost = cost;
    card.local.turn_cost = cost;
    card.local.x_cost = matches!(card.id, Collect | ConjureBlade);
    card.local.self_retain = matches!(card.id, Crescendo | FlyingSleeves | Protect | Tranquility | Insight | ThroughViolence) || card.id == Blasphemy && up;
    card.local.exhaust_on_finish = matches!(card.id, Crescendo | Tranquility | Collect | ForeignInfluence | TalkToTheHand | Alpha | Blasphemy
        | ConjureBlade | DeusExMachina | LessonLearned | Omniscience | Scrawl | Vault | Wish | Insight | ThroughViolence | Beta);
    card.local.innate = card.id == Alpha && up;
    card.local.ethereal = card.id == DevaForm && !up;
    let pick = |base: i32, upgraded: i32| Some(if up { upgraded } else { base });
    card.local.base_damage = match card.id {
        BowlingBash => pick(7, 10), Consecrate => pick(5, 8), CrushJoints => pick(8, 10), FlyingSleeves => pick(4, 6),
        FollowUp => pick(7, 11), SashWhip => pick(8, 10), Conclude => pick(12, 16), FearNoEvil => pick(8, 11),
        ReachHeaven => pick(10, 15), SignatureMove => pick(30, 40), TalkToTheHand => pick(5, 7), Tantrum => Some(3),
        Wallop => pick(9, 12), WheelKick => pick(15, 20), LessonLearned => pick(10, 13), Ragnarok => pick(5, 6),
        ThroughViolence => pick(20, 30), Expunger => pick(9, 15), _ => None,
    };
    card.local.base_block = match card.id {
        EmptyBody => pick(7, 10), Evaluate => pick(6, 10), Halt => pick(3, 4), Protect => pick(12, 16),
        Sanctity => pick(6, 9), Swivel => pick(8, 11), _ => None,
    };
    card.local.magic_number = match card.id {
        CrushJoints | SashWhip | WaveOfTheHand => pick(1, 2), Halt => pick(9, 14), PressurePoints => pick(8, 11),
        EmptyMind | SimmeringFury | DeusExMachina | Insight => pick(2, 3), Fasting | InnerPeace | Pray | SpiritShield | Tantrum => pick(3, 4),
        Indignation => pick(3, 5), Sanctity | WheelKick => Some(2), TalkToTheHand => pick(2, 3), WreathOfFlame => pick(5, 8),
        Judgment => pick(30, 40), Ragnarok => pick(5, 6), Omega => pick(50, 60), _ => None,
    };
    Ok(())
}
