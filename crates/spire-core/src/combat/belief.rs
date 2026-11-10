//! M34s-b public belief `spire-gym-public-belief-v1` (design §4.6): the canonical public root of a combat and the
//! research-seeded re-sampling of its hidden state from public knowledge only.
//!
//! A root is the combat-scope transplant (`Run::combat_scope`) canonicalised here: the unknown draw cards in canonical
//! order, every RNG key and cursor zeroed, every ambiguous pre-roll at a fixed placeholder. A sample re-draws exactly that
//! hidden state from the research key: R1 the draw order from the exact draw knowledge (§4.5.1), R2 every combat stream,
//! R3 each ambiguous pre-roll uniformly on its consistent set (§4.5.2). Everything else is kept. Nothing here reads the
//! true draw order, a true pre-roll or a true key: canonicalisation overwrites them before sampling starts.
use super::knowledge::{self, DrawKnowledge, Segment, Unrepresentable};
use super::model::State;
use super::rng::hmac_sha256;
use super::view::CardClass;
use super::*;

pub const PUBLIC_BELIEF_ID: &str = "spire-gym-public-belief-v1";

/// Test-only leaky canonicalisers (design §4.10 negative control (b)): each must make a privacy test fail. Outside
/// tests no leak is ever active.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Leak { KeepStreams, KeepDrawOrder, KeepPreroll, AllowAll }
#[cfg(test)]
thread_local! { pub(crate) static LEAK: std::cell::Cell<Option<Leak>> = const { std::cell::Cell::new(None) }; }
#[cfg(test)]
fn leak(l: Leak) -> bool { LEAK.with(|v| v.get()) == Some(l) }
#[cfg(not(test))]
fn leak(_: Leak) -> bool { false }

/// Why a combat is not a supported public root (design §4.6.1). Each names a class of reason, never a value (P7).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unsupported {
    /// The combat does not maintain public knowledge (captured profiles, Gym66).
    NoPublicKnowledge,
    /// The combat has an outcome.
    Terminal,
    /// The combat holds a fault.
    Faulted,
    /// The combat is not at a published decision or selection boundary.
    NotAtDecision,
    /// Hidden intents (Runic Dome, L2).
    HiddenIntents,
    /// A queued or held continuation outside the allow-list; the static label names its kind.
    DisallowedContinuation(&'static str),
    /// The draw knowledge exceeds the exact model (L1).
    UnrepresentableKnowledge,
    /// A hidden pre-rolled value without a consistent set under its law (L5).
    UnrepresentablePreroll,
}
impl Unsupported {
    pub fn reason(self) -> &'static str {
        match self {
            Self::NoPublicKnowledge => "the combat does not maintain public knowledge",
            Self::Terminal => "the combat has ended",
            Self::Faulted => "the combat is faulted",
            Self::NotAtDecision => "the combat is not at a published decision",
            Self::HiddenIntents => "hidden intents are not modelled",
            Self::DisallowedContinuation(kind) => kind,
            Self::UnrepresentableKnowledge => "draw knowledge exceeds the exact model",
            Self::UnrepresentablePreroll => "a pre-rolled value has no consistent set",
        }
    }
}

/// The randomness a sample consumes: uniform draws below a bound, nothing else (so tests can enumerate it exactly).
pub(crate) trait Uniform { fn below(&mut self, bound: u128) -> u128; }

/// A research stream: HMAC-SHA256(key, counter) words, rejection-sampled to an exactly uniform value.
pub(crate) struct ResearchStream { key: [u8; 32], counter: u64 }
impl ResearchStream {
    pub(crate) fn new(key: [u8; 32]) -> Self { Self { key, counter: 0 } }
}
impl Uniform for ResearchStream {
    fn below(&mut self, bound: u128) -> u128 {
        assert!(bound > 0, "uniform bound must be positive");
        // Largest multiple of `bound` that fits, minus one: accept words at or below it.
        let limit = u128::MAX - (u128::MAX % bound + 1) % bound;
        loop {
            let word = hmac_sha256(&self.key, &self.counter.to_be_bytes());
            self.counter += 1;
            let value = u128::from_be_bytes(word[..16].try_into().unwrap());
            if value <= limit { return value % bound }
        }
    }
}

/// R1: a draw-pile class sequence, top to bottom, from the exact knowledge: a component with probability ∝ its weight,
/// then per segment its known class, or for a bag its first card uniform over the copies not top-excluded and the
/// remaining copies in uniformly random order.
pub(crate) fn sample_draw_classes(knowledge: &DrawKnowledge, rng: &mut impl Uniform) -> Vec<CardClass> {
    let mut pick = rng.below(knowledge.total_weight());
    let component = knowledge.components().iter().find(|c| if pick < c.weight { true } else { pick -= c.weight; false })
        .expect("weights sum to the total");
    let mut out = vec![];
    for segment in &component.segments {
        match segment {
            Segment::Known(c) => out.push(*knowledge.class(*c)),
            Segment::Bag(entries) => {
                let mut left = entries.iter().map(|e| (e.class, u128::from(e.count), u128::from(e.count - e.excluded))).collect::<Vec<_>>();
                let mut first = true;
                while left.iter().any(|e| e.1 > 0) {
                    let weight = |e: &(u16, u128, u128)| if first { e.2 } else { e.1 };
                    let mut r = rng.below(left.iter().map(weight).sum());
                    let at = left.iter().position(|e| if r < weight(e) { true } else { r -= weight(e); false }).unwrap();
                    out.push(*knowledge.class(left[at].0));
                    left[at].1 -= 1;
                    first = false;
                }
            }
        }
    }
    out
}

/// The fail-closed continuation allow-list (design §4.6.1, census b0 lane 2): every queued or held resolver item must hold
/// no already-sampled hidden value. One exhaustive match per kind, with no wildcard arm: a new `EffectKind` variant, or a
/// new variant of a nested kind with a denied member, does not compile until classified (P6). Denied: a randomly chosen
/// autoplay target held across a suspended child play (Havoc, Mayhem, Distilled Chaos), captured but unoffered generated
/// cards, an Alchemize proposal, and spawn or replacement work in progress.
fn allow_list(state: &State) -> Result<(), Unsupported> {
    for effect in state.pending.iter().chain(state.current.iter()) { allow_effect(&effect.kind)?; }
    // A nested child play holding an enemy target: the autoplay paths draw that target at random (Havoc, Mayhem, Distilled
    // Chaos); a replayed player target (Double Tap, Duplication) is denied too (fail closed; disclosed).
    if state.play_frames.iter().any(|f| matches!(f.target, Some(Actor::Enemy(_)))) {
        return Err(Unsupported::DisallowedContinuation("a nested card play holds an enemy target"))
    }
    if state.spawn_progress.is_some() || state.replacement_progress.is_some() {
        return Err(Unsupported::DisallowedContinuation("enemy spawn or replacement in progress"))
    }
    // A thief's rolled post-Mug branch awaiting its move roll (census: consumed before any boundary; fail closed).
    if state.enemies.iter().any(|e| matches!(&e.full_memory, Some(enemies::EnemyMemory::Thief(m)) if m.pending_next.is_some())) {
        return Err(Unsupported::DisallowedContinuation("a rolled enemy branch awaits its move"))
    }
    Ok(())
}
fn allow_effect(kind: &EffectKind) -> Result<(), Unsupported> {
    use EffectKind::*;
    let deny = |what| Err(Unsupported::DisallowedContinuation(what));
    let allow_watcher = |work: &super::cards::watcher::WatcherEffect| {
        use super::cards::watcher::WatcherEffect as W;
        match work {
            W::OpenStanceChoice | W::PotionEnergy{..} | W::CreateRequest{..} | W::Create{..} | W::CreateShuffled{..}
            | W::WishOption{..} | W::Meditate{..} | W::RequestEndTurn | W::Scry{..} | W::ScryFromRelic{..}
            | W::CommitSelection{..} | W::RetainRestore{..} | W::ReduceCost{..} | W::EstablishmentDiscount{..}
            | W::GainMantra{..} | W::StanceEnergy{..} | W::ReturnDiscard{..} | W::PressurePoints | W::GrantExtraTurn
            | W::ExhaustDrawnDeus{..} | W::OpenOmniscience | W::OpenForeignInfluence | W::JudgmentKill{..}
            | W::OmnisciencePlay{..} | W::OmniscienceRepeat{..} => Ok(()),
        }
    };
    let allow_defect = |work: &super::cards::DefectEffect| {
        use super::cards::{DefectEffect as D, defect::{DirectEffect as R, StateEffect as S}};
        match work {
            // Retrieve's card is All For One's public discard pick; Seek's grid is the shared order-hiding grid (I1).
            D::State(state) => match state {
                S::Ftl{..} | S::X{..} | S::Hologram | S::Seek{..} | S::Recycle | S::AllForOne | S::Retrieve{..} | S::Scrape
                | S::Reboot => Ok(()),
            },
            D::Direct(direct) => match direct {
                R::Chaos{orbs, ..} if !orbs.is_empty() => deny("Defect Chaos holds sampled orb rolls"),
                R::Aggregate{..} | R::DoubleEnergy | R::RandomHit{..} | R::Chaos{..} | R::Melter{..}
                | R::Sunder{..} | R::SunderCheck => Ok(()),
            },
            D::EssenceOfDarkness{..} | D::GeneticGrowth{..} | D::ClawGrowth | D::SteamWear | D::StreamlineCost
            | D::Repeat{..} => Ok(()),
        }
    };
    let allow_thief = |work: &super::enemies::act1::thieves::ThiefEffect| {
        use super::enemies::act1::thieves::ThiefEffect as T;
        match work { T::Ledger{..} | T::TheftDamage{..} | T::Escape{..} | T::DeathReward{..} => Ok(()) }
    };
    let allow_hook = |hook: &super::hooks::Hook| {
        use super::hooks::Hook as H;
        match hook {
            H::CardAcquired | H::CardRemoved | H::RoomEntered | H::CombatStarted | H::TurnStartedPreDraw | H::TurnStarted
            | H::TurnEnded | H::PlayerPreEndTurnCards | H::PlayerEndTurn | H::EnemyTurnStarted | H::CardPlayed
            | H::CardAfterUse | H::CardDiscarded | H::CardManuallyDiscarded | H::CardExhausted | H::CardDrawn
            | H::Damage | H::Death | H::CombatSettled | H::CombatStartedPreDraw | H::ChestOpened{..}
            | H::ChestOpenedAfter{..} | H::Rested | H::HalfDeath | H::CardAfterPlayed | H::AttackResolved
            | H::DrawPileShuffled => Ok(()),
        }
    };
    match kind {
        OpeningHand{..} | ShelledArmorBreak | GainBlock(_) | Heal(_) | IncreaseMaxHpThenHeal(_) | NeowsLament
        | ApplyPower{..} | ReducePower{..} | RemovePower(_) | ApplyFullPower{..} | ReduceFullPower{..} | RemoveFullPower(_)
        | FinishCard | Decision | JuggernautRandomDamage(_) | Draw(_) | Recycle | DiscardHand | DrawOne
        | AfterCardDrawn{..} | GainEnergy(_) | LoseEnergy(_) | ManualDiscardCallback{..} | CreateCard{..} | CopyCard{..}
        | ExhaustHandCard{..} | ExhaustBatchStart{..} | ExhaustBatchContinue(_) | SelectCard{..} | SelectCards{..}
        | SelectHandCardsAfterDraw{..} | SelectNightmareCard | NightmareCopy{..} | OpenRetainSelection{..}
        | EndPlayerTurn | MonsterStart | EnemyMove | RollMove | EndRound | StartNextPlayer | ShieldBashFocus
        | ReplaceEnemy{..} | GroundByrd | SpawnEnemy(_) | CompleteSpawn | RallyGremlins{..} | ProtectGremlin
        | SetEnemyMove{..} | EscapeEnemy | BookStab{..} | ChampCleanse | CollectorSuicide | AutomatonSuicide
        | ShapeSuicide | ShapeDamageComplete | PoisonTickComplete | RevivalHeal(_) | DarklingCollapse | DarklingRevive
        | DarklingsDefeat | TransientFade | MawNom{..} | ObtainParasite | ResetMalleable | ReptomancerSuicide
        | AwakenedCollapse | AwakenedRebirth | AwakenedCanLose | TimeWarpEndTurn | TimeEaterCleanse
        | CaptureStasis{..} | ReleaseStasis{..} | CorruptHeartBuff | Catalyst | ShuffleDraw | CardOffer{..}
        | SmokeEscape | DeliverCardOffer{..} | ObtainPotion{..} => Ok(()),
        Watcher(work) => allow_watcher(work),
        Defect(work) => allow_defect(work),
        Thief(work) => allow_thief(work),
        Damage(payload) => match payload {
            DamagePayload::Fixed(_) | DamagePayload::Orb(_) | DamagePayload::Thorns(_) | DamagePayload::HpLoss(_)
            | DamagePayload::PoisonTickHpLoss{..} | DamagePayload::EnemyMove(_) | DamagePayload::FullEnemyMove(_)
            | DamagePayload::CorpseExplosion{..} => Ok(()),
        },
        SilentItem(work) => match work { SilentItemEffect::SpecimenPoisonTransfer{..} => Ok(()) },
        Terminal(outcome) => match outcome { Outcome::Victory | Outcome::Death | Outcome::Escaped => Ok(()) },
        Hook(hook) => allow_hook(hook),
        Character(work) => match work {
            CharacterEffect::Stance(stance) => match stance {
                characters::Stance::Neutral | characters::Stance::Wrath | characters::Stance::Calm | characters::Stance::Divinity => Ok(()),
            },
            CharacterEffect::Channel(kind) => match kind {
                characters::OrbKind::Lightning | characters::OrbKind::Frost | characters::OrbKind::Dark | characters::OrbKind::Plasma => Ok(()),
            },
            CharacterEffect::PassiveGroup(group) => match group {
                characters::OrbPassiveGroup::NaturalStart | characters::OrbPassiveGroup::NaturalEnd
                | characters::OrbPassiveGroup::All | characters::OrbPassiveGroup::Dark => Ok(()),
            },
            CharacterEffect::Focus(_) | CharacterEffect::EvokeFront{..} | CharacterEffect::EvokeAt{..}
            | CharacterEffect::EndTurn | CharacterEffect::StartTurn | CharacterEffect::Slots(_)
            | CharacterEffect::ChannelState{..} | CharacterEffect::Recursion | CharacterEffect::Fission{..}
            | CharacterEffect::Lightning{..} | CharacterEffect::OrbEnergy(_) => Ok(()),
        },
        GuardianChange(work) => match work { super::enemies::GuardianEffect::Defensive | super::enemies::GuardianEffect::Offensive
            | super::enemies::GuardianEffect::Reset | super::enemies::GuardianEffect::LoseBlock => Ok(()) },
        HexaghostBurn(work) => match work { super::enemies::HexaghostEffect::UpgradeBurns | super::enemies::HexaghostEffect::InsertBurns => Ok(()) },
        Silent(work) => match work {
            super::cards::SilentEffect::FlaskBounce{..} => deny("a Flask of Poison bounce holds a randomly chosen target"),
            super::cards::SilentEffect::DrawContinue(_) | super::cards::SilentEffect::BaneHit{..}
            | super::cards::SilentEffect::FlaskStart{..} | super::cards::SilentEffect::ToolsDiscard{..}
            | super::cards::SilentEffect::Setup | super::cards::SilentEffect::BulletTime | super::cards::SilentEffect::GlassKnifeAfterHits
            | super::cards::SilentEffect::EndlessAgonyCopy{..} | super::cards::SilentEffect::Discard{..}
            | super::cards::SilentEffect::DiscardSpecific{..} | super::cards::SilentEffect::Gamble | super::cards::SilentEffect::Unload
            | super::cards::SilentEffect::Expertise{..} | super::cards::SilentEffect::EscapePlanBlock{..}
            | super::cards::SilentEffect::AttackCountHits{..} | super::cards::SilentEffect::SkillCountHits{..}
            | super::cards::SilentEffect::SneakyEnergy | super::cards::SilentEffect::BurstRepeat{..}
            | super::cards::SilentEffect::StormOfSteel{..} => Ok(()),
        },
        Colorless(work) => match work {
            super::cards::ColorlessEffect::MayhemPlayTopCard{..} | super::cards::ColorlessEffect::MayhemAutoplay{..} =>
                deny("a queued Mayhem play holds a random target"),
            super::cards::ColorlessEffect::ImpatienceResolve{..} | super::cards::ColorlessEffect::MayhemQueueTopCard
            | super::cards::ColorlessEffect::BombInstall{..} | super::cards::ColorlessEffect::BombTick{..}
            | super::cards::ColorlessEffect::PanacheDamage{..} | super::cards::ColorlessEffect::ApotheosisUpgrade
            | super::cards::ColorlessEffect::EnlightenmentResolve | super::cards::ColorlessEffect::ForethoughtSelect
            | super::cards::ColorlessEffect::MadnessResolve | super::cards::ColorlessEffect::PuritySelect
            | super::cards::ColorlessEffect::SecretRetrieve{..} | super::cards::ColorlessEffect::ThinkingAheadSelect
            | super::cards::ColorlessEffect::ViolenceMove{..} => Ok(()),
        },
        Potion(work) => match work {
            PotionEffect::DistilledChaosTopCard{..} | PotionEffect::DistilledChaosQueuedChild{..} => deny("a queued Distilled Chaos play holds a random target"),
            PotionEffect::UpgradeHand | PotionEffect::RandomizeHandCosts | PotionEffect::GamblersBrewSelectDiscard
            | PotionEffect::LiquidMemories{..} | PotionEffect::LiquidMemoriesRetrieve{..} | PotionEffect::DuplicationRepeat{..}
            | PotionEffect::EntropicBrew{..} => Ok(()),
        },
        Relic(work) => match work {
            RelicEffect::UpgradeRandomHandCard | RelicEffect::CapEnemyHp{..} | RelicEffect::SetRandomHandCardCostZeroThisTurn
            | RelicEffect::RemovePlayerDebuffs | RelicEffect::GamblingChipSelect | RelicEffect::RetainHand | RelicEffect::RedSkullCheck => Ok(()),
        },
        Ironclad(work) => {
            use super::cards::IroncladEffect as I;
            match work {
                I::HavocTop { .. } | I::HavocExtract { .. } | I::HavocQueued { .. } => deny("a queued Havoc play holds a random target"),
                I::UpgradeHand { .. } | I::UpgradeCard { .. } | I::SwordBoomerangHit { .. } | I::DoubleCurrentBlock | I::SpotWeakness { .. }
                | I::TrueGritRandomExhaust | I::ChooseTrueGritCard | I::ChooseHeadbuttCard | I::ChooseExhumeCard | I::ChooseDualWieldCard
                | I::WhirlwindRound { .. } | I::WhirlwindSpend | I::DoubleTapRepeat { .. } | I::FinishSuppressed | I::LimitBreakDispatch
                | I::DropkickDispatch { .. } | I::RampageGrow { .. } | I::ReaperResolve { .. } | I::InfernalBladeGenerate => Ok(()),
            }
        }
        CardContinuation(work) => match work {
            super::cards::DefectContinuation::GoForTheEyesWeak{..} | super::cards::DefectContinuation::BarrageStart{..}
            | super::cards::DefectContinuation::BarrageNext{..} | super::cards::DefectContinuation::CompileDriverDraw => Ok(()),
        },
        Alchemize { proposal } => if proposal.is_some() { deny("an Alchemize proposal is held") } else { Ok(()) },
        Generate(work) => if !work.cards.is_empty() && work.offer.is_none() { deny("generated cards are held unoffered") } else { Ok(()) },
    }
}

/// The public combat identity streams derive from (design §4.6.4): the Run's act, the encounter and visit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct StreamIdentity<'a> { pub act: u8, pub encounter: &'a str, pub visit: u64 }

/// Replace every combat stream (shuffle, card random, each actor's AI stream and the entry copy) by the gameplay
/// derivation from `rng` at the public identity, cursors at zero. With the zero Run root this is canonicalisation; with a
/// research root it is R2. Child streams for splits and spawns derive from these at use.
fn restream(state: &mut State, rng: &crate::run::RunRng, id: StreamIdentity) -> Result<(), Unsupported> {
    use super::rng::Domain;
    let derive = |domain, slot: Option<u32>, role: &str| rng.derive_combat_stream(domain, id.act, id.encounter, id.visit, slot, role)
        .map_err(|_| Unsupported::DisallowedContinuation("stream identity outside the gameplay derivation"));
    state.rng.shuffle = derive(Domain::RunCombatShuffle, None, "shuffle")?;
    if state.card_rng.is_some() { state.card_rng = Some(derive(Domain::RunCombatCardRandom, None, "card")?); }
    let mut entry_copy = None;
    for enemy in &mut state.enemies {
        let Some(slot) = state.actor_slots.get(&enemy.instance_id).copied() else {
            if enemy.full_ai_rng.is_some() { return Err(Unsupported::DisallowedContinuation("actor stream without a slot")) }
            continue
        };
        let stream = derive(Domain::RunCombatEnemyAi, Some(u32::from(slot)), enemy.instance_id.as_str())?;
        if enemy.instance_id == state.rng.enemy_instance_id { entry_copy = Some(stream.clone()); }
        if enemy.full_ai_rng.is_some() { enemy.full_ai_rng = Some(stream); }
    }
    state.rng.enemy_ai = entry_copy.ok_or(Unsupported::DisallowedContinuation("entry stream owner missing"))?;
    Ok(())
}

/// The handles of the draw pile rearranged so its classes read `top_to_bottom`: published known cards stay at their
/// positions; per class, the remaining handles of that class in canonical (handle) order fill that class's remaining
/// positions top to bottom (design §4.6.2 R1). `None` if the sequence is not an arrangement of the pile's classes.
fn arrange_draw(state: &mut State, top_to_bottom: &[CardClass]) -> Option<()> {
    let len = state.piles.draw.len();
    if top_to_bottom.len() != len { return None }
    let known = state.public.known_draw.iter().enumerate().map(|(i, h)| (i, h.clone()))
        .chain(state.public.known_draw_bottom.iter().enumerate().map(|(i, h)| (len.checked_sub(1 + i).unwrap_or(usize::MAX), h.clone())))
        .collect::<Vec<_>>();
    let mut cards = std::mem::take(&mut state.piles.draw);
    let mut slots: Vec<Option<model::OwnedCard>> = vec![None; len];
    for (depth, handle) in &known {
        let at = cards.iter().position(|c| c.handle() == *handle)?;
        if *depth >= len || slots[*depth].is_some() { state.piles.draw = cards; return None }
        slots[*depth] = Some(cards.remove(at));
    }
    // Remaining cards by (class, handle): the canonical order the root keeps and the sample fills from.
    let mut rest = cards.into_iter().map(|c| (knowledge::class_of(state, &c), c)).collect::<Vec<_>>();
    if rest.iter().any(|(class, _)| class.is_none()) { return None }
    rest.sort_by(|a, b| (a.0, a.1.handle()).cmp(&(b.0, b.1.handle())));
    for depth in 0..len {
        if slots[depth].is_some() { continue }
        let at = rest.iter().position(|(class, _)| *class == Some(top_to_bottom[depth]))?;
        slots[depth] = Some(rest.remove(at).1);
    }
    let mut ordered = slots.into_iter().collect::<Option<Vec<_>>>()?;
    ordered.reverse(); // physical order is bottom to top
    state.piles.draw = ordered;
    Some(())
}

/// The canonical draw sequence: known cards at their positions, every other card by (class, handle) top to bottom.
fn canonical_draw_classes(state: &State) -> Option<Vec<CardClass>> {
    let len = state.piles.draw.len();
    let class = |h: &CardHandle| state.piles.draw.iter().find(|c| c.handle() == *h).and_then(|c| knowledge::class_of(state, c));
    let mut out: Vec<Option<CardClass>> = vec![None; len];
    for (i, h) in state.public.known_draw.iter().enumerate() { *out.get_mut(i)? = Some(class(h)?); }
    for (i, h) in state.public.known_draw_bottom.iter().enumerate() { *out.get_mut(len.checked_sub(1 + i)?)? = Some(class(h)?); }
    let known = state.public.known_draw.iter().chain(&state.public.known_draw_bottom).collect::<std::collections::BTreeSet<_>>();
    let mut rest = state.piles.draw.iter().filter(|c| !known.contains(&c.handle())).map(|c| knowledge::class_of(state, c)).collect::<Option<Vec<_>>>()?;
    rest.sort();
    let mut rest = rest.into_iter();
    for slot in out.iter_mut().filter(|s| s.is_none()) { *slot = rest.next(); }
    out.into_iter().collect()
}

impl Combat {
    /// Design §4.6.1: whether this combat is a supported public root, and why not (a class of reason, never a value).
    pub(crate) fn belief_support(&self) -> Result<(DrawKnowledge, Vec<knowledge::PrerollCandidates>), Unsupported> {
        let state = &self.state;
        if !state.knowledge.maintained { return Err(Unsupported::NoPublicKnowledge) }
        if state.fault.is_some() { return Err(Unsupported::Faulted) }
        if state.outcome.is_some() { return Err(Unsupported::Terminal) }
        if !matches!(state.phase, Phase::Decision | Phase::Selection) { return Err(Unsupported::NotAtDecision) }
        if super::view::public_rules(state).hide_intents { return Err(Unsupported::HiddenIntents) }
        if !leak(Leak::AllowAll) { allow_list(state)?; }
        let draw = self.draw_knowledge().map_err(|Unrepresentable| Unsupported::UnrepresentableKnowledge)?;
        let prerolls = knowledge::preroll_candidates(state).map_err(|Unrepresentable| Unsupported::UnrepresentablePreroll)?;
        Ok((draw, prerolls))
    }
    /// Design §4.6.3: the canonical public root of this combat: unknown draw cards in canonical order, every stream
    /// re-derived from the zero Run root at the public identity (cursors zero), each ambiguous pre-roll at its smallest
    /// consistent value. Public-equivalent combats give equal roots.
    pub(crate) fn belief_canonical(&self, id: StreamIdentity) -> Result<Combat, Unsupported> {
        let (_, prerolls) = self.belief_support()?;
        let mut root = self.clone();
        let state = &mut root.state;
        if !leak(Leak::KeepDrawOrder) {
            let canonical = canonical_draw_classes(state).ok_or(Unsupported::UnrepresentableKnowledge)?;
            arrange_draw(state, &canonical).ok_or(Unsupported::UnrepresentableKnowledge)?;
        }
        if !leak(Leak::KeepStreams) { restream(state, &crate::run::RunRng::new([0; 32], super::rng::Partition::Test, 0), id)?; }
        for p in &prerolls { if p.values.len() > 1 && !leak(Leak::KeepPreroll) { knowledge::set_preroll(state, p, p.values[0]); } }
        finish(state)?;
        Ok(root)
    }
    /// Design §4.6.2: the world of sample `key` from this canonical root: R1 the draw order from the exact knowledge,
    /// R2 every stream from the research Run root, R3 each ambiguous pre-roll uniform on its consistent set.
    pub(crate) fn belief_sample(&self, key: &[u8; 32], id: StreamIdentity) -> Result<Combat, Unsupported> {
        let (draw, prerolls) = self.belief_support()?;
        let mut world = self.clone();
        let state = &mut world.state;
        let classes = sample_draw_classes(&draw, &mut ResearchStream::new(hmac_sha256(key, b"r1-draw-order")));
        if !leak(Leak::KeepDrawOrder) { arrange_draw(state, &classes).ok_or(Unsupported::UnrepresentableKnowledge)?; }
        restream(state, &crate::run::RunRng::new(hmac_sha256(key, b"r2-run-root"), super::rng::Partition::Test, 0), id)?;
        for p in &prerolls {
            if p.values.len() < 2 { continue }
            let label = [b"r3-preroll\0".as_slice(), &[p.slot, p.kind as u8]].concat();
            let pick = ResearchStream::new(hmac_sha256(key, &label)).below(p.values.len() as u128) as usize;
            knowledge::set_preroll(state, p, p.values[pick]);
        }
        finish(state)?;
        Ok(world)
    }
}
/// Refresh the intent outputs from the (re)set bases (the engine's own refresh law) and resynchronise the knowledge's
/// physical bookkeeping with the new arrangement; the knowledge value itself is unchanged.
fn finish(state: &mut State) -> Result<(), Unsupported> {
    super::effects::refresh_enemy_values(state).map_err(|_| Unsupported::UnrepresentablePreroll)?;
    knowledge::resync(state);
    Ok(())
}

/// Fuzz oracle only (never public; design §4.10 P2 in spire-fuzz search mode): perturb hidden state a public root must
/// not see — every stream key and cursor, the unknown draw order within the knowledge's support, an ambiguous pre-roll
/// within its consistent set. Returns a bitmask of the perturbations applied (1 streams, 2 draw order, 4 pre-roll).
#[cfg(any(test, feature = "fuzz-diagnostics"))]
impl Combat {
    pub fn fuzz_perturb_hidden(&mut self, salt: u8) -> u8 {
        let state = &mut self.state;
        let mut applied = 0;
        let bump = |s: &super::rng::StreamState| super::rng::StreamState::new(s.id().clone(), [salt | 1; 32], s.word_index().saturating_add(u128::from(salt) + 1)).ok();
        if let Some(x) = bump(&state.rng.shuffle) { state.rng.shuffle = x; applied |= 1; }
        if let Some(x) = bump(&state.rng.enemy_ai) { state.rng.enemy_ai = x; }
        state.card_rng = state.card_rng.as_ref().and_then(bump).or(state.card_rng.take());
        for e in &mut state.enemies { e.full_ai_rng = e.full_ai_rng.as_ref().and_then(bump).or(e.full_ai_rng.take()); }
        if let Ok(k) = knowledge_of(state) {
            let len = state.piles.draw.len();
            let unknown = state.public.known_draw.len()..len.saturating_sub(state.public.known_draw_bottom.len());
            let classes = |d: &[model::OwnedCard], st: &State| d.iter().rev().map(|c| knowledge::class_of(st, c)).collect::<Option<Vec<_>>>();
            'swap: for i in unknown.clone() { for j in unknown.clone() {
                if i >= j || (i + j + salt as usize) % 3 == 1 { continue }
                let mut trial = state.piles.draw.clone();
                trial.swap(len - 1 - i, len - 1 - j);
                if let (Some(seq), Some(before)) = (classes(&trial, state), classes(&state.piles.draw, state)) {
                    if seq != before && k.supports(&seq) { state.piles.draw = trial; knowledge::resync(state); applied |= 2; break 'swap }
                }
            } }
        }
        if let Some(p) = knowledge::preroll_candidates(state).ok().and_then(|ps| ps.into_iter().find(|p| p.values.len() > 1)) {
            let base = state.enemies.iter().find(|e| e.instance_id == p.enemy).map(|e| e.full_damage_entries[&p.intent].base);
            if let Some(other) = p.values.iter().copied().find(|v| Some(*v) != base) {
                knowledge::set_preroll(state, &p, other);
                if super::effects::refresh_enemy_values(state).is_ok() { applied |= 4; }
            }
        }
        applied
    }
}
#[cfg(any(test, feature = "fuzz-diagnostics"))]
fn knowledge_of(state: &State) -> Result<DrawKnowledge, Unrepresentable> {
    if !state.knowledge.maintained { return Err(Unrepresentable) }
    state.knowledge.draw.clone()
}

/// Test support: the exact distribution a sampler induces, by enumerating every path of its uniform draws.
#[cfg(test)]
pub(crate) mod exact {
    use super::*;
    /// Replays a fixed choice prefix, then takes 0, recording each bound (exact enumeration of a sampler's draws).
    struct Scripted { path: Vec<u128>, bounds: Vec<u128> }
    impl Uniform for Scripted {
        fn below(&mut self, bound: u128) -> u128 {
            let i = self.bounds.len();
            self.bounds.push(bound);
            if i >= self.path.len() { self.path.push(0) }
            self.path[i]
        }
    }
    /// Every outcome of `sample` with its exact probability (numerator over the product of the bounds on its path).
    pub(crate) fn enumerate<T: Ord + Clone>(mut sample: impl FnMut(&mut dyn FnMut(u128) -> u128) -> T) -> Vec<(T, u128, u128)> {
        let mut out = vec![];
        let mut path: Vec<u128> = vec![];
        loop {
            let mut s = Scripted { path: path.clone(), bounds: vec![] };
            let value = sample(&mut |b| s.below(b));
            let den = s.bounds.iter().product::<u128>();
            out.push((value, 1, den));
            path = s.path;
            path.truncate(s.bounds.len());
            // Odometer step over the recorded bounds.
            loop {
                let Some(last) = path.len().checked_sub(1) else { return out };
                path[last] += 1;
                if path[last] < s.bounds[last] { break }
                path.pop();
            }
        }
    }
    struct Scripting<'a>(&'a mut dyn FnMut(u128) -> u128);
    impl Uniform for Scripting<'_> { fn below(&mut self, bound: u128) -> u128 { (self.0)(bound) } }
    pub(crate) fn sampler_distribution(knowledge: &DrawKnowledge) -> std::collections::BTreeMap<Vec<CardClass>, (u128, u128)> {
        let mut dist = std::collections::BTreeMap::new();
        for (seq, n, d) in enumerate(|f| sample_draw_classes(knowledge, &mut Scripting(f))) {
            let entry = dist.entry(seq).or_insert((0u128, 1u128));
            let (a, b) = *entry;
            let g = gcd(b, d);
            let (num, den) = (a * (d / g) + n * (b / g), b / g * d);
            let g = gcd(num, den).max(1);
            *entry = (num / g, den / g);
        }
        dist
    }
    fn gcd(a: u128, b: u128) -> u128 { if b == 0 { a } else { gcd(b, a % b) } }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn research_stream_is_uniform_and_deterministic() {
        let mut a = ResearchStream::new([7; 32]);
        let mut b = ResearchStream::new([7; 32]);
        let xs = (0..64).map(|_| a.below(6)).collect::<Vec<_>>();
        assert_eq!(xs, (0..64).map(|_| b.below(6)).collect::<Vec<_>>());
        assert!((0..6).all(|v| xs.contains(&v)) && xs.iter().all(|x| *x < 6));
        let mut c = ResearchStream::new([8; 32]);
        assert_ne!(xs, (0..64).map(|_| c.below(6)).collect::<Vec<_>>());
    }
}
