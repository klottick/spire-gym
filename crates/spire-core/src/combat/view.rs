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
    pub origin: CombatCardRef, pub upgrades: u32, pub misc:i32, pub watcher:Option<WatcherCardValues>, pub effective_cost:Option<i32>,pub values:Option<CardPublicValues>,
    pub lifecycle:CardLifecycleView }
/// M34s-a (a0(iv) follow-up): the card's cost and free-play lifecycle, each a fact the original player sees or derives
/// from public history and each read by later gameplay: `free_play` an autoplay waiver still held (shown as cost 0),
/// `free_to_play_once` a one-use waiver (Forethought/Setup; shown as cost 0), `randomized_cost` a rolled cost
/// (Confusion/Snecko Oil; shown when drawn), `mutable_combat_cost` the modified combat cost, `combat_cost_reduction`
/// Blood for Blood's HP-loss discount, `infernal_cost_this_turn` Infernal Blade's cost-0 that expires at end of turn.
/// Not part of the captured Debug (unchanged); part of equality and the canonical card order.
#[derive(Clone,Copy,Debug,Default,PartialEq,Eq,PartialOrd,Ord)]
pub struct CardLifecycleView{pub free_play:bool,pub free_to_play_once:bool,pub randomized_cost:Option<i32>,
    pub mutable_combat_cost:Option<i32>,pub combat_cost_reduction:i32,pub infernal_cost_this_turn:bool}
pub(super) fn live_card_view(state:&State,live:&super::LiveCard)->LiveCardView{
    let l=&live.local;
    LiveCardView{misc:live.misc,instance:live.instance,id:live.id,origin:live.origin.clone(),upgrades:live.upgrades,
        effective_cost:super::cards::visible_cost(state,live),values:super::cards::public_values(live),
        watcher:super::cards::watcher::card_values(state,live),
        lifecycle:CardLifecycleView{free_play:l.free_play,free_to_play_once:l.free_to_play_once,randomized_cost:l.randomized_cost,
            mutable_combat_cost:l.mutable_combat_cost,combat_cost_reduction:l.combat_cost_reduction,
            infernal_cost_this_turn:l.infernal_blade_cost_this_turn}}
}
/// M34s-a canonical card key (design §4.3, §4.5.1 card class): the published card values, then the origin
/// (a permanent card's deck instance; every combat-local card alike), without the instance and the CombatLocal id.
/// Cards with equal classes are interchangeable in the public draw knowledge (`knowledge::DrawKnowledge`).
#[derive(Clone,Copy,Debug,PartialEq,Eq,PartialOrd,Ord)]
pub struct CardClass(crate::content::CardId,u32,Option<i32>,Option<CardValuesKey>,i32,Option<(Option<i32>,Option<i32>,bool,bool)>,CardLifecycleView,(u8,u64));
type CardValuesKey=(Option<i32>,Option<i32>,Option<i32>,Option<i32>,u8,bool,bool,bool,bool,i32);
pub fn card_class(v:&LiveCardView)->CardClass{
    let values=v.values.as_ref().map(|p|(p.base_damage,p.base_block,p.magic_number,p.damage_bonus,p.cost_kind as u8,p.free_play,p.exhaust_on_finish,p.ethereal,p.innate,p.temporary_cost));
    let watcher=v.watcher.as_ref().map(|w|(w.combat_cost,w.turn_cost,w.self_retain,w.retain));
    let origin=match v.origin{CombatCardRef::Permanent(id)=>(0,id.0),CombatCardRef::CombatLocal(_)=>(1,0)};
    CardClass(v.id,v.upgrades,v.effective_cost,values,v.misc,watcher,v.lifecycle,origin)
}
impl CardClass{
    /// The card kind of this class.
    pub fn id(&self)->crate::content::CardId{self.0}
}
/// M34s-a canonical card order: the card class (published values, then origin), the instance only as the final
/// tie-break between cards that are equal in every published value (design §4.3).
pub fn canonical_card_order(a:&LiveCardView,b:&LiveCardView)->std::cmp::Ordering{
    card_class(a).cmp(&card_class(b)).then(a.instance.0.cmp(&b.instance.0))
}
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
    /// M25 Watcher: type of the previous card played this combat (FollowUp, CrushJoints, SashWhip, Sanctity).
    pub previous_card_type:Option<crate::content::CardType>,
    pub stance_choice:Option<StanceChoiceView>,
    pub played_powers:Vec<LiveCardView>,
    /// Cards whose accepted use has started, in suspended-parent to active-child order.
    pub playing_cards:Vec<LiveCardView>,
    pub stasis:Vec<StasisCardView>,
    pub content_id: String, pub turn: u32, pub player: FullPlayerView, pub enemies: Vec<FullEnemyView>,
    pub character: CharacterCombatView,
    pub hand: Vec<LiveCardView>, pub draw_count: usize, pub discard: Vec<LiveCardView>,
    pub exhaust: Vec<LiveCardView>, pub outcome: Option<Outcome>,
    /// Current combat decision phase, including a paused modal selection.
    pub phase: super::Phase,
    /// The phase the open choice interrupted, recorded when it opened. Every commit
    /// path (modal or not, and stance choices and card offers) resumes exactly it.
    pub selection_resume_phase: Option<super::Phase>,
    pub hp_before_healing: Option<i32>, pub hp_after_cleanup: Option<i32>,
    pub targets: Vec<(TargetId, bool)>,
    pub selection: Option<SelectionView>,
    pub known_draw:Vec<KnownDrawCardView>,
    /// M34s-a: every draw-pile card not in `known_draw`, as a multiset in canonical card order
    /// (`canonical_card_order`); never physical order. The original draw-pile screen shows it sorted. Populated only in a
    /// combat that maintains public knowledge (`Combat::maintains_public_knowledge`: Standard, `standard-combat-v1`), the
    /// profiles that publish draw rows; empty elsewhere (final review M7).
    pub draw_unknown:Vec<LiveCardView>,
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
        if self.phase==super::Phase::Selection {
            value.field("phase",&self.phase);
            value.field("selection_resume_phase",&self.selection_resume_phase);
        }
        value.field("hp_before_healing",&self.hp_before_healing);
        value.field("hp_after_cleanup",&self.hp_after_cleanup);
        value.field("targets",&self.targets);
        value.field("selection",&self.selection);
        value.field("known_draw",&self.known_draw);
        value.field("draw_unknown",&self.draw_unknown);
        value.field("encounter_facing",&self.encounter_facing);
        value.field("inventory",&self.inventory);
        if self.card_offer.is_some(){value.field("card_offer",&self.card_offer);}
        if self.mantra_gained.is_some(){value.field("mantra_gained",&self.mantra_gained);}
        if self.previous_card_type.is_some(){value.field("previous_card_type",&self.previous_card_type);}
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
    /// M34a: source intent category and planned damage applications; both `None` while intents are hidden.
    pub intent_category: Option<super::intents::IntentCategory>, pub intent_hits: Option<u32>,
    /// M34a: the live Invincible cap (Corrupt Heart only).
    pub invincible_cap: Option<i32>,
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
    }).chain(state.player_full_powers.iter().filter(|p| !super::powers::publicly_hidden(p.id))
        .map(|p| FullPowerView { id: p.id, amount: p.state.amount })).collect::<Vec<_>>();
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
    debug_assert_registered(&powers, None);
    Ok(FullPlayerView { hp: p.hp, max_hp: p.max_hp, block: p.block, energy: p.energy,
        max_energy: p.max_energy, powers })
}

pub(super) fn public_rules(state:&State)->crate::items::laws::PublicViewRules{
    state.item_context.as_ref().map_or_else(crate::items::laws::PublicViewRules::default,|context|{
        // M27 seam: public-view
        crate::items::laws::public_view(context)
    })
}

/// Debug builds reject a published power or intent missing from the engine
/// registry that every observation encoder vocabulary is derived from.
fn debug_assert_registered(powers: &[FullPowerView], intent: Option<super::FullIntentId>) {
    for power in powers {
        debug_assert!(super::public_ids::is_public_power(power.id.0), "unregistered public power {:?}", power.id.0);
    }
    if let Some(intent) = intent {
        debug_assert!(super::public_ids::is_public_intent(intent.as_str()), "unregistered public intent {:?}", intent.as_str());
    }
}

pub(super) fn full_enemies(state: &State,hide_intents:bool) -> Vec<FullEnemyView> {
    let enemies: Vec<FullEnemyView> = state.enemies.iter().map(|e| FullEnemyView {
        instance_id: e.instance_id.clone(),
        enemy_id: match e.enemy_id { super::ActorEnemyId::Full(id) => id,
            super::ActorEnemyId::Legacy(id) => match id { EnemyKind::Cultist => EnemyId::Cultist, EnemyKind::JawWorm => EnemyId::JawWorm } },
        hp: e.hp, max_hp: e.max_hp, block: e.block,
        powers: e.powers.iter().map(|p| FullPowerView { id: super::FullPowerId(p.id.as_str()), amount: p.state.amount })
            .chain(e.full_powers.iter().map(|p| FullPowerView { id: p.id, amount: p.state.amount }))
            .collect(),
        intent: if hide_intents { super::FullIntentId::Unknown("hidden") } else {
            match e.intent_id { super::ActorIntentId::Full(id) => id,
                super::ActorIntentId::Legacy(id) => super::FullIntentId::from_name(id.as_str()) }
        },
        intent_damage: if hide_intents{None}else{e.intent_damage.map(|d|super::effects::displayed_intent_damage(state,e,d))}, replaced: e.life == super::ActorLife::Replaced,
        escaped:e.life==super::ActorLife::Escaped,half_dead:e.life==super::ActorLife::HalfDead,logical_slot:state.actor_slots.get(&e.instance_id).copied().unwrap_or(0),
        intent_category: if hide_intents{None}else{super::intents::category(super::intents::enemy_id(e),e.intent_id.as_str(),state.ascension)},
        intent_hits: if hide_intents{None}else{super::intents::planned_hits(e,state.ascension)},
        invincible_cap: super::intents::invincible_cap(e,state.ascension),
    }).collect();
    for enemy in &enemies { debug_assert_registered(&enemy.powers, Some(enemy.intent)); }
    enemies
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

#[cfg(all(test, debug_assertions))]
mod registry_guard_tests {
    use super::{debug_assert_registered, FullPowerView};
    use crate::combat::{FullIntentId, FullPowerId};

    #[test]
    fn registered_power_and_intent_pass_the_view_guard() {
        debug_assert_registered(&[FullPowerView { id: FullPowerId("strength"), amount: 1 }], Some(FullIntentId::Unknown("hidden")));
    }

    #[test]
    #[should_panic(expected = "unregistered public power")]
    fn unregistered_power_fails_loudly_at_the_view_guard() {
        debug_assert_registered(&[FullPowerView { id: FullPowerId("not_a_registered_power"), amount: 1 }], None);
    }

    #[test]
    #[should_panic(expected = "unregistered public intent")]
    fn unregistered_intent_fails_loudly_at_the_view_guard() {
        debug_assert_registered(&[], Some(FullIntentId::Unknown("not_a_registered_intent")));
    }
}
