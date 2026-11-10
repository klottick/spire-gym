//! M34s-c synthetic positions `spire-gym-combat-position-v1` (design §4.8; c0 `task0-v1.md` §2–§4). A position describes
//! the state at the opening decision of a native `standard-combat-v1` start and is applied in two places, so every mechanic
//! sees a native-consistent history:
//! 1. enemy fields at setup (`enemies::setup_with_overrides`): initial HP and listed powers, before the opening move
//!    selection, so the opening intent is chosen from the edited state and no damage event is implied;
//! 2. player and pile fields after the native opening ([`Combat::apply_position`]): one owner, on a scratch copy, per the
//!    overwrite table; listed enemy powers are re-asserted and enemy block set there.
//! Every static check ([`Position::validate`]) runs before any state change. The position never carries hidden state: the
//! draw order is shuffled on the session's shuffle stream, hand costs are rolled on its card stream, and public knowledge
//! is reset to the new piles.
use super::model::{OwnedCard, Phase, State};
use super::*;
use crate::content::{CardId, Character, EncounterId, EnemyId};

pub const POSITION_SCHEMA_ID: &str = "spire-gym-combat-position-v1";

/// A rejected position: `Input` names the offending field; `Unsupported` is a boundary the position cannot describe.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PositionError { Input(&'static str), Unsupported(&'static str) }

/// One pile row: a deck card (`generated` false; matched to the entry deck by (card, upgrades, misc)) or a card the combat
/// generates (from the character's generated allowlist).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PositionCard { pub id: CardId, pub upgrades: u32, pub misc: i32, pub generated: bool }
/// Every pile, replacing the native opening's: `hand` left to right, `draw_top` top first (known), `draw` an unordered
/// multiset (shuffled by the gym), `discard` and `exhaust` oldest first.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PositionPiles { pub hand: Vec<PositionCard>, pub draw_top: Vec<PositionCard>, pub draw: Vec<PositionCard>, pub discard: Vec<PositionCard>, pub exhaust: Vec<PositionCard> }
/// One formation slot's fields: setup HP, block at the opening decision, listed powers (amount 0 removes). `slot` is the
/// formation index ([`super::enemies::position_formation`]: the setup order), not the view's logical slot.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EnemyFields { pub slot: u8, pub hp: Option<i32>, pub block: Option<i32>, pub powers: Vec<(FullPowerId, i32)> }
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Position {
    pub enemies: Vec<EnemyFields>,
    /// Player powers (amount 0 removes); unlisted powers keep their native opening values.
    pub powers: Vec<(FullPowerId, i32)>,
    pub block: Option<i32>, pub energy: Option<i32>, pub stance: Option<Stance>, pub mantra: Option<i32>,
    pub orb_slots: Option<u8>, pub orbs: Option<Vec<OrbKind>>,
    pub piles: Option<PositionPiles>,
}

/// The registered power id of `name`.
pub fn power_id(name: &str) -> Option<FullPowerId> { public_ids::PUBLIC_POWERS.iter().find(|p| **p == name).map(|p| FullPowerId(p)) }

/// c0 §4: player powers whose whole state is the amount (fail closed; draw-trigger, separate-owner and skip-first powers
/// are absent; Mantra is the `mantra` scalar, Focus the character owner).
pub const PLAYER_POWERS: &[&str] = &["strength", "dexterity", "focus", "weak", "vulnerable", "frail", "artifact", "thorns", "metallicize",
    "plated_armor", "vigor", "demon_form", "feel_no_pain", "dark_embrace", "barricade", "juggernaut", "rage", "noxious_fumes",
    "a_thousand_cuts", "after_image", "envenom", "accuracy", "infinite_blades", "storm", "static_discharge", "heatsinks", "loop",
    "mental_fortress", "rushdown", "like_water", "devotion", "battle_hymn", "energized", "draw_card_next_turn", "next_turn_block"];
/// c0 §3: powers a native start can give any enemy (Philosopher's Stone, Bag of Marbles, Red Mask, Twisted Funnel).
const ENEMY_START_RELIC_POWERS: &[&str] = &["strength", "vulnerable", "weak", "poison"];
/// c0 §3, final review m3: the power a declared burning-elite buff installs (one buff, on every member, one amount).
fn burning_power(burning: Option<crate::run::BurningEliteBuff>) -> Option<&'static str> {
    match burning { Some(crate::run::BurningEliteBuff::Metallicize) => Some("metallicize"), Some(crate::run::BurningEliteBuff::Regeneration) => Some("regenerate"), _ => None }
}
/// c0 §3: a species' native amount-only setup powers (structural and move-only powers are never admitted).
fn species_powers(id: EnemyId) -> &'static [&'static str] {
    match id {
        EnemyId::Deca | EnemyId::Donu | EnemyId::SpireShield | EnemyId::SpireSpear | EnemyId::BronzeAutomaton | EnemyId::SphericGuardian
            | EnemyId::Sentry => &["artifact"],
        EnemyId::Spiker => &["thorns"], EnemyId::OrbWalker => &["strength_up"], EnemyId::FungiBeast => &["spore_cloud"],
        EnemyId::ShelledParasite => &["plated_armor"], EnemyId::Lagavulin => &["metallicize"],
        _ => &[],
    }
}
/// c0 §4: cards a Standard combat of `character` can generate that a position may list (statuses everywhere; the
/// character's own generated cards).
fn generatable(character: Character, id: CardId) -> bool {
    matches!(id, CardId::Wound | CardId::Dazed | CardId::Burn | CardId::Slimed | CardId::Void) || match character {
        Character::Silent => id == CardId::Shiv,
        Character::Watcher => matches!(id, CardId::Miracle | CardId::Insight | CardId::Smite | CardId::Safety | CardId::ThroughViolence
            | CardId::Expunger | CardId::Beta | CardId::Omega),
        _ => false,
    }
}
/// A listed amount is 0 (remove) or inside the engine's own power domain (`valid_full_powers`).
fn in_domain(id: FullPowerId, amount: i32) -> bool {
    amount == 0 || super::engine::valid_full_powers(&[FullPowerEntry { id, state: PowerState { amount, skip_first: false, monster_source_grace: false } }])
}
fn unique<T: PartialEq>(items: impl Iterator<Item = T>) -> bool { let v = items.collect::<Vec<_>>(); v.iter().enumerate().all(|(i, a)| !v[..i].contains(a)) }

impl Position {
    /// The static checks (design §4.8: before any state change): every field's domain, the allowlists, the formation and
    /// the deck rows. `deck` is the entry's owned master deck.
    pub fn validate(&self, character: Character, ascension: u8, encounter: EncounterId, elite: bool, burning: Option<crate::run::BurningEliteBuff>,
        deck: &[crate::run::CardInstance]) -> Result<(), PositionError> {
        use PositionError::Input;
        if !self.enemies.is_empty() {
            let formation = super::enemies::position_formation(encounter)
                .ok_or(Input("enemies: enemy fields require a fixed formation with seed-independent species"))?;
            let buff = burning_power(burning).filter(|_| elite);
            if !unique(self.enemies.iter().map(|e| e.slot)) { return Err(Input("enemies.slot: duplicated slot")) }
            for e in &self.enemies {
                let species = *formation.get(e.slot as usize).ok_or(Input("enemies.slot: outside the formation"))?;
                if e.hp.is_some_and(|hp| !(1..=super::enemies::hp_range(species, ascension).0).contains(&hp)) { return Err(Input("enemies.hp: outside 1..=the species' minimum max HP")) }
                if e.block.is_some_and(|b| !(0..=999).contains(&b)) { return Err(Input("enemies.block: outside 0..=999")) }
                if !unique(e.powers.iter().map(|p| p.0)) { return Err(Input("enemies.powers: duplicated power")) }
                for (id, amount) in &e.powers {
                    let admitted = ENEMY_START_RELIC_POWERS.contains(&id.0) || buff == Some(id.0) || species_powers(species).contains(&id.0);
                    if !admitted { return Err(Input("enemies.powers: not start-applicable for the species")) }
                    if !in_domain(*id, *amount) { return Err(Input("enemies.powers: amount out of domain")) }
                }
            }
            if let Some(buff) = buff {
                let amounts = self.enemies.iter().filter_map(|e| e.powers.iter().find(|p| p.0 .0 == buff).map(|p| p.1)).collect::<Vec<_>>();
                if !amounts.is_empty() && (amounts.len() != formation.len() || amounts.iter().any(|a| *a != amounts[0])) {
                    return Err(Input("enemies.powers: a burning buff is listed on every member with one amount"))
                }
            }
        }
        if !unique(self.powers.iter().map(|p| p.0)) { return Err(Input("player.powers: duplicated power")) }
        for (id, amount) in &self.powers {
            if !PLAYER_POWERS.contains(&id.0) { return Err(Input("player.powers: power not admitted")) }
            if !in_domain(*id, *amount) { return Err(Input("player.powers: amount out of domain")) }
        }
        if self.block.is_some_and(|b| !(0..=999).contains(&b)) { return Err(Input("player.block: outside 0..=999")) }
        if self.energy.is_some_and(|e| !(0..=999).contains(&e)) { return Err(Input("player.energy: outside 0..=999")) }
        if self.mantra.is_some_and(|m| !(0..=9).contains(&m)) { return Err(Input("player.mantra: outside 0..=9")) }
        if self.orb_slots.is_some_and(|s| !(1..=10).contains(&s)) { return Err(Input("player.orb_slots: outside 1..=10")) }
        if self.orb_slots.is_some() != self.orbs.is_some() { return Err(Input("player.orbs: orb_slots and orbs are listed together")) }
        if let (Some(slots), Some(orbs)) = (self.orb_slots, &self.orbs) { if orbs.len() > slots as usize { return Err(Input("player.orbs: more orbs than slots")) } }
        if let Some(p) = &self.piles {
            if p.hand.len() > 10 { return Err(Input("piles.hand: more than 10 cards")) }
            let rows = || p.hand.iter().chain(&p.draw_top).chain(&p.draw).chain(&p.discard).chain(&p.exhaust);
            if rows().count() > 999 { return Err(Input("piles: more than 999 cards")) }
            let mut wanted = deck.iter().map(|c| (c.id.as_str(), c.upgrades, c.misc)).collect::<Vec<_>>();
            for row in rows() {
                if row.generated {
                    if !generatable(character, row.id) { return Err(Input("piles: generated card not admitted")) }
                    if row.misc != 0 || row.upgrades > 1 || super::card_state::generated(row.id, CardInstanceId(1), row.upgrades).is_none() {
                        return Err(Input("piles: generated card upgrades or misc out of range"))
                    }
                } else {
                    let at = wanted.iter().position(|w| *w == (row.id.as_str(), row.upgrades, row.misc)).ok_or(Input("piles: deck row not in the deck (extra or duplicated)"))?;
                    wanted.swap_remove(at);
                }
            }
            if !wanted.is_empty() { return Err(Input("piles: a deck card is missing")) }
            if p.hand.iter().any(|c| matches!(c.id, CardId::Void | CardId::EndlessAgony)) { return Err(Input("piles.hand: an on-draw card (Void, Endless Agony)")) }
        }
        Ok(())
    }
    /// The setup half (design §4.8 (1)).
    pub(crate) fn setup_overrides(&self) -> Vec<super::enemies::EnemySetupOverride> {
        self.enemies.iter().map(|e| super::enemies::EnemySetupOverride { slot: e.slot as usize, hp: e.hp, powers: e.powers.clone() }).collect()
    }
}

fn set_power(powers: &mut Vec<FullPowerEntry>, id: FullPowerId, amount: i32) {
    match powers.iter_mut().find(|p| p.id == id) {
        Some(p) if amount != 0 => p.state = PowerState { amount, skip_first: false, monster_source_grace: false },
        Some(_) => powers.retain(|p| p.id != id),
        None if amount != 0 => powers.push(FullPowerEntry { id, state: PowerState { amount, skip_first: false, monster_source_grace: false } }),
        None => {}
    }
}
/// The enemy at formation index `slot` (final review I2: the index used at validation and setup). Enemies are append-only
/// and a fixed formation spawns nothing before its opening decision, so the combat's enemies start in setup order.
fn formation_enemy(s: &mut State, slot: u8) -> Option<&mut EnemyState> { s.enemies.get_mut(slot as usize) }
/// The opening decision is a plain main decision: no selection, offer, stance choice or queued work.
fn at_main_decision(state: &State) -> bool {
    state.phase == Phase::Decision && state.pending.is_empty() && state.current.is_none() && state.pending_selection.is_none()
        && state.silent.card_offer.is_none() && state.watcher.stance_choice.is_none() && state.outcome.is_none() && state.fault.is_none()
}

impl Combat {
    /// Design §4.8 (2): this native opening with the position applied, on a scratch copy (a rejection changes nothing).
    /// `deck` is the entry's master deck (permanent instances). Requires `Position::validate` to have passed.
    pub(crate) fn apply_position(&self, p: &Position, deck: &[crate::run::CardInstance]) -> Result<Combat, PositionError> {
        use PositionError::{Input, Unsupported};
        if !at_main_decision(&self.state) { return Err(Unsupported("a position requires the main opening decision")) }
        let mut combat = self.clone();
        let s = &mut combat.state;
        // Enemies: listed powers re-asserted (final = listed), block set; edited enemies must still be alive.
        for e in &p.enemies {
            let enemy = formation_enemy(s, e.slot).ok_or(Input("enemies.slot: outside the formation"))?;
            if !enemy.targetable() { return Err(Input("enemies: an edited enemy is not alive at the opening decision")) }
            for (id, amount) in &e.powers { set_power(&mut enemy.full_powers, *id, *amount); }
            if let Some(block) = e.block { enemy.block = block; }
        }
        for (id, amount) in &p.powers {
            if id.0 == "focus" { s.character.focus = *amount; } else { set_power(&mut s.player_full_powers, *id, *amount); }
        }
        if let Some(block) = p.block { s.player.block = block; }
        if let Some(energy) = p.energy { s.player.energy = energy; }
        if p.stance.is_some() || p.mantra.is_some() {
            if !super::cards::watcher::cards_active(s) { return Err(Input("player.stance: stance and mantra need Watcher card laws")) }
            if let Some(stance) = p.stance { s.character.stance = stance; }
            if let Some(mantra) = p.mantra { set_power(&mut s.player_full_powers, FullPowerId("mantra"), mantra); }
        }
        if let (Some(slots), Some(orbs)) = (p.orb_slots, &p.orbs) {
            if s.character.orb_slots.is_empty() { return Err(Input("player.orbs: this combat has no orb slots")) }
            s.character.orb_slots = (0..slots as usize).map(|i| orbs.get(i).map(|k| OrbState::new(*k))).collect();
            // PM ruling c0 (5): every listed orb was channelled this combat, none evoked.
            s.defect.frost_channeled = orbs.iter().filter(|k| **k == OrbKind::Frost).count() as u32;
            s.defect.lightning_channeled = orbs.iter().filter(|k| **k == OrbKind::Lightning).count() as u32;
        }
        if !(s.silent.played_this_turn.is_empty() && s.silent.discarded_this_turn == 0 && s.played_card_types.is_empty()
            && s.played_powers.is_empty() && s.inactive_cards.is_empty() && s.stasis.is_empty() && s.in_play.is_none() && s.queued_autoplay.is_empty()) {
            return Err(Unsupported("the native opening holds turn history or custody a position cannot describe"))
        }
        if let Some(piles) = &p.piles { replace_piles(s, piles, deck)?; }
        // Final review m2: the engine runs the hand-emptied seam (Unceasing Top) before publishing a decision with an empty
        // hand, so an empty hand that seam would refill is never a native decision.
        if super::engine::hand_emptied_work(&mut s.clone()).is_some_and(|w| w.map_or(true, |w| !w.is_empty())) {
            return Err(Input("piles.hand: an empty hand the hand-emptied seam refills (Unceasing Top) is never a native decision"))
        }
        // Derived values, then public knowledge: pre-roll sets are re-observed at this (published) boundary only.
        if let Some(amount) = s.player_full_powers.iter().find(|x| x.id.0 == "accuracy").map(|x| x.state.amount) { super::cards::rewrite_accuracy_shivs(s, amount); }
        super::piles::refresh_hand_values(s);
        super::effects::refresh_enemy_values(s).map_err(|_| Input("position: derived values could not be refreshed"))?;
        s.knowledge.prerolls = Default::default();
        super::knowledge::observe_prerolls(s);
        super::engine::validate_runtime(s).map_err(|_| Input("position: not a consistent combat state"))?;
        combat.refresh_legacy_cache();
        Ok(combat)
    }
}

/// Overwrite table, piles: every native card leaves; non-deck cards are de-issued; deck cards are rebuilt fresh (entry
/// initialisation) and placed; generated rows are issued anew; the draw multiset is shuffled on the session shuffle stream
/// and `draw_top` top-decked over it (known); each hand card passes the shared draw callback in hand order.
fn replace_piles(s: &mut State, p: &PositionPiles, deck: &[crate::run::CardInstance]) -> Result<(), PositionError> {
    use PositionError::{Input, Unsupported};
    let character = s.silent.source_character.unwrap_or(Character::Ironclad);
    if [&p.hand, &p.draw_top, &p.draw, &p.discard, &p.exhaust].into_iter().flatten().any(|r| r.generated && !generatable(character, r.id)) {
        return Err(Input("piles: generated card not admitted"))
    }
    // Fresh deck cards, with the instances a native opening forces into the hand first among identical rows.
    let mut fresh = Vec::new();
    for c in deck {
        let mut live = card_state::permanent_with_misc(c.id, CardInstanceId(c.instance), c.upgrades, c.misc);
        super::cards::initialize_complete(&mut live).map_err(|_| Input("piles: a deck card cannot be initialised"))?;
        let forced = super::piles::opening_first(s, &live);
        fresh.push((forced, live));
    }
    fresh.sort_by_key(|(forced, c)| (!*forced, c.instance.0));
    let forced = fresh.iter().filter(|(f, _)| *f).map(|(_, c)| c.instance).collect::<Vec<_>>();
    s.piles = Default::default();
    s.card_ids_issued = fresh.iter().map(|(_, c)| card_state::handle(c.instance)).collect();
    s.public = super::model::PublicKnowledge { revealed_order: vec![], known_draw: vec![], known_draw_bottom: vec![] };
    let mut take = |s: &mut State, row: &PositionCard| -> Result<OwnedCard, PositionError> {
        if row.generated {
            let instance = super::piles::next_generated_instance(s).map_err(|_| Input("piles: generated card identity exhausted"))?;
            let card = card_state::generated_with_history(row.id, instance, row.upgrades, card_state::CardConstructionHistory::from_state(s))
                .ok_or(Input("piles: generated card upgrades or misc out of range"))?;
            s.card_ids_issued.insert(card_state::handle(instance));
            Ok(OwnedCard::Full(card))
        } else {
            let at = fresh.iter().position(|(_, c)| (c.id, c.upgrades, c.misc) == (row.id, row.upgrades, row.misc)).ok_or(Input("piles: deck row not in the deck (extra or duplicated)"))?;
            Ok(OwnedCard::Full(fresh.remove(at).1))
        }
    };
    let hand = p.hand.iter().map(|r| take(s, r)).collect::<Result<Vec<_>, _>>()?;
    let top = p.draw_top.iter().map(|r| take(s, r)).collect::<Result<Vec<_>, _>>()?;
    let mut draw = p.draw.iter().map(|r| take(s, r)).collect::<Result<Vec<_>, _>>()?;
    s.piles.discard = p.discard.iter().map(|r| take(s, r)).collect::<Result<Vec<_>, _>>()?;
    s.piles.exhaust = p.exhaust.iter().map(|r| take(s, r)).collect::<Result<Vec<_>, _>>()?;
    if !fresh.is_empty() { return Err(Input("piles: a deck card is missing")) }
    if forced.iter().any(|i| !hand.iter().any(|c| c.full().is_some_and(|c| c.instance == *i))) {
        return Err(Input("piles.hand: an innate or bottled deck card must be in the hand"))
    }
    // The gym owns the order: canonical multiset order, then the session's shuffle stream.
    draw.sort_by_key(|c| { let c = c.full().unwrap(); (c.id.as_str(), c.upgrades, c.misc, c.instance.0) });
    let (shuffled, next) = rng::shuffle(&s.rng.shuffle, &draw).map_err(|_| Unsupported("position draw shuffle failed"))?;
    s.rng.shuffle = next;
    s.piles.draw = shuffled;
    super::knowledge::note(s, super::knowledge::PileOp::Shuffled);
    for card in top.into_iter().rev() { let handle = card.handle(); super::piles::topdeck_owned_card(s, card, handle); }
    // The hand, as if drawn: history, reveal, then the shared draw callback (Confused/Snecko costs on the card stream,
    // Eviscerate's price); other on-draw work is rejected (v1).
    let evolve = s.player_full_powers.iter().any(|x| matches!(x.id.0, "evolve" | "fire_breathing") && x.state.amount > 0);
    let typed = |id: CardId| crate::content::Catalog::standard().card(id).map(|d| d.card_type);
    if evolve && hand.iter().any(|c| matches!(typed(c.full().unwrap().id), Some(crate::content::CardType::Status | crate::content::CardType::Curse))) {
        return Err(Input("piles.hand: a status or curse under a draw-trigger power (Evolve, Fire Breathing)"))
    }
    s.silent.drawn_cards.clear();
    for card in hand {
        let handle = card.handle();
        if s.silent.history_enabled { let c = card.full().unwrap(); s.silent.drawn_cards.push(super::cards::DrawnCard { id: c.id, instance: Some(c.instance) }); }
        s.public.revealed_order.push(handle.clone());
        s.piles.hand.push(card);
        super::piles::after_draw(s, &handle).map_err(|_| Input("piles.hand: a hand card's draw callback failed"))?;
        if !s.pending.is_empty() { return Err(Input("piles.hand: a hand card's draw callback queues work (not admitted in v1)")) }
    }
    Ok(())
}
