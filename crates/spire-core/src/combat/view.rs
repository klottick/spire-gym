//! Public values; never inspect private random or future-AI state.
use super::*;
use super::model::{OwnedCard, State};
use crate::content::EnemyId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CardView { pub instance_id: CardHandle, pub card_id: CardKind, pub upgraded: bool }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlayerView {
    pub hp: i32, pub max_hp: i32, pub block: i32, pub energy: i32, pub max_energy: i32,
    pub powers: Vec<(PowerId, i32)>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnemyView {
    pub instance_id: EnemyHandle, pub enemy_id: EnemyKind, pub hp: i32, pub max_hp: i32,
    pub block: i32, pub powers: Vec<(PowerId, i32)>, pub intent_id: MoveId, pub intent_damage: Option<i32>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicView {
    pub content_id: String, pub turn: u32, pub player: PlayerView, pub enemies: Vec<EnemyView>,
    pub hand: Vec<CardView>, pub draw_count: usize, pub discard: Vec<CardView>, pub exhaust: Vec<CardView>,
    pub revealed_order: Vec<CardHandle>, pub outcome: Option<Outcome>,
    pub hp_before_healing: Option<i32>, pub hp_after_cleanup: Option<i32>,
}
#[derive(Clone, PartialEq, Eq)]
pub struct LiveCardView { pub instance: CardInstanceId, pub id: crate::content::CardId,
    pub origin: CombatCardRef, pub upgrades: u32, pub misc:i32, pub watcher:Option<WatcherCardValues>, pub effective_cost:Option<i32>,pub values:Option<CardPublicValues> }
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct WatcherCardValues{pub combat_cost:Option<i32>,pub turn_cost:Option<i32>,pub self_retain:bool,pub retain:bool}
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct StanceChoiceView{pub options:Vec<super::Stance>}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum CardCostKind { Fixed,X,Unplayable }
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct CardPublicValues {
    pub base_damage:Option<i32>,pub base_block:Option<i32>,pub magic_number:Option<i32>,pub damage_bonus:Option<i32>,
    pub cost_kind:CardCostKind,pub free_play:bool,pub exhaust_on_finish:bool,pub ethereal:bool,pub innate:bool,pub temporary_cost:i32,
}
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct StasisCardView {pub orb:EnemyHandle,pub card:LiveCardView}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectionView {
    pub parent:super::SelectionParent,
    pub operation:super::SelectionOperation,
    pub candidates: Vec<LiveCardView>, pub required: usize,
    pub selected: Vec<CardInstanceId>, pub disposition: super::SelectionDisposition,
    pub source_zone:super::SelectionSourceZone,pub minimum:usize,pub maximum:usize,
}
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct KnownDrawCardView { pub card:LiveCardView,pub from_top:bool,pub offset:usize }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CombatInventoryView {
    pub relic_counters:Vec<crate::items::RelicCounterView>,
    pub relics: Vec<crate::content::RelicId>, pub potions: Vec<Option<crate::content::PotionId>>,
}
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct CombatCardOfferView { pub cards:Vec<crate::content::CardId>,pub optional:bool }
#[derive(Clone, PartialEq, Eq)]
pub struct CombatPublic {
    pub mantra_gained:Option<i32>,
    pub stance_choice:Option<StanceChoiceView>,
    pub played_powers:Vec<LiveCardView>,
    /// Cards whose accepted use has started, in suspended-parent to active-child order.
    pub playing_cards:Vec<LiveCardView>,
    pub stasis:Vec<StasisCardView>,
    pub content_id: String, pub turn: u32, pub player: FullPlayerView, pub enemies: Vec<FullEnemyView>,
    pub character: CharacterCombatView,
    pub hand: Vec<LiveCardView>, pub draw_count: usize, pub discard: Vec<LiveCardView>,
    pub exhaust: Vec<LiveCardView>, pub outcome: Option<Outcome>,
    pub hp_before_healing: Option<i32>, pub hp_after_cleanup: Option<i32>,
    pub targets: Vec<(TargetId, bool)>,
    pub selection: Option<SelectionView>,
    pub known_draw:Vec<KnownDrawCardView>,
    pub encounter_facing:Option<EncounterFacingView>,
    pub inventory: Option<CombatInventoryView>,
    pub card_offer:Option<CombatCardOfferView>,
}
impl std::fmt::Debug for CombatPublic {
    fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result {
        let mut value=f.debug_struct("CombatPublic");
        value.field("played_powers",&self.played_powers);
        value.field("playing_cards",&self.playing_cards);
        value.field("stasis",&self.stasis);
        value.field("content_id",&self.content_id);
        value.field("turn",&self.turn);
        value.field("player",&self.player);
        value.field("enemies",&self.enemies);
        value.field("character",&self.character);
        value.field("hand",&self.hand);
        value.field("draw_count",&self.draw_count);
        value.field("discard",&self.discard);
        value.field("exhaust",&self.exhaust);
        value.field("outcome",&self.outcome);
        value.field("hp_before_healing",&self.hp_before_healing);
        value.field("hp_after_cleanup",&self.hp_after_cleanup);
        value.field("targets",&self.targets);
        value.field("selection",&self.selection);
        value.field("known_draw",&self.known_draw);
        value.field("encounter_facing",&self.encounter_facing);
        value.field("inventory",&self.inventory);
        if self.card_offer.is_some(){value.field("card_offer",&self.card_offer);}
        if self.mantra_gained.is_some(){value.field("mantra_gained",&self.mantra_gained);}
        if self.stance_choice.is_some(){value.field("stance_choice",&self.stance_choice);}
        value.finish()
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FullPowerView { pub id: super::FullPowerId, pub amount: i32 }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FullPlayerView {
    pub hp: i32, pub max_hp: i32, pub block: i32, pub energy: i32, pub max_energy: i32,
    pub powers: Vec<FullPowerView>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FullEnemyView {
    pub instance_id: EnemyHandle, pub enemy_id: EnemyId, pub hp: i32, pub max_hp: i32,
    pub block: i32, pub powers: Vec<FullPowerView>, pub intent: super::FullIntentId,
    pub intent_damage: Option<i32>, pub replaced: bool, pub escaped:bool, pub half_dead:bool, pub logical_slot:u8,
}
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct EncounterActorView {pub instance_id:EnemyHandle,pub side:super::EncounterSide,pub back_attack:bool}
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct EncounterFacingView {pub player_facing:super::PlayerFacing,pub surrounded:bool,pub enemies:Vec<EncounterActorView>}

/// Native combat-to-run handoff. It contains only actual full-run cards and
/// persistent combat result/resource values; run settlement remains a later owner.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CombatExit {
    pub played_powers:Vec<LiveCard>,
    pub stasis:Vec<LiveCard>,
    pub hp: i32, pub max_hp: i32, pub block: i32, pub energy: i32,
    pub outcome: Outcome, pub hand: Vec<LiveCard>, pub draw: Vec<LiveCard>,
    pub discard: Vec<LiveCard>, pub exhaust: Vec<LiveCard>, pub in_play: Option<LiveCard>,pub play_frames:Vec<LiveCard>,pub queued_autoplay:Vec<LiveCard>,
    pub inactive_cards:Vec<LiveCard>,
    pub(crate) item_context: Option<crate::items::CombatItemContext>,
}

/// Bounded public actor values and borrowed visible piles at a ready boundary.
/// No draw order, reveal history, private lifecycle flags or future selections.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicProjection<'a> {
    pub turn: u32, pub player: PlayerView, pub enemies: Vec<EnemyView>,
    pub draw_per_turn: i32, pub draw_count: usize,
    pub hand: &'a [CardInstance], pub discard: &'a [CardInstance], pub exhaust: &'a [CardInstance],
    pub outcome: Option<Outcome>,
    pub hp_before_healing: Option<i32>, pub hp_after_cleanup: Option<i32>,
}

impl Combat {
    /// Descriptive encounter-owned facing without changing observation-v1 or
    /// exposing actor history, selection state, or hidden resolver data.
    pub fn encounter_facing_view(&self)->Result<Option<EncounterFacingView>,AccessError>{
        ready(&self.state)?;
        encounter_facing(&self.state)
    }

    /// Read public values without copying visible piles or growing reveal history.
    /// Decision and terminal boundaries use the same access checks as `public_view`.
    pub fn public_projection(&self) -> Result<PublicProjection<'_>, AccessError> {
        if self.state.full_runtime { return Err(AccessError::NotReady); }
        let state = &self.state;
        ready(state)?;
        Ok(PublicProjection { turn: state.turn, player: player(state), enemies: enemies(state),
            draw_per_turn: state.player.draw_per_turn, draw_count: state.piles.draw.len(),
            hand: &self.legacy_cache.hand, discard: &self.legacy_cache.discard, exhaust: &self.legacy_cache.exhaust,
            outcome: state.outcome, hp_before_healing: state.hp_before_healing,
            hp_after_cleanup: state.hp_after_cleanup })
    }

    /// Explicit diagnostic access to a detached first-fault summary, including while faulted.
    pub fn fault_record(&self) -> Option<FaultRecord> {
        self.state.fault.clone()
    }
}

pub(super) fn encounter_facing(state:&State)->Result<Option<EncounterFacingView>,AccessError>{
    let Some(facing)=state.facing else{return Ok(None)};
    let surrounded=state.player_full_powers.iter().any(|p|p.id.0=="surrounded"&&p.state.amount>0);
    let mut enemies=Vec::new();
    for enemy in &state.enemies {
        if !matches!(enemy.enemy_id,super::ActorEnemyId::Full(crate::content::EnemyId::SpireShield|crate::content::EnemyId::SpireSpear)){continue}
        let slot=*state.actor_slots.get(&enemy.instance_id).ok_or(AccessError::NotReady)?;
        let side=match slot{0=>super::EncounterSide::Left,1=>super::EncounterSide::Right,_=>return Err(AccessError::NotReady)};
        enemies.push(EncounterActorView{instance_id:enemy.instance_id.clone(),side,back_attack:super::effects::back_attack_active(state,enemy)});
    }
    Ok(Some(EncounterFacingView{player_facing:facing,surrounded,enemies}))
}

#[derive(Clone, PartialEq, Eq)]
pub struct CharacterCombatView {
    pub stance: super::characters::Stance,
    pub orbs: Vec<Option<super::characters::OrbKind>>,
    pub orb_dark_damage:Vec<Option<i32>>,
    pub focus: i32,
}

pub(super) fn full_player(state: &State) -> Result<FullPlayerView,crate::run::RunError> {
    super::cards::validate_colorless_state(state).map_err(|_|crate::run::RunError::InvalidAction)?;
    let p = &state.player;
    let mut powers = p.powers.iter().map(|p| FullPowerView {
        id: super::FullPowerId(p.id.as_str()), amount: p.state.amount,
    }).chain(state.player_full_powers.iter().map(|p| FullPowerView { id: p.id, amount: p.state.amount })).collect::<Vec<_>>();
    if state.character.focus!=0 {powers.push(FullPowerView{id:FullPowerId("focus"),amount:state.character.focus});}
    powers.sort_by_key(|p| p.id.0);
    // These source-public counters are projections of the existing owners,
    // not extra physical powers or a second timer/callback implementation.
    if let Some(remaining)=state.colorless.panache_remaining {
        powers.push(FullPowerView{id:super::FullPowerId("panache_countdown"),amount:i32::from(remaining)});
    }
    // Timer row order is the existing callback/creation order, including
    // different captured payloads; no private cast IDs need to be exposed.
    for bomb in &state.colorless.bombs {
        let id=match bomb.damage{40=>"the_bomb",50=>"the_bomb_plus",_=>return Err(crate::run::RunError::InvalidAction)};
        powers.push(FullPowerView{id:super::FullPowerId(id),amount:i32::from(bomb.remaining)});
    }
    Ok(FullPlayerView { hp: p.hp, max_hp: p.max_hp, block: p.block, energy: p.energy,
        max_energy: p.max_energy, powers })
}

pub(super) fn full_enemies(state: &State) -> Vec<FullEnemyView> {
    state.enemies.iter().map(|e| FullEnemyView {
        instance_id: e.instance_id.clone(),
        enemy_id: match e.enemy_id { super::ActorEnemyId::Full(id) => id,
            super::ActorEnemyId::Legacy(id) => match id { EnemyKind::Cultist => EnemyId::Cultist, EnemyKind::JawWorm => EnemyId::JawWorm } },
        hp: e.hp, max_hp: e.max_hp, block: e.block,
        powers: e.powers.iter().map(|p| FullPowerView { id: super::FullPowerId(p.id.as_str()), amount: p.state.amount })
            .chain(e.full_powers.iter().map(|p| FullPowerView { id: p.id, amount: p.state.amount }))
            .collect(),
        intent: match e.intent_id { super::ActorIntentId::Full(id) => id,
            super::ActorIntentId::Legacy(id) => super::FullIntentId::from_name(id.as_str()) },
        intent_damage: e.intent_damage, replaced: e.life == super::ActorLife::Replaced,
        escaped:e.life==super::ActorLife::Escaped,half_dead:e.life==super::ActorLife::HalfDead,logical_slot:state.actor_slots.get(&e.instance_id).copied().unwrap_or(0),
    }).collect()
}

pub(super) fn ready(state: &State) -> Result<(), AccessError> {
    if state.fault.is_some() { return Err(AccessError::Faulted); }
    if state.pending_time_warp.is_some(){return Err(AccessError::NotReady)}
    if !matches!(state.phase, Phase::Decision | Phase::Terminal)
        || state.current.is_some() || !state.pending.is_empty()
        || (state.phase == Phase::Decision && state.in_play.is_some()) {
        return Err(AccessError::NotReady);
    }
    Ok(())
}

pub(super) fn legal_actions(state: &State) -> Result<Vec<Action>, AccessError> {
    ready(state)?;
    if state.phase == Phase::Terminal { return Ok(vec![]); }
    let mut actions = vec![];
    for owned in &state.piles.hand {
        let Some(card) = owned.legacy() else { continue; };
        if card_stats(card.card_id, card.upgraded).cost > state.player.energy { continue; }
        let target_id = match card.card_id {
            CardKind::Defend => None,
            CardKind::Strike | CardKind::Bash => {
                let Some(enemy) = state.enemies.iter().find(|e| e.hp > 0) else { continue; };
                Some(enemy.instance_id.clone())
            }
        };
        actions.push(Action::PlayCard { card_instance_id: card.instance_id.clone(), target_id });
    }
    actions.push(Action::EndTurn);
    Ok(actions)
}

fn powers(entries: &[PowerEntry]) -> Vec<(PowerId, i32)> {
    let mut values: Vec<_> = entries.iter().map(|p| (p.id, p.state.amount)).collect();
    values.sort_by_key(|(id, _)| id.as_str());
    values
}

fn cards(entries: &[OwnedCard]) -> Result<Vec<CardView>, AccessError> {
    entries.iter().map(|owned| owned.legacy().map(|c| CardView { instance_id: c.instance_id.clone(),
        card_id: c.card_id, upgraded: c.upgraded }).ok_or(AccessError::NotReady)).collect()
}

pub(super) fn player(state: &State) -> PlayerView {
    let p = &state.player;
    PlayerView { hp: p.hp, max_hp: p.max_hp, block: p.block, energy: p.energy,
        max_energy: p.max_energy, powers: powers(&p.powers) }
}

pub(super) fn enemies(state: &State) -> Vec<EnemyView> {
    state.enemies.iter().map(|e| EnemyView { instance_id: e.instance_id.clone(),
        enemy_id: e.enemy_id.legacy().expect("legacy public view only receives legacy actors"), hp: e.hp, max_hp: e.max_hp, block: e.block,
        powers: powers(&e.powers), intent_id: e.intent_id.legacy().expect("legacy public view only receives legacy intents"),
        intent_damage: e.intent_damage }).collect()
}

pub(super) fn public_view(state: &State) -> Result<PublicView, AccessError> {
    ready(state)?;
    Ok(PublicView { content_id: state.content_id.clone(), turn: state.turn,
        player: player(state), enemies: enemies(state),
         hand: cards(&state.piles.hand)?, draw_count: state.piles.draw.len(),
         discard: cards(&state.piles.discard)?, exhaust: cards(&state.piles.exhaust)?,
        revealed_order: state.public.revealed_order.clone(), outcome: state.outcome,
        hp_before_healing: state.hp_before_healing, hp_after_cleanup: state.hp_after_cleanup })
}

impl std::fmt::Debug for LiveCardView {
    fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result {
        let mut d=f.debug_struct("LiveCardView");d.field("instance",&self.instance).field("id",&self.id).field("origin",&self.origin).field("upgrades",&self.upgrades);
        if self.misc!=0{d.field("misc",&self.misc);}
        d.field("effective_cost",&self.effective_cost).field("values",&self.values);if self.watcher.is_some(){d.field("watcher",&self.watcher);}d.finish()
    }
}
impl std::fmt::Debug for CharacterCombatView {
    fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result {
        let mut d=f.debug_struct("CharacterCombatView");d.field("stance",&self.stance).field("orbs",&self.orbs);
        if self.orb_dark_damage.iter().any(Option::is_some){d.field("orb_dark_damage",&self.orb_dark_damage);}
        d.field("focus",&self.focus).finish()
    }
}
