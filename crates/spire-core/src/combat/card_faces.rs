//! Static public card faces for the opt-in card-metadata descriptor (spire-90f).
//!
//! A read-only probe of the existing constructor, cost and payload owners under one
//! neutral complete-card state: one full-HP Cultist, no powers, relics or potions, three energy. It never
//! steps a combat, samples no RNG and exports no state. It is not a rules table:
//! a live card view (temporary cost, upgrades, powers, targets) stays authoritative.
use super::model::State;
use super::rng::{make_stream, Domain, Partition, StreamId, RNG_CONTRACT_ID};
use super::*;
use crate::content::{CardColor, CardId, CardType, Catalog, Character};

/// Printed cost of a fresh card in the neutral probe state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StaticCost { Energy(i32), X, Unplayable, Unresolved }

/// Whether the owning payload resolves without or only with an enemy target.
/// `Unresolved` covers unplayable kinds and play conditions the probe state fails.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StaticTarget { Enemy, Untargeted, Unresolved }

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StaticCardFace {
    pub cost: StaticCost,
    pub target: StaticTarget,
    pub exhaust: bool,
    pub ethereal: bool,
    pub innate: bool,
    pub retain: bool,
    /// Printed numbers from the public value owner (`cards::public_values`), else the constructor.
    pub base_damage: Option<i32>,
    pub base_block: Option<i32>,
    pub magic_number: Option<i32>,
}

/// The face of `id` at `upgrades` through the complete (M26) constructor.
pub fn static_card_face(id: CardId, upgrades: u32) -> Result<StaticCardFace, InputError> {
    let card = complete_live_card(id, CardInstanceId(1), upgrades)?;
    let color = Catalog::standard().card(id).ok_or_else(|| InputError::Unsupported("card without catalog row".into()))?.color;
    let state = probe_state(match color {
        CardColor::Green => Character::Silent, CardColor::Blue => Character::Defect,
        CardColor::Purple => Character::Watcher, _ => Character::Ironclad,
    })?;
    let target = if cards::payload_with_wrist_blade(&state, &card, None).is_ok() { StaticTarget::Untargeted }
        else if cards::payload_with_wrist_blade(&state, &card, Some(1)).is_ok() { StaticTarget::Enemy }
        else { StaticTarget::Unresolved };
    // The live card view's public value owner; constructor fields cover kinds it does not project.
    let values = cards::public_values(&card);
    let kind = values.as_ref().map(|values| values.cost_kind);
    let unplayable_kind = Catalog::standard().card(id).is_some_and(|row| matches!(row.card_type, CardType::Status | CardType::Curse));
    let cost = if card.local.x_cost || kind == Some(CardCostKind::X) { StaticCost::X }
        else if kind == Some(CardCostKind::Unplayable) { StaticCost::Unplayable }
        else { match cards::base_cost(&state, &card) {
            Some(cost) => StaticCost::Energy(cost),
            None if unplayable_kind && target == StaticTarget::Unresolved => StaticCost::Unplayable,
            None => StaticCost::Unresolved,
        } };
    let local = &card.local;
    Ok(StaticCardFace { cost, target, exhaust: local.exhaust_on_finish, ethereal: local.ethereal, innate: local.innate,
        retain: local.retain || local.self_retain,
        base_damage: values.as_ref().and_then(|values| values.base_damage).or(local.base_damage),
        base_block: values.as_ref().and_then(|values| values.base_block).or(local.base_block),
        magic_number: values.as_ref().and_then(|values| values.magic_number).or(local.magic_number) })
}

fn probe_state(character: Character) -> Result<State, InputError> {
    let stream = |domain, scope: &[&str]| make_stream(&[0; 32], Partition::Test, "card-metadata-probe",
        &StreamId::new(domain, scope.iter().map(|field| (*field).to_owned()).collect()).expect("static probe stream id"))
        .expect("static probe stream");
    let enemy = EnemyHandle::parse("enemy-1").expect("static probe enemy");
    let scenario = Scenario {
        encounter_id: EnemyKind::Cultist,
        player: PlayerState { hp: 50, max_hp: 50, block: 0, energy: 3, max_energy: 3, draw_per_turn: 5, powers: vec![] },
        enemies: vec![EnemySetup { instance_id: enemy.clone(), enemy_id: EnemyKind::Cultist, hp: 48, max_hp: 48, block: 0,
            powers: vec![], intent_id: MoveId::Incantation, move_history: vec![MoveId::Incantation] }],
        piles: Piles { hand: vec![CardInstance { instance_id: CardHandle::parse("1").expect("static probe card"),
            card_id: CardKind::Strike, upgraded: false }], draw: vec![], discard: vec![], exhaust: vec![] },
        rng: CombatRng { contract_id: RNG_CONTRACT_ID.into(), shuffle: stream(Domain::CombatShuffle, &["probe"]),
            enemy_ai: stream(Domain::CombatEnemyAi, &["probe", "enemy-1"]), enemy_instance_id: enemy },
    };
    let mut state = Combat::new(&scenario)?.state;
    state.full_runtime = true;
    state.character = characters::CharacterState::for_character(character);
    state.item_context = Some(crate::items::CombatItemContext::new(vec![], vec![]).with_complete_cards());
    Ok(state)
}

/// Upgrade limit of the complete constructor: `None` is unbounded (Searing Blow).
/// A finite profile may admit fewer; its own support descriptor is the authority.
pub fn static_card_upgrade_limit(id: CardId) -> Option<u32> {
    if crate::run::complete_upgrades_valid(id, 2) { None }
    else if crate::run::complete_upgrades_valid(id, 1) { Some(1) } else { Some(0) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn neutral_faces_come_from_the_native_owners_and_live_cost_stays_separate() {
        let strike = static_card_face(CardId::Strike, 0).unwrap();
        assert_eq!((strike.cost, strike.target), (StaticCost::Energy(1), StaticTarget::Enemy));
        assert_eq!(static_card_face(CardId::Defend, 1).unwrap().target, StaticTarget::Untargeted);
        assert_eq!(static_card_face(CardId::Whirlwind, 0).unwrap().cost, StaticCost::X);
        assert_eq!(static_card_face(CardId::Wound, 0).unwrap().cost, StaticCost::Unplayable);
        assert!(static_card_face(CardId::Wound, 1).is_err());
        assert_eq!((static_card_upgrade_limit(CardId::SearingBlow), static_card_upgrade_limit(CardId::Strike),
            static_card_upgrade_limit(CardId::Wound)), (None, Some(1), Some(0)));
        // A live temporary reduction changes the effective cost, never the static face.
        let mut live = complete_live_card(CardId::Strike, CardInstanceId(1), 0).unwrap();
        live.local.temporary_cost = -1;
        assert_eq!(card_state::effective_cost(&live, 1), Ok(0));
        assert_eq!(static_card_face(CardId::Strike, 0).unwrap(), strike);
    }
}
