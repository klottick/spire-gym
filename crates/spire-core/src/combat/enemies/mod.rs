//! Native full-run enemy handlers. Species behavior is queued into the shared
//! combat resolver; this module does not own a second queue or simulator.
use super::{Actor, ActorEnemyId, ActorIntentId, DamagePayload, Effect, EffectKind,
    EnemyState, FullCombatSetup, FullEnemySetup, FullIntentId, FullPowerId,
    InputError, PlayerState, Source};
use super::model::{EnemyHandle, State};
use super::rng::{self, RngError, StreamState};
use crate::content::{Character, EnemyId};

pub mod act1;
pub mod act2;
pub mod act3;
pub mod act4;
mod elites;
pub(super) mod guardian;
pub use guardian::GuardianEffect;
pub(super) mod hexaghost;
pub use hexaghost::HexaghostEffect;

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub(crate) enum BurningBuff { Strength,MaxHp,Metallicize,Regeneration }
pub(crate) fn apply_burning_buff(entry:&mut super::CombatEntry,buff:BurningBuff)->Result<(),crate::run::RunError>{
    let mut actors=entry.setup.enemies.clone();
    for actor in &mut actors {
        if buff==BurningBuff::MaxHp {
            let bonus=(i64::from(actor.max_hp)+2)/4;
            actor.max_hp=i32::try_from(i64::from(actor.max_hp)+bonus).map_err(|_|crate::run::RunError::Unsupported("burning maximum HP overflow"))?;
            actor.hp=i32::try_from(i64::from(actor.hp)+bonus).map_err(|_|crate::run::RunError::Unsupported("burning current HP overflow"))?;
        } else {
            let (id,amount)=match buff {BurningBuff::Strength=>("strength",2),BurningBuff::Metallicize=>("metallicize",4),BurningBuff::Regeneration=>("regeneration",3),BurningBuff::MaxHp=>unreachable!()};
            if let Some(power)=actor.powers.iter_mut().find(|p|p.id.0==id){power.state.amount=power.state.amount.checked_add(amount).ok_or(crate::run::RunError::Unsupported("burning power overflow"))?;}
            else{actor.powers.push(super::FullPowerEntry{id:FullPowerId(id),state:super::PowerState{amount,skip_first:false,monster_source_grace:false}});}
        }
    }entry.setup.enemies=actors;Ok(())
}

/// Encounter-selected native behavior profile, never a consumer-controlled AI policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnemyProfile { Standard, JawWormHorde, LagavulinAwake, SentryCenter }
impl EnemyProfile {
    pub fn valid_for(self, id: EnemyId) -> bool {
        matches!((self, id), (Self::Standard, _) | (Self::JawWormHorde, EnemyId::JawWorm)
            |(Self::LagavulinAwake,EnemyId::Lagavulin)|(Self::SentryCenter,EnemyId::Sentry))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JawWormVariant { Act1, HordeHard }
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum AwakenedPhase{First,Rebirth,Second}

/// Native typed continuation data for queued Slime replacement. The live frame
/// belongs to combat State; diagnostics must not expose private RNG keys.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ReplacementProgress {
    pub(crate) parent: EnemyHandle,
    pub(crate) children: Vec<EnemyId>,
    pub(crate) inherited_hp: i32,
    pub(crate) next_child: usize,
    pub(crate) prepared: Vec<EnemyState>,
    pub(crate) latest_child_rng: Option<StreamState>,
}

/// Finite per-actor behavior state. Selection history stays in the common
/// `EnemyState`; these variants hold only species-local counters/phase data.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EnemyMemory {
    Cultist { ascension: u8 },
    JawWorm { ascension: u8, variant: JawWormVariant },
    GremlinNob { ascension: u8 },
    Lagavulin { sleep_turns:u8, awake:bool, attacks:u8 },
    Sentry { initial_beam:bool },
    Guardian { quota:i32, loss:i32, open:bool, latched:bool },
    Hexaghost { divider_base:Option<i32>,cycle:u8,sear_upgraded:bool },
    SphericGuardian,
    Byrd { flying:bool },
    Chosen, Snecko,
    Centurion, Mystic,
    GremlinLeader, GremlinMad, GremlinSneaky, GremlinFat, GremlinShield, GremlinWizard { charge:u8 },
    SlaverBlue, Taskmaster, SlaverRed { used_entangle:bool }, BookOfStabbing { stab_count:u32 },
    Champ { turns:u32,forges:u8,angered:bool },
    Collector { turns_taken:u32,ult_used:bool,initial_spawn:bool,slots:std::collections::BTreeMap<u8,EnemyHandle> }, TorchHead,
    BronzeAutomaton { turns:u8 }, BronzeOrb { used_stasis:bool },
    OrbWalker,Repulsor,Exploder {turns:u8},Spiker {buffs:u8},
    Darkling {center:bool,nip:i32,first:bool},
    SpireGrowth,Transient{turns:u8,base:i32},Maw{turns:u32,roared:bool},
    WrithingMass{first:bool,parasite_used:bool,parasite_pending:bool},
    GiantHead{count:i8},
    Nemesis{first:bool,cooldown:u8},
    Reptomancer{first:bool,slots:std::collections::BTreeMap<u8,EnemyHandle>},Dagger{first:bool},
    AwakenedOne{phase:AwakenedPhase,first:bool},
    TimeEater{haste_used:bool},
    Deca{attacking:bool},Donu{attacking:bool},
    SpireShield{selections:u32},SpireSpear{selections:u32},
    LouseRed { curl_up_used: bool },
    LouseGreen { curl_up_used: bool },
    AcidSlime { split_triggered: bool },
    SpikeSlime { split_triggered: bool },
    SlimeBoss { split_triggered: bool },
    CorruptHeart { selection_count:u32, first_attack:Option<bool>, buff_count:u32, last_buff_selection:Option<u32> },
}

impl EnemyMemory {
    pub fn for_enemy(id: EnemyId) -> Result<Self, InputError> {
        Self::for_enemy_at_ascension(id, 0)
    }

    pub fn for_enemy_at_ascension(id: EnemyId, ascension: u8) -> Result<Self, InputError> {
        Self::for_profile(id, ascension, EnemyProfile::Standard)
    }

    pub fn for_profile(id: EnemyId, ascension: u8, profile: EnemyProfile) -> Result<Self, InputError> {
        if ascension > 20 {
            return Err(InputError::Unsupported("enemy ascension must be within 0..=20".into()));
        }
        if !profile.valid_for(id) || (ascension != 0
            && !matches!(id, EnemyId::Cultist | EnemyId::JawWorm | EnemyId::GremlinNob)) {
            return Err(InputError::Unsupported("enemy profile/ascension unavailable".into()));
        }
        match id {
            EnemyId::Deca=>Ok(Self::Deca{attacking:true}),EnemyId::Donu=>Ok(Self::Donu{attacking:false}),
            EnemyId::SpireShield=>Ok(Self::SpireShield{selections:0}),EnemyId::SpireSpear=>Ok(Self::SpireSpear{selections:0}),
            EnemyId::CorruptHeart=>Ok(Self::CorruptHeart{selection_count:0,first_attack:None,buff_count:0,last_buff_selection:None}),
            EnemyId::TimeEater=>Ok(Self::TimeEater{haste_used:false}),
            EnemyId::AwakenedOne=>Ok(Self::AwakenedOne{phase:AwakenedPhase::First,first:true}),
            EnemyId::Reptomancer=>Ok(Self::Reptomancer{first:true,slots:Default::default()}),EnemyId::Dagger=>Ok(Self::Dagger{first:true}),
            EnemyId::GiantHead=>Ok(Self::GiantHead{count:5}),
            EnemyId::Nemesis=>Ok(Self::Nemesis{first:true,cooldown:0}),
            EnemyId::WrithingMass=>Ok(Self::WrithingMass{first:true,parasite_used:false,parasite_pending:false}),
            EnemyId::SpireGrowth=>Ok(Self::SpireGrowth),EnemyId::Transient=>Ok(Self::Transient{turns:0,base:30}),EnemyId::Maw=>Ok(Self::Maw{turns:0,roared:false}),
            EnemyId::Darkling=>Ok(Self::Darkling{center:false,nip:7,first:true}),
            EnemyId::OrbWalker=>Ok(Self::OrbWalker),EnemyId::Repulsor=>Ok(Self::Repulsor),EnemyId::Exploder=>Ok(Self::Exploder{turns:0}),EnemyId::Spiker=>Ok(Self::Spiker{buffs:0}),
            EnemyId::Cultist => Ok(Self::Cultist { ascension }),
            EnemyId::JawWorm => Ok(Self::JawWorm { ascension, variant: match profile {
                EnemyProfile::Standard => JawWormVariant::Act1,
                EnemyProfile::JawWormHorde => JawWormVariant::HordeHard,
                _=>return Err(InputError::Invariant("invalid Jaw Worm profile".into())),
            } }),
            EnemyId::GremlinNob => Ok(Self::GremlinNob { ascension }),
            EnemyId::Lagavulin=>Ok(Self::Lagavulin{sleep_turns:0,awake:profile==EnemyProfile::LagavulinAwake,attacks:0}),
            EnemyId::Sentry=>Ok(Self::Sentry{initial_beam:profile==EnemyProfile::SentryCenter}),
            EnemyId::Guardian=>Ok(Self::Guardian{quota:30,loss:0,open:true,latched:false}),
            EnemyId::Hexaghost=>Ok(Self::Hexaghost{divider_base:None,cycle:0,sear_upgraded:false}),
            EnemyId::SphericGuardian=>Ok(Self::SphericGuardian),
            EnemyId::Byrd=>Ok(Self::Byrd{flying:true}),
            EnemyId::Chosen=>Ok(Self::Chosen),EnemyId::Snecko=>Ok(Self::Snecko),
            EnemyId::Centurion=>Ok(Self::Centurion),EnemyId::Mystic=>Ok(Self::Mystic),
            _ if ascension != 0 => Err(InputError::Unsupported(
                "ascension-specific enemy behavior is unavailable for this species".into())),
            EnemyId::LouseRed => Ok(Self::LouseRed { curl_up_used: false }),
            EnemyId::LouseGreen => Ok(Self::LouseGreen { curl_up_used: false }),
            EnemyId::AcidSlimeS | EnemyId::AcidSlimeM | EnemyId::AcidSlimeL =>
                Ok(Self::AcidSlime { split_triggered: false }),
            EnemyId::SpikeSlimeS | EnemyId::SpikeSlimeM | EnemyId::SpikeSlimeL =>
                Ok(Self::SpikeSlime { split_triggered: false }),
            EnemyId::SlimeBoss => Ok(Self::SlimeBoss { split_triggered: false }),
            EnemyId::Unknown(name) => Err(InputError::Unsupported(format!("enemy memory missing for {name}"))),
            EnemyId::GremlinLeader=>Ok(Self::GremlinLeader),EnemyId::GremlinMad=>Ok(Self::GremlinMad),EnemyId::GremlinSneaky=>Ok(Self::GremlinSneaky),EnemyId::GremlinFat=>Ok(Self::GremlinFat),EnemyId::GremlinShield=>Ok(Self::GremlinShield),EnemyId::GremlinWizard=>Ok(Self::GremlinWizard{charge:1}),
            EnemyId::SlaverBlue=>Ok(Self::SlaverBlue),EnemyId::Taskmaster=>Ok(Self::Taskmaster),EnemyId::SlaverRed=>Ok(Self::SlaverRed{used_entangle:false}),EnemyId::BookOfStabbing=>Ok(Self::BookOfStabbing{stab_count:1}),
            EnemyId::Champ=>Ok(Self::Champ{turns:0,forges:0,angered:false}),
            EnemyId::Collector=>Ok(Self::Collector{turns_taken:0,ult_used:false,initial_spawn:true,slots:Default::default()}),EnemyId::TorchHead=>Ok(Self::TorchHead),
            EnemyId::BronzeAutomaton=>Ok(Self::BronzeAutomaton{turns:0}),EnemyId::BronzeOrb=>Ok(Self::BronzeOrb{used_stasis:false}),
        }
    }

    pub fn ascension(&self) -> u8 {
        match self {
            Self::Cultist { ascension } | Self::JawWorm { ascension, .. }
            | Self::GremlinNob { ascension } => *ascension,
            _ => 0,
        }
    }

    pub fn valid_for(&self, id: EnemyId) -> bool {
        match (id, self) {
            (EnemyId::Deca,Self::Deca{..})|(EnemyId::Donu,Self::Donu{..})=>true,
            (EnemyId::SpireShield,Self::SpireShield{..})|(EnemyId::SpireSpear,Self::SpireSpear{..})=>true,
            (EnemyId::CorruptHeart,Self::CorruptHeart{selection_count,first_attack,buff_count,last_buff_selection})=>{
                let maximum=*selection_count/3;
                let minimum=maximum.saturating_sub(u32::from(*selection_count>0&&selection_count%3==0));
                *selection_count<=i32::MAX as u32&&(*selection_count==0)==first_attack.is_none()
                    &&*buff_count>=minimum&&*buff_count<=maximum&&*buff_count<=i32::MAX as u32
                    &&((*buff_count==0&&last_buff_selection.is_none())
                        ||(*buff_count>0&&*last_buff_selection==buff_count.checked_mul(3).and_then(|n|n.checked_sub(1))))
            },
            (EnemyId::TimeEater,Self::TimeEater{..})=>true,
            (EnemyId::AwakenedOne,Self::AwakenedOne{..})=>true,
            (EnemyId::Reptomancer,Self::Reptomancer{slots,..})=>slots.keys().all(|slot|*slot<4)&&slots.values().collect::<std::collections::BTreeSet<_>>().len()==slots.len(),
            (EnemyId::Dagger,Self::Dagger{..})=>true,
            (EnemyId::GiantHead,Self::GiantHead{count})=>(-6..=5).contains(count),
            (EnemyId::Nemesis,Self::Nemesis{cooldown,..})=>*cooldown<=2,
            (EnemyId::WrithingMass,Self::WrithingMass{parasite_pending,parasite_used,..})=>!*parasite_pending||*parasite_used,
            (EnemyId::SpireGrowth,Self::SpireGrowth)=>true,
            (EnemyId::Transient,Self::Transient{turns,base})=>*turns<=5&&(*base==30+10*i32::from(*turns)||*turns>0&&*base==20+10*i32::from(*turns)),
            (EnemyId::Maw,Self::Maw{turns,..})=>*turns<i32::MAX as u32,
            (EnemyId::Darkling,Self::Darkling{nip:7..=11,..})=>true,
            (EnemyId::OrbWalker,Self::OrbWalker)|(EnemyId::Repulsor,Self::Repulsor)|(EnemyId::Exploder,Self::Exploder{turns:0..=3})|(EnemyId::Spiker,Self::Spiker{buffs:0..=6})=>true,
            (EnemyId::Guardian,Self::Guardian{quota,loss,..})=>*quota>=30&&*loss>=0&&*loss<*quota,
            (EnemyId::Hexaghost,Self::Hexaghost{divider_base,cycle,..})=>*cycle<7&&divider_base.is_none_or(|base|(1..=84).contains(&base)),
            (EnemyId::SphericGuardian,Self::SphericGuardian)|(EnemyId::Byrd,Self::Byrd{..})=>true,
            (EnemyId::Chosen,Self::Chosen)|(EnemyId::Snecko,Self::Snecko)=>true,
            (EnemyId::Centurion,Self::Centurion)|(EnemyId::Mystic,Self::Mystic)=>true,
            (EnemyId::GremlinLeader,Self::GremlinLeader)|(EnemyId::GremlinMad,Self::GremlinMad)|(EnemyId::GremlinSneaky,Self::GremlinSneaky)|(EnemyId::GremlinFat,Self::GremlinFat)|(EnemyId::GremlinShield,Self::GremlinShield)|(EnemyId::GremlinWizard,Self::GremlinWizard{charge:0..=3})=>true,
            (EnemyId::SlaverBlue,Self::SlaverBlue)|(EnemyId::Taskmaster,Self::Taskmaster)|(EnemyId::SlaverRed,Self::SlaverRed{..})=>true,
            (EnemyId::BookOfStabbing,Self::BookOfStabbing{stab_count})=>(1..=i32::MAX as u32).contains(stab_count),
            (EnemyId::Champ,Self::Champ{turns,forges,..})=>*turns<=i32::MAX as u32&&*forges<=2,
            (EnemyId::Collector,Self::Collector{turns_taken,slots,..})=>*turns_taken<=i32::MAX as u32&&slots.keys().all(|s|*s<2)&&slots.values().collect::<std::collections::BTreeSet<_>>().len()==slots.len(),
            (EnemyId::TorchHead,Self::TorchHead)=>true,
            (EnemyId::BronzeAutomaton,Self::BronzeAutomaton{turns:0..=4})|(EnemyId::BronzeOrb,Self::BronzeOrb{..})=>true,
            (EnemyId::Cultist, Self::Cultist { ascension })
            | (EnemyId::JawWorm, Self::JawWorm { ascension, .. }) => *ascension <= 20,
            (EnemyId::GremlinNob, Self::GremlinNob { ascension }) => *ascension <= 20,
            (EnemyId::LouseRed, Self::LouseRed { .. })
            | (EnemyId::Lagavulin,Self::Lagavulin{sleep_turns:0..=3,attacks:0..=2,..})
            | (EnemyId::Sentry,Self::Sentry{..})
            | (EnemyId::LouseGreen, Self::LouseGreen { .. })
            | (EnemyId::AcidSlimeS | EnemyId::AcidSlimeM | EnemyId::AcidSlimeL,
                Self::AcidSlime { .. })
            | (EnemyId::SpikeSlimeS | EnemyId::SpikeSlimeM | EnemyId::SpikeSlimeL,
                Self::SpikeSlime { .. })
            | (EnemyId::SlimeBoss, Self::SlimeBoss { .. }) => true,
            _ => false,
        }
    }
}

/// Derive a split child's actor-owned stream from semantic parent identity and
/// ordinal, independent of the parent's mutable draw cursor or sibling order.
pub fn split_child_stream(parent: &StreamState, child: &EnemyHandle, ordinal: u32)
    -> Result<StreamState, RngError> {
    if parent.id().domain() != rng::Domain::RunCombatEnemyAi || parent.id().scope().len() != 2 {
        return Err(RngError::Invalid("split parent must own a run-combat enemy stream".into()));
    }
    let mut message = b"spire-lab/rng/v1/slime-split-child\0".to_vec();
    let fields = [parent.id().domain().as_str(), parent.id().scope()[0].as_str(),
        parent.id().scope()[1].as_str(), child.as_str()];
    message.extend((fields.len() as u32).to_be_bytes());
    for field in fields {
        message.extend((field.len() as u32).to_be_bytes());
        message.extend(field.as_bytes());
    }
    message.extend(ordinal.to_be_bytes());
    let key = rng::hmac_sha256(parent.key(), &message);
    let id = rng::StreamId::new(rng::Domain::RunCombatEnemyAi,
        vec![parent.id().scope()[0].clone(), child.as_str().to_owned()])?;
    StreamState::new(id, key, 0)
}

/// Prepare one fresh native Slime child without mutating combat state. The
/// caller owns staging/commit so partially prepared children survive a fault.
pub(super) fn prepare_split_child(id: EnemyId, child: EnemyHandle,
    hp: i32, stream: StreamState) -> Result<EnemyState, SelectError> {
    let memory = EnemyMemory::for_enemy(id)?;
    let selected = select_move(id, &[], &memory, &stream)?;
    let full_damage_entries = damage_bases(id).map_err(SelectError::Invalid)?.into_iter()
        .map(|(intent, base)| (intent, super::DamageEntry { base, output: base }))
        .collect::<std::collections::BTreeMap<_, _>>();
    let full_powers = if matches!(id, EnemyId::AcidSlimeL | EnemyId::SpikeSlimeL) {
        vec![super::FullPowerEntry { id: FullPowerId("split"), state: super::PowerState {
            amount: 1, skip_first: false, monster_source_grace: false,
        } }]
    } else { vec![] };
    let intent = selected.intent;
    let intent_damage = full_damage_entries.get(&intent).map(|entry| entry.output);
    Ok(EnemyState { instance_id: child, enemy_id: ActorEnemyId::Full(id), hp, max_hp: hp,
        block: 0, powers: vec![], full_powers,
        intent_id: ActorIntentId::Full(intent), move_history: vec![ActorIntentId::Full(intent)],
        selected_move: ActorIntentId::Full(intent), damage_entries: Default::default(),
        full_damage_entries, full_ai_rng: Some(selected.stream),
        full_memory: Some(selected.memory), intent_damage, life: super::ActorLife::Active })
}

/// Apply the validated species replacement on the existing combat state and
/// retain each prepared child in the current fault frame until atomic commit.
pub(super) fn apply_replacement(state: &mut State, parent_handle: &EnemyHandle,
    child_ids: &[EnemyId]) -> Result<(), super::EffectError> {
    use super::model::ActorLife;

    if state.replacement_progress.is_none() {
        let parent = state.enemies.iter().find(|enemy| &enemy.instance_id == parent_handle)
            .ok_or_else(|| InputError::Invariant("replacement parent missing".into()))?;
        if parent.life != ActorLife::Active || parent.hp <= 0 {
            return Err(InputError::Invariant("replacement parent is not active".into()).into());
        }
        state.replacement_progress = Some(ReplacementProgress {
            parent: parent_handle.clone(), children: child_ids.to_vec(),
            inherited_hp: parent.hp, next_child: 0, prepared: Vec::new(),
            latest_child_rng: None,
        });
    }

    let progress = state.replacement_progress.as_ref().unwrap();
    if progress.parent != *parent_handle || progress.children != child_ids
        || progress.next_child != progress.prepared.len()
        || progress.next_child > child_ids.len()
        || progress.inherited_hp <= 0 {
        return Err(InputError::Invariant("replacement continuation changed or invalid".into()).into());
    }

    while state.replacement_progress.as_ref().unwrap().next_child < child_ids.len() {
        let index = state.replacement_progress.as_ref().unwrap().next_child;
        let progress = state.replacement_progress.as_ref().unwrap();
        let child_handle = EnemyHandle::parse(&format!("{}/split/{index}", parent_handle.as_str()))?;
        if state.enemies.iter().any(|enemy| enemy.instance_id == child_handle)
            || progress.prepared.iter().any(|enemy| enemy.instance_id == child_handle) {
            return Err(InputError::Invariant("split child identity collision".into()).into());
        }
        let parent_stream = state.enemies.iter().find(|enemy| &enemy.instance_id == parent_handle)
            .and_then(|enemy| enemy.full_ai_rng.as_ref())
            .ok_or_else(|| InputError::Invariant("replacement parent AI stream missing".into()))?;
        let child_stream = split_child_stream(parent_stream, &child_handle, index as u32)?;
        state.replacement_progress.as_mut().unwrap().latest_child_rng = Some(child_stream.clone());
        let prepared = match prepare_split_child(child_ids[index], child_handle,
            state.replacement_progress.as_ref().unwrap().inherited_hp, child_stream) {
            Ok(child) => child,
            Err(SelectError::Invalid(error)) => return Err(error.into()),
            Err(SelectError::Random(error)) => return Err(error.into()),
        };
        let progress = state.replacement_progress.as_mut().unwrap();
        progress.latest_child_rng = prepared.full_ai_rng.clone();
        progress.prepared.push(prepared);
        progress.next_child += 1;
    }

    // Do every fallible validation/refresh on a detached candidate. The live
    // actor vector and parent remain untouched until the replacement is ready.
    let mut candidate = state.clone();
    let parent = candidate.enemies.iter_mut().find(|enemy| &enemy.instance_id == parent_handle)
        .ok_or_else(|| InputError::Invariant("replacement parent disappeared".into()))?;
    if parent.life != ActorLife::Active || parent.hp <= 0 {
        return Err(InputError::Invariant("replacement parent no longer active".into()).into());
    }
    parent.hp = 0;
    parent.life = ActorLife::Replaced;
    let prepared = candidate.replacement_progress.as_ref().unwrap().prepared.clone();
    candidate.enemies.extend(prepared);
    let stored = candidate.enemies.len();
    let active = candidate.enemies.iter().filter(|enemy|
        enemy.life == ActorLife::Active && enemy.hp > 0).count();
    if u64::try_from(stored).is_err() || active > 9 {
        return Err(InputError::Invariant("replacement actor count exceeds native bounds".into()).into());
    }
    // Preserve the accepted Slime vector-order turn semantics; only living slots are reassigned.
    for (slot,enemy) in candidate.enemies.iter().filter(|e|e.life==ActorLife::Active&&e.hp>0).enumerate(){candidate.actor_slots.insert(enemy.instance_id.clone(),slot as u8);}
    for enemy in &candidate.enemies{candidate.actor_slots.entry(enemy.instance_id.clone()).or_insert(0);}
    super::effects::refresh_enemy_values(&mut candidate)?;
    state.enemies = candidate.enemies;
    state.actor_slots=candidate.actor_slots;
    state.replacement_progress = None;
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MoveSelection {
    pub intent: FullIntentId,
    pub stream: StreamState,
    pub memory: EnemyMemory,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SelectError {
    Invalid(InputError),
    Random(RngError),
}

impl From<InputError> for SelectError { fn from(value: InputError) -> Self { Self::Invalid(value) } }
impl From<RngError> for SelectError { fn from(value: RngError) -> Self { Self::Random(value) } }

/// Construct ordinary shared-resolver effects for the actor's selected intent.
pub fn move_work(enemy: &EnemyState) -> Result<Vec<Effect>, InputError> {
    if enemy.enemy_id==ActorEnemyId::Full(EnemyId::CorruptHeart){return act4::heart::move_work(enemy)}
    if matches!(enemy.enemy_id,ActorEnemyId::Full(EnemyId::Deca|EnemyId::Donu)){return Err(InputError::Unsupported("Deca/Donu payload requires live group context".into()))}
    if enemy.enemy_id==ActorEnemyId::Full(EnemyId::TimeEater){return act3::time_eater::move_work(enemy)}
    if enemy.enemy_id==ActorEnemyId::Full(EnemyId::AwakenedOne){return act3::awakened::move_work(enemy)}
    if matches!(enemy.enemy_id,ActorEnemyId::Full(id) if act3::reptomancer::supports(id)){return act3::reptomancer::move_work(enemy)}
    if matches!(enemy.enemy_id,ActorEnemyId::Full(id) if act3::elites::supports(id)){return act3::elites::move_work(enemy)}
    if enemy.enemy_id==ActorEnemyId::Full(EnemyId::WrithingMass){return act3::writhing::move_work(enemy)}
    if matches!(enemy.enemy_id,ActorEnemyId::Full(id) if act3::ordinary::supports(id)){return act3::ordinary::move_work(enemy)}
    if enemy.enemy_id==ActorEnemyId::Full(EnemyId::Darkling){return act3::darklings::move_work(enemy)}
    if matches!(enemy.enemy_id,ActorEnemyId::Full(id) if act3::supports(id)){return act3::shapes::move_work(enemy)}
    if matches!(enemy.enemy_id,ActorEnemyId::Full(EnemyId::SphericGuardian|EnemyId::Byrd)){return act2::flight::move_work(enemy);}
    if matches!(enemy.enemy_id,ActorEnemyId::Full(EnemyId::Chosen|EnemyId::Snecko)){return act2::chosen_snecko::move_work(enemy);}
    if enemy.enemy_id==ActorEnemyId::Full(EnemyId::Guardian){return guardian::move_work(enemy);}
    if enemy.enemy_id==ActorEnemyId::Full(EnemyId::Hexaghost){return hexaghost::move_work(enemy);}
    if matches!(enemy.enemy_id,ActorEnemyId::Full(EnemyId::Lagavulin|EnemyId::Sentry)){return elites::move_work(enemy);}
    let ActorEnemyId::Full(id) = &enemy.enemy_id else {
        return Err(InputError::Unsupported("enemy handler requires native full identity".into()));
    };
    let ActorIntentId::Full(intent) = &enemy.intent_id else {
        return Err(InputError::Invariant("native enemy has non-full intent".into()));
    };
    if matches!(*id, EnemyId::AcidSlimeS | EnemyId::AcidSlimeM | EnemyId::AcidSlimeL
        | EnemyId::SpikeSlimeS | EnemyId::SpikeSlimeM | EnemyId::SpikeSlimeL | EnemyId::SlimeBoss) {
        return act1::move_work(enemy, *id, *intent);
    }
    let actor = Actor::Enemy(enemy.instance_id.clone());
    let work = match (*id, *intent) {
        (EnemyId::Cultist, FullIntentId::Incantation) => {
            let ascension = match enemy.full_memory.as_ref() {
                Some(EnemyMemory::Cultist { ascension }) => *ascension,
                _ => return Err(InputError::Invariant("Cultist ascension config missing".into())),
            };
            let amount = if ascension >= 17 { 5 } else if ascension >= 2 { 4 } else { 3 };
            vec![effect(actor.clone(), actor,
                EffectKind::ApplyFullPower { id: FullPowerId("ritual"), amount })]
        },
        (EnemyId::Cultist, FullIntentId::Attack) => vec![damage(actor, *intent)],
        (EnemyId::GremlinNob, FullIntentId::Bellow) => {
            let ascension = match enemy.full_memory.as_ref() {
                Some(EnemyMemory::GremlinNob { ascension }) => *ascension,
                _ => return Err(InputError::Invariant("Gremlin Nob ascension config missing".into())),
            };
            vec![effect(actor.clone(), actor, EffectKind::ApplyFullPower {
                id: FullPowerId("enrage"), amount: if ascension >= 18 { 3 } else { 2 },
            })]
        },
        (EnemyId::GremlinNob, intent) if intent.as_str() == "skull_bash" => {
            vec![damage(actor.clone(), intent), effect(actor.clone(), Actor::Player,
                EffectKind::ApplyFullPower { id: FullPowerId("vulnerable"), amount: 2 })]
        },
        (EnemyId::GremlinNob, intent) if intent.as_str() == "bull_rush" => vec![damage(actor, intent)],
        (EnemyId::JawWorm, FullIntentId::Chomp) => vec![damage(actor, *intent)],
        (EnemyId::JawWorm, FullIntentId::Thrash) => vec![damage(actor.clone(), *intent),
            effect(actor.clone(), actor, EffectKind::GainBlock(5))],
        (EnemyId::JawWorm, FullIntentId::Bellow) => jaw_bellow_work(enemy)?,
        (EnemyId::LouseRed, FullIntentId::Bite) | (EnemyId::LouseGreen, FullIntentId::Bite) => {
            vec![damage(actor, *intent)]
        }
        (EnemyId::LouseRed, FullIntentId::Grow) => vec![effect(actor.clone(), actor,
            EffectKind::ApplyFullPower { id: FullPowerId("strength"), amount: 3 })],
        (EnemyId::LouseGreen, FullIntentId::SpitWeb) => vec![effect(actor.clone(), Actor::Player,
            EffectKind::ApplyFullPower { id: FullPowerId("weak"), amount: 2 })],
        (EnemyId::Unknown(name), _) => return Err(InputError::Unsupported(format!("unhandled full enemy {name}"))),
        _ => return Err(InputError::Unsupported(format!("unhandled full enemy intent {}", intent.as_str()))),
    };
    Ok(work)
}

fn jaw_bellow_work(enemy: &EnemyState) -> Result<Vec<Effect>, InputError> {
    let ascension = match enemy.full_memory.as_ref() {
        Some(EnemyMemory::JawWorm { ascension, .. }) => *ascension,
        _ => return Err(InputError::Invariant("Jaw Worm ascension config missing".into())),
    };
    let strength = if ascension >= 17 { 5 } else if ascension >= 2 { 4 } else { 3 };
    let block = if ascension >= 17 { 9 } else { 6 };
    let actor = Actor::Enemy(enemy.instance_id.clone());
    Ok(vec![effect(actor.clone(), actor.clone(), EffectKind::ApplyFullPower {
        id: FullPowerId("strength"), amount: strength }),
        effect(actor.clone(), actor, EffectKind::GainBlock(block))])
}

/// One-time setup work for an encounter-selected native profile. These are
/// ordinary resolver effects, not a fabricated monster turn or history entry.
pub(super) fn prebattle_work(enemy: &EnemyState) -> Result<Vec<Effect>, InputError> {
    if enemy.enemy_id==ActorEnemyId::Full(EnemyId::CorruptHeart){return Ok(act4::heart::prebattle_work(enemy))}
    match enemy.full_memory.as_ref() {
        Some(EnemyMemory::SphericGuardian)=>{let actor=Actor::Enemy(enemy.instance_id.clone());Ok(vec![effect(actor.clone(),actor,EffectKind::GainBlock(40))])},
        Some(EnemyMemory::Lagavulin{awake:false,..})=>{let actor=Actor::Enemy(enemy.instance_id.clone());Ok(vec![effect(actor.clone(),actor.clone(),EffectKind::GainBlock(8)),effect(actor.clone(),actor,EffectKind::ApplyFullPower{id:FullPowerId("metallicize"),amount:8})])},
        Some(EnemyMemory::JawWorm { variant: JawWormVariant::HordeHard, .. }) => jaw_bellow_work(enemy),
        _ => Ok(vec![]),
    }
}

fn effect(source: Actor, target: Actor, kind: EffectKind) -> Effect {
    Effect { source: Source::Actor(source), target: Some(target), kind }
}

fn damage(source: Actor, intent: FullIntentId) -> Effect {
    Effect { source: Source::Actor(source), target: Some(Actor::Player),
        kind: EffectKind::Damage(DamagePayload::FullEnemyMove(intent)) }
}

/// Select the next move using only this actor's history, memory, and AI stream.
pub fn select_move(id: EnemyId, history: &[FullIntentId], memory: &EnemyMemory,
    stream: &StreamState) -> Result<MoveSelection, SelectError> {
    if act3::elites::supports(id)&&!memory.valid_for(id){return Err(InputError::Invariant("elite memory/species mismatch".into()).into())}
    if stream.id().domain() != rng::Domain::RunCombatEnemyAi {
        return Err(InputError::Unsupported("native full enemy selection requires run enemy AI stream".into()).into());
    }
    if id==EnemyId::CorruptHeart{return act4::heart::select_move(history,memory,stream)}
    if act4::supports(id){return act4::select_move(id,history,memory,stream);}
    match (id, memory) {
        (EnemyId::Deca|EnemyId::Donu,_)=>act3::partners::select_move(id,history,memory,stream),
        (EnemyId::TimeEater,_)=>Err(InputError::Unsupported("Time Eater selection requires live owner context".into()).into()),
        (EnemyId::AwakenedOne,_)=>act3::awakened::select_move(history,memory,stream),
        (id,_) if act3::reptomancer::supports(id)=>act3::reptomancer::select_move(id,history,memory,stream,None),
        (id,_) if act3::elites::supports(id)=>act3::elites::select_move(history,memory,stream),
        (EnemyId::WrithingMass,_)=>act3::writhing::select_move(history,memory,stream),
        (id,_) if act3::ordinary::supports(id)=>act3::ordinary::select_move(id,history,memory,stream,None),
        (EnemyId::Darkling,_)=>act3::darklings::select_move(history,memory,stream,false),
        (id,_) if act3::supports(id)=>act3::shapes::select_move(id,history,memory,stream),
        (EnemyId::Guardian,EnemyMemory::Guardian{..})=>guardian::select_move(history,memory,stream),
        (EnemyId::SphericGuardian,EnemyMemory::SphericGuardian)|(EnemyId::Byrd,EnemyMemory::Byrd{..})=>act2::flight::select_move(history,memory,stream),
        (EnemyId::Chosen,EnemyMemory::Chosen)|(EnemyId::Snecko,EnemyMemory::Snecko)=>act2::chosen_snecko::select_move(history,memory,stream),
        (EnemyId::GremlinMad,EnemyMemory::GremlinMad)|(EnemyId::GremlinSneaky,EnemyMemory::GremlinSneaky)|(EnemyId::GremlinFat,EnemyMemory::GremlinFat)|(EnemyId::GremlinShield,EnemyMemory::GremlinShield)|(EnemyId::GremlinWizard,EnemyMemory::GremlinWizard{..})=>act2::gremlin_leader::select_small(history,memory,stream),
        (EnemyId::SlaverBlue,EnemyMemory::SlaverBlue)|(EnemyId::Taskmaster,EnemyMemory::Taskmaster)|(EnemyId::SlaverRed,EnemyMemory::SlaverRed{..})=>act2::slavers::select_move(history,memory,stream),
        (EnemyId::BookOfStabbing,EnemyMemory::BookOfStabbing{..})=>act2::book::select_move(history,memory,stream),
        (EnemyId::TorchHead,EnemyMemory::TorchHead)=>Ok(MoveSelection{intent:FullIntentId::from_name("torch_attack"),memory:memory.clone(),stream:stream.clone()}),
        (EnemyId::BronzeAutomaton,EnemyMemory::BronzeAutomaton{..})|(EnemyId::BronzeOrb,EnemyMemory::BronzeOrb{..})=>act2::automaton::select_move(history,memory,stream),
        (EnemyId::Hexaghost,EnemyMemory::Hexaghost{..})=>hexaghost::select_move(history,memory,stream),
        (EnemyId::Lagavulin,EnemyMemory::Lagavulin{..})|(EnemyId::Sentry,EnemyMemory::Sentry{..})=>elites::select_move(id,history,memory,stream),
        (EnemyId::Cultist, EnemyMemory::Cultist { .. }) => {
            let intent = if history.is_empty() { FullIntentId::Incantation } else { FullIntentId::Attack };
            Ok(MoveSelection { intent, stream: stream.clone(), memory: memory.clone() })
        }
        (EnemyId::GremlinNob, memory @ EnemyMemory::GremlinNob { .. }) =>
            select_gremlin_nob(history, memory, stream),
        (EnemyId::SlimeBoss, EnemyMemory::SlimeBoss { .. }) => act1::select_slime_boss(history, memory, stream),
        (EnemyId::JawWorm, EnemyMemory::JawWorm { .. }) => select_jaw(history, memory, stream),
        (EnemyId::LouseRed, EnemyMemory::LouseRed { .. }) |
        (EnemyId::LouseGreen, EnemyMemory::LouseGreen { .. }) => select_louse(id, history, memory, stream),
        (EnemyId::AcidSlimeS | EnemyId::AcidSlimeM | EnemyId::AcidSlimeL,
            EnemyMemory::AcidSlime { .. }) => act1::select_acid(id, history, memory, stream),
        (EnemyId::SpikeSlimeS | EnemyId::SpikeSlimeM | EnemyId::SpikeSlimeL,
            EnemyMemory::SpikeSlime { .. }) => act1::select_spike(id, history, memory, stream),
        (EnemyId::Unknown(name), _) => Err(InputError::Unsupported(format!("enemy selector missing for {name}")).into()),
        _ => Err(InputError::Invariant("enemy memory/identity mismatch".into()).into()),
    }
}

const NOB_BELLOW: &str = "bellow";
const NOB_SKULL_BASH: &str = "skull_bash";
const NOB_BULL_RUSH: &str = "bull_rush";

fn select_gremlin_nob(history: &[FullIntentId], memory: &EnemyMemory, stream: &StreamState)
    -> Result<MoveSelection, SelectError> {
    if !matches!(memory, EnemyMemory::GremlinNob { .. })
        || history.iter().any(|intent| ![NOB_BELLOW, NOB_SKULL_BASH, NOB_BULL_RUSH].contains(&intent.as_str()))
        || history.first().map(|intent| intent.as_str()).is_some_and(|first| first != NOB_BELLOW) {
        return Err(InputError::Unsupported("Gremlin Nob history/identity mismatch".into()).into());
    }
    let ascension = memory.ascension();
    let intent = if history.is_empty() {
        FullIntentId::from_name(NOB_BELLOW)
    } else if ascension >= 18 {
        match history.last().map(|intent| intent.as_str()) {
            Some(NOB_BELLOW) => FullIntentId::from_name(NOB_SKULL_BASH),
            Some(NOB_SKULL_BASH) => FullIntentId::from_name(NOB_BULL_RUSH),
            Some(NOB_BULL_RUSH) if history.ends_with(&[
                FullIntentId::from_name(NOB_BULL_RUSH), FullIntentId::from_name(NOB_BULL_RUSH)]) =>
                FullIntentId::from_name(NOB_SKULL_BASH),
            Some(NOB_BULL_RUSH) => FullIntentId::from_name(NOB_BULL_RUSH),
            _ => return Err(InputError::Unsupported("Gremlin Nob A18+ history mismatch".into()).into()),
        }
    } else if history.ends_with(&[
        FullIntentId::from_name(NOB_BULL_RUSH), FullIntentId::from_name(NOB_BULL_RUSH)]) {
        FullIntentId::from_name(NOB_SKULL_BASH)
    } else {
        let (roll, next) = rng::uniform_below(stream, 100)?;
        return Ok(MoveSelection {
            intent: if roll < 33 { FullIntentId::from_name(NOB_SKULL_BASH) }
                else { FullIntentId::from_name(NOB_BULL_RUSH) },
            stream: next, memory: memory.clone(),
        });
    };
    Ok(MoveSelection { intent, stream: stream.clone(), memory: memory.clone() })
}

/// Produce Enrage's on-use callback only for the exact accepted live Skill card.
/// The shared hook dispatcher invokes this while that card remains in play.
pub(super) fn gremlin_nob_card_played_work(state: &State, source: &Source)
    -> Result<Vec<Effect>, InputError> {
    let Source::Card(card_handle) = source else { return Ok(vec![]); };
    let Some(in_play) = state.in_play.as_ref() else {
        return Err(InputError::Invariant("CardPlayed hook has no live in-play card".into()));
    };
    if in_play.handle() != *card_handle {
        return Err(InputError::Invariant("CardPlayed source is not the in-play card".into()));
    }
    let Some(card) = in_play.full() else { return Ok(vec![]); };
    let Some(definition) = crate::content::Catalog::standard().card(card.id) else {
        return Err(InputError::Unsupported("CardPlayed card type is not registered".into()));
    };
    if definition.card_type != crate::content::CardType::Skill { return Ok(vec![]); }

    let mut work = Vec::new();
    for enemy in state.enemies.iter().filter(|enemy|
        enemy.life == super::ActorLife::Active && enemy.hp > 0
            && matches!(enemy.enemy_id, ActorEnemyId::Full(EnemyId::GremlinNob))) {
        let Some(enrage) = enemy.full_powers.iter().find(|power| power.id == FullPowerId("enrage"))
            else { continue; };
        let actor = Actor::Enemy(enemy.instance_id.clone());
        work.push(Effect { source: source.clone(), target: Some(actor),
            kind: EffectKind::ApplyFullPower { id: FullPowerId("strength"), amount: enrage.state.amount } });
    }
    Ok(work)
}

fn select_louse(id: EnemyId, history: &[FullIntentId], memory: &EnemyMemory,
    stream: &StreamState) -> Result<MoveSelection, SelectError> {
    use FullIntentId::{Bite, Grow, SpitWeb};
    let nonattack = if id == EnemyId::LouseRed { Grow } else { SpitWeb };
    if history.iter().any(|intent| !matches!(intent, Bite | Grow | SpitWeb))
        || (!history.is_empty() && history[0] != Bite)
        || history.iter().any(|intent| *intent == Grow && id != EnemyId::LouseRed)
        || history.iter().any(|intent| *intent == SpitWeb && id != EnemyId::LouseGreen) {
        return Err(InputError::Unsupported("Louse history/identity mismatch".into()).into());
    }
    if history.ends_with(&[nonattack, nonattack]) {
        return Ok(MoveSelection { intent: Bite, stream: stream.clone(), memory: memory.clone() });
    }
    if history.ends_with(&[Bite, Bite]) {
        return Ok(MoveSelection { intent: nonattack, stream: stream.clone(), memory: memory.clone() });
    }
    if history.is_empty() {
        return Ok(MoveSelection { intent: Bite, stream: stream.clone(), memory: memory.clone() });
    }
    let (roll, next) = rng::uniform_below(stream, 4)?;
    let intent = if roll == 0 { nonattack } else { Bite };
    Ok(MoveSelection { intent, stream: next, memory: memory.clone() })
}

/// Curl Up observes resolved HP damage at the native damage checkpoint. It
/// latches before enqueueing its ordinary Block/removal payload after existing
/// action work, but before the next decision. Already-queued later hits are not
/// synchronously blocked and a public decision never precedes this trigger.
pub(super) fn on_hit(state: &mut State, target: &EnemyHandle, source: &Source,
    normal_damage: bool, hp_before: i32, hp_damage: i32) {
    if !state.full_runtime { return; }
    guardian::on_hit(state,target,hp_damage);
    elites::on_hit(state,target,hp_damage);
    let Some(enemy) = state.enemies.iter_mut().find(|enemy| &enemy.instance_id == target) else { return; };
    if hp_damage > 0 && enemy.hp > 0 && enemy.hp <= enemy.max_hp / 2 {
        let (split_intent, split_requested) = match (&enemy.enemy_id, enemy.full_memory.as_mut()) {
            (ActorEnemyId::Full(EnemyId::AcidSlimeL), Some(EnemyMemory::AcidSlime { split_triggered }))
                if !*split_triggered => { *split_triggered = true; (FullIntentId::AcidSplit, true) },
            (ActorEnemyId::Full(EnemyId::SpikeSlimeL), Some(EnemyMemory::SpikeSlime { split_triggered }))
                if !*split_triggered => { *split_triggered = true; (FullIntentId::SpikeSplit, true) },
            (ActorEnemyId::Full(EnemyId::SlimeBoss), Some(EnemyMemory::SlimeBoss { split_triggered }))
                if !*split_triggered => {
                    *split_triggered = true;
                    (FullIntentId::from_name("split"), true)
                },
            _ => (FullIntentId::Attack, false),
        };
        if split_requested {
            let selected = ActorIntentId::Full(split_intent);
            enemy.intent_id = selected.clone();
            enemy.selected_move = selected.clone();
            enemy.move_history.push(selected);
            enemy.intent_damage = None;
        }
    }
    if !normal_damage || matches!(source, Source::None) || hp_damage <= 0 || hp_damage >= hp_before { return; }
    let Some(memory) = enemy.full_memory.as_mut() else { return; };
    let latched = match memory {
        EnemyMemory::LouseRed { curl_up_used } | EnemyMemory::LouseGreen { curl_up_used } => curl_up_used,
        _ => return,
    };
    if *latched { return; }
    let Some(curl) = enemy.full_powers.iter().find(|power| power.id == FullPowerId("curl_up")) else { return; };
    let block = curl.state.amount;
    *latched = true;
    let actor = Actor::Enemy(target.clone());
    let checkpoint = state.pending.iter().position(|work| matches!(work.kind, EffectKind::Decision))
        .unwrap_or(state.pending.len());
    state.pending.insert(checkpoint, Effect { source: Source::Actor(actor.clone()), target: Some(actor.clone()),
        kind: EffectKind::GainBlock(block) });
    state.pending.insert(checkpoint + 1, Effect { source: Source::Actor(actor.clone()), target: Some(actor),
        kind: EffectKind::RemoveFullPower(FullPowerId("curl_up")) });
}

fn hp_range(id: EnemyId, ascension: u8) -> (i32, i32) {
    match id {
        EnemyId::Cultist => if ascension >= 7 { (50, 56) } else { (48, 54) },
        EnemyId::JawWorm => if ascension >= 7 { (42, 46) } else { (40, 44) },
        EnemyId::GremlinNob => if ascension >= 8 { (85, 90) } else { (82, 86) },
        EnemyId::Lagavulin=>(109,111),EnemyId::Sentry=>(38,42),
        EnemyId::Guardian=>(240,240),
        EnemyId::Hexaghost=>(250,250),
        EnemyId::SphericGuardian=>(20,20),EnemyId::Byrd=>(25,31),
        EnemyId::Chosen=>(95,99),EnemyId::Snecko=>(114,120),
        EnemyId::Centurion=>(76,80),EnemyId::Mystic=>(48,56),
        EnemyId::LouseRed => (10, 15), EnemyId::LouseGreen => (11, 17),
        EnemyId::AcidSlimeS => (8, 12), EnemyId::AcidSlimeM => (28, 32),
        EnemyId::AcidSlimeL => (65, 69), EnemyId::SpikeSlimeS => (10, 14),
        EnemyId::SpikeSlimeM => (28, 32), EnemyId::SpikeSlimeL => (64, 70),
        EnemyId::SlimeBoss => (140, 140),
        EnemyId::Unknown(_) => (0, 0),
        EnemyId::TimeEater=>(456,456),
        EnemyId::Deca|EnemyId::Donu=>(250,250),
        EnemyId::AwakenedOne=>(300,300),
        EnemyId::Reptomancer=>(180,190),EnemyId::Dagger=>(20,25),
        EnemyId::GiantHead=>(500,500),EnemyId::Nemesis=>(185,185),
        EnemyId::WrithingMass=>(160,160),
        EnemyId::SpireGrowth=>(170,170),EnemyId::Transient=>(999,999),EnemyId::Maw=>(300,300),
        EnemyId::Darkling=>(48,56),
        EnemyId::OrbWalker=>(90,96),EnemyId::Repulsor=>(29,35),EnemyId::Exploder=>(30,30),EnemyId::Spiker=>(42,56),
        EnemyId::GremlinLeader=>(140,148),EnemyId::GremlinMad=>(20,24),EnemyId::GremlinSneaky=>(10,14),EnemyId::GremlinFat=>(13,17),EnemyId::GremlinShield=>(12,15),EnemyId::GremlinWizard=>(21,25),
        EnemyId::SlaverBlue|EnemyId::SlaverRed=>(46,50),EnemyId::Taskmaster=>(54,60),EnemyId::BookOfStabbing=>(160,164),
        EnemyId::Champ=>(420,420),EnemyId::CorruptHeart=>(750,750),
        EnemyId::Collector=>(282,282),EnemyId::TorchHead=>(38,40),
        EnemyId::BronzeAutomaton=>(300,300),EnemyId::BronzeOrb=>(52,58),
        EnemyId::SpireShield=>(110,110),EnemyId::SpireSpear=>(160,160),
    }
}

pub fn damage_bases(id: EnemyId) -> Result<std::collections::BTreeMap<FullIntentId, i32>, InputError> {
    damage_bases_at_ascension(id, 0)
}

pub fn damage_bases_at_ascension(id: EnemyId, ascension: u8)
    -> Result<std::collections::BTreeMap<FullIntentId, i32>, InputError> {
    if ascension > 20 {
        return Err(InputError::Unsupported("enemy ascension must be within 0..=20".into()));
    }
    if ascension != 0 && !matches!(id, EnemyId::Cultist | EnemyId::JawWorm | EnemyId::GremlinNob) {
        return Err(InputError::Unsupported("ascension-specific damage unavailable for this species".into()));
    }
    use FullIntentId::*;
    let entries: &[(FullIntentId, i32)] = match id {
        EnemyId::Cultist => &[(Attack, 6)],
        EnemyId::Lagavulin=>&[(Attack,18)],EnemyId::Sentry=>&[(FullIntentId::from_name("beam"),9)],
        EnemyId::Guardian=>&[(FullIntentId::from_name("fierce_bash"),32),(FullIntentId::from_name("whirlwind"),5),(FullIntentId::from_name("roll_attack"),9),(FullIntentId::from_name("twin_slam"),8)],
        EnemyId::Hexaghost=>&[(FullIntentId::from_name("divider"),1),(FullIntentId::from_name("sear"),6),(FullIntentId::from_name("tackle"),5),(FullIntentId::from_name("inferno"),2)],
        EnemyId::SphericGuardian=>&[(FullIntentId::from_name("sphere_debuff"),10),(FullIntentId::from_name("sphere_slam"),10),(FullIntentId::from_name("sphere_harden"),10)],
        EnemyId::Byrd=>&[(FullIntentId::from_name("byrd_peck"),1),(FullIntentId::from_name("byrd_swoop"),12),(FullIntentId::from_name("byrd_headbutt"),3)],
        EnemyId::Chosen=>&[(FullIntentId::from_name("chosen_poke"),5),(FullIntentId::from_name("chosen_debilitate"),10),(FullIntentId::from_name("chosen_zap"),18)],
        EnemyId::Snecko=>&[(FullIntentId::from_name("snecko_bite"),15),(FullIntentId::from_name("snecko_tail_whip"),8)],
        EnemyId::Centurion=>&[(FullIntentId::from_name("centurion_slash"),12),(FullIntentId::from_name("centurion_fury"),6)],
        EnemyId::Mystic=>&[(FullIntentId::from_name("mystic_attack"),8)],
        EnemyId::JawWorm => if ascension >= 2 { &[(Chomp, 12), (Thrash, 7)] }
            else { &[(Chomp, 11), (Thrash, 7)] },
        EnemyId::GremlinNob => &[
            (FullIntentId::from_name("skull_bash"), if ascension >= 3 { 8 } else { 6 }),
            (FullIntentId::from_name("bull_rush"), if ascension >= 3 { 16 } else { 14 }),
        ],
        EnemyId::LouseRed | EnemyId::LouseGreen => &[(Bite, 5)],
        EnemyId::AcidSlimeS => &[(AcidTackle, 3)],
        EnemyId::AcidSlimeM => &[(AcidSpit, 7), (AcidTackle, 10)],
        EnemyId::AcidSlimeL => &[(AcidSpit, 11), (AcidTackle, 16)],
        EnemyId::SpikeSlimeS => &[(SpikeTackle, 5)],
        EnemyId::SpikeSlimeM => &[(SpikeFlameTackle, 8)],
        EnemyId::SpikeSlimeL => &[(SpikeFlameTackle, 16)],
        EnemyId::SlimeBoss => &[(FullIntentId::from_name("slam"), 35)],
        EnemyId::Unknown(name) => return Err(InputError::Unsupported(format!("damage bases missing for {name}"))),
        EnemyId::TimeEater=>&[(FullIntentId::from_name("time_reverb"),7),(FullIntentId::from_name("time_slam"),26)],
        EnemyId::Deca=>&[(FullIntentId::from_name("deca_beam"),10)],
        EnemyId::Donu=>&[(FullIntentId::from_name("donu_beam"),10)],
        EnemyId::AwakenedOne=>&[(FullIntentId::from_name("awa_slash"),20),(FullIntentId::from_name("awa_soul"),6),(FullIntentId::from_name("awa_echo"),40),(FullIntentId::from_name("awa_sludge"),18),(FullIntentId::from_name("awa_tackle"),10)],
        EnemyId::Reptomancer=>&[(FullIntentId::from_name("repto_snake"),13),(FullIntentId::from_name("repto_bite"),30)],
        EnemyId::Dagger=>&[(FullIntentId::from_name("dagger_wound"),9),(FullIntentId::from_name("dagger_explode"),25)],
        EnemyId::GiantHead=>&[(FullIntentId::from_name("head_count"),13),(FullIntentId::from_name("head_time"),30)],
        EnemyId::Nemesis=>&[(FullIntentId::from_name("nemesis_tri"),6),(FullIntentId::from_name("nemesis_scythe"),45)],
        EnemyId::WrithingMass=>&[(FullIntentId::from_name("writhing_multi"),7),(FullIntentId::from_name("writhing_block"),15),(FullIntentId::from_name("writhing_debuff"),10),(FullIntentId::from_name("writhing_big"),32)],
        EnemyId::SpireGrowth=>&[(FullIntentId::from_name("growth_quick"),16),(FullIntentId::from_name("growth_smash"),22)],
        EnemyId::Transient=>&[(FullIntentId::from_name("transient_attack"),30)],
        EnemyId::Maw=>&[(FullIntentId::from_name("maw_slam"),25),(FullIntentId::from_name("maw_nom"),5)],
        EnemyId::Darkling=>&[(FullIntentId::from_name("darkling_nip"),7),(FullIntentId::from_name("darkling_chomp"),8)],
        EnemyId::OrbWalker=>&[(FullIntentId::from_name("orb_laser"),10),(FullIntentId::from_name("orb_claw"),15)],
        EnemyId::Repulsor=>&[(FullIntentId::from_name("repulsor_attack"),11)],EnemyId::Exploder=>&[(FullIntentId::from_name("exploder_attack"),9)],EnemyId::Spiker=>&[(FullIntentId::from_name("spiker_attack"),7)],
        EnemyId::GremlinLeader=>&[(FullIntentId::from_name("leader_stab"),6)],
        EnemyId::GremlinMad=>&[(FullIntentId::from_name("gremlin_scratch"),4)],EnemyId::GremlinSneaky=>&[(FullIntentId::from_name("gremlin_puncture"),9)],EnemyId::GremlinFat=>&[(FullIntentId::from_name("gremlin_smash"),4)],EnemyId::GremlinShield=>&[(FullIntentId::from_name("gremlin_bash"),6)],EnemyId::GremlinWizard=>&[(FullIntentId::from_name("gremlin_blast"),25)],
        EnemyId::SlaverBlue=>&[(FullIntentId::from_name("slaver_stab"),12),(FullIntentId::from_name("slaver_rake"),7)],EnemyId::Taskmaster=>&[(FullIntentId::from_name("taskmaster_whip"),7)],EnemyId::SlaverRed=>&[(FullIntentId::from_name("slaver_stab"),13),(FullIntentId::from_name("slaver_scrape"),8)],EnemyId::BookOfStabbing=>&[(FullIntentId::from_name("book_multi"),6),(FullIntentId::from_name("book_big"),21)],
        EnemyId::Champ=>&[(FullIntentId::from_name("champ_slash"),16),(FullIntentId::from_name("champ_execute"),10),(FullIntentId::from_name("champ_slap"),12)],
        EnemyId::Collector=>&[(FullIntentId::from_name("collector_rake"),18)],EnemyId::TorchHead=>&[(FullIntentId::from_name("torch_attack"),7)],
        EnemyId::BronzeAutomaton=>&[(FullIntentId::from_name("automaton_flail"),7),(FullIntentId::from_name("automaton_beam"),45)],EnemyId::BronzeOrb=>&[(FullIntentId::from_name("orb_beam"),8)],
        EnemyId::CorruptHeart=>&[(FullIntentId::from_name("heart_blood_shots"),2),(FullIntentId::from_name("heart_echo"),40)],
        EnemyId::SpireShield=>&[(FullIntentId::from_name("shield_bash"),12),(FullIntentId::from_name("shield_smash"),34)],
        EnemyId::SpireSpear=>&[(FullIntentId::from_name("spear_burn_strike"),5),(FullIntentId::from_name("spear_skewer"),10)],
    };
    Ok(entries.iter().copied().collect())
}

fn split_work(enemy: &EnemyState, child_id: EnemyId) -> Vec<Effect> {
    vec![Effect { source: Source::Actor(Actor::Enemy(enemy.instance_id.clone())), target: None,
        kind: EffectKind::ReplaceEnemy { parent: enemy.instance_id.clone(), children: vec![child_id, child_id] } }]
}

fn select_jaw(history: &[FullIntentId], memory: &EnemyMemory, stream: &StreamState)
    -> Result<MoveSelection, SelectError> {
    use FullIntentId::{Bellow, Chomp, Thrash};
    let horde_hard = matches!(memory, EnemyMemory::JawWorm {
        variant: JawWormVariant::HordeHard, ..
    });
    if history.iter().any(|m| !matches!(m, Chomp | Thrash | Bellow))
        || (!history.is_empty() && if horde_hard {
            !matches!(history[0], Chomp | Thrash | Bellow)
        } else { history[0] != Chomp }) {
        return Err(InputError::Unsupported("Jaw Worm history/identity mismatch".into()).into());
    }
    if history.is_empty() {
        if horde_hard {
            let (bucket, next) = rng::uniform_below(stream, 100)?;
            let intent = match bucket {
                0..=24 => Chomp,
                25..=54 => Thrash,
                55..=99 => Bellow,
                _ => unreachable!("bounded hard Jaw Worm opening bucket"),
            };
            return Ok(MoveSelection { intent, stream: next, memory: memory.clone() });
        }
        return Ok(MoveSelection { intent: Chomp, stream: stream.clone(), memory: memory.clone() });
    }
    let (roll, mut cursor) = rng::uniform_below(stream, 100)?;
    let last = history.last().copied();
    let last_two_thrash = history.ends_with(&[Thrash, Thrash]);
    let conditional = match roll {
        0..=24 if last == Some(Chomp) => Some((9_437_184, 16_777_216)),
        25..=54 if last_two_thrash => Some((5_989_466, 16_777_216)),
        55..=99 if last == Some(Bellow) => Some((6_979_322, 16_777_216)),
        _ => None,
    };
    let coin = if let Some((numerator, denominator)) = conditional {
        let (ticket, next) = rng::uniform_below(&cursor, denominator)?;
        cursor = next;
        Some(ticket < numerator)
    } else { None };
    let intent = match (roll, coin) {
        (0..=24, Some(true)) => Bellow,
        (0..=24, Some(false)) => Thrash,
        (0..=24, None) => Chomp,
        (25..=54, Some(true)) => Chomp,
        (25..=54, Some(false)) => Bellow,
        (25..=54, None) => Thrash,
        (55..=99, Some(true)) => Chomp,
        (55..=99, Some(false)) => Thrash,
        (55..=99, None) => Bellow,
        _ => unreachable!("bounded roll"),
    };
    Ok(MoveSelection { intent, stream: cursor, memory: memory.clone() })
}

/// Append native enemy payload to the same queue consumed by the shared resolver.
pub(super) fn act(state: &mut State, owner: &EnemyHandle) -> Result<(), super::EffectError> {
    if state.enemies.iter().any(|enemy|&enemy.instance_id==owner&&enemy.enemy_id==ActorEnemyId::Full(EnemyId::CorruptHeart)){
        let enemy=state.enemies.iter().find(|enemy|&enemy.instance_id==owner).expect("selected Heart owner");
        let work=act4::heart::move_work(enemy)?;
        for effect in work.into_iter().rev(){state.pending.push_front(effect);}
        super::effects::refresh_enemy_values(state)?;
        return Ok(())
    }
    if state.enemies.iter().any(|e|&e.instance_id==owner&&matches!(e.enemy_id,ActorEnemyId::Full(id) if act4::supports(id))){let work=act4::move_work(state,owner)?;for effect in work.into_iter().rev(){state.pending.push_front(effect);}super::effects::refresh_enemy_values(state)?;return Ok(())}
    if state.enemies.iter().any(|e|&e.instance_id==owner&&matches!(e.enemy_id,ActorEnemyId::Full(id) if act3::supports(id))){let work=act3::take_turn(state,owner)?;for effect in work.into_iter().rev(){state.pending.push_front(effect);}super::effects::refresh_enemy_values(state)?;return Ok(())}
    if state.enemies.iter().any(|e|&e.instance_id==owner&&matches!(e.enemy_id,ActorEnemyId::Full(EnemyId::BronzeAutomaton|EnemyId::BronzeOrb))){let work=act2::automaton::move_work(state,owner)?;for effect in work.into_iter().rev(){state.pending.push_front(effect);}super::effects::refresh_enemy_values(state)?;return Ok(())}
    if state.enemies.iter().any(|e|&e.instance_id==owner&&matches!(e.enemy_id,ActorEnemyId::Full(EnemyId::SlaverBlue|EnemyId::Taskmaster|EnemyId::SlaverRed|EnemyId::BookOfStabbing|EnemyId::Champ|EnemyId::Collector|EnemyId::TorchHead))){
        let work=act2::take_turn(state,owner)?;for effect in work.into_iter().rev(){state.pending.push_front(effect);}super::effects::refresh_enemy_values(state)?;return Ok(())
    }
    if state.enemies.iter().any(|e|&e.instance_id==owner&&matches!(e.enemy_id,ActorEnemyId::Full(EnemyId::Centurion|EnemyId::Mystic|EnemyId::GremlinLeader|EnemyId::GremlinMad|EnemyId::GremlinSneaky|EnemyId::GremlinFat|EnemyId::GremlinShield|EnemyId::GremlinWizard))){
        let work=act2::take_turn(state,owner)?;for effect in work.into_iter().rev(){state.pending.push_front(effect);}super::effects::refresh_enemy_values(state)?;return Ok(())
    }
    let player_hp=state.player.hp;
    let enemy = state.enemies.iter_mut().find(|enemy| &enemy.instance_id == owner)
        .ok_or_else(|| InputError::Invariant("enemy turn actor missing".into()))?;
    let headbutt=enemy.enemy_id==ActorEnemyId::Full(EnemyId::Byrd)&&enemy.intent_id.as_str()=="byrd_headbutt";
    let work = if matches!(enemy.enemy_id,ActorEnemyId::Full(EnemyId::Lagavulin)){elites::take_turn(enemy)?}
        else if enemy.enemy_id==ActorEnemyId::Full(EnemyId::Byrd){act2::flight::take_turn(enemy)?}
        else if enemy.enemy_id==ActorEnemyId::Full(EnemyId::Hexaghost){hexaghost::take_turn(enemy,player_hp)?}else{move_work(enemy)?};
    if headbutt {state.pending.retain(|work|!(work.kind==EffectKind::RollMove&&work.target==Some(Actor::Enemy(owner.clone()))));}
    for effect in work.into_iter().rev() { state.pending.push_front(effect); }
    super::effects::refresh_enemy_values(state)?;
    Ok(())
}

// Setup is intentionally an entry-context helper, not a fake player/deck
// constructor. Coordinator integration belongs to the run/combat bridge owner.
pub struct SetupContext {
    pub player: PlayerState,
    pub player_powers: Vec<super::FullPowerEntry>,
    pub character: Character,
    pub cards: Vec<super::LiveCard>,
    pub opening_draw: usize,
    pub encounter_visit: u64,
}

/// Build the supported native Act1 encounter from real run-owned player/deck
/// context. Species without an implemented ascension profile remain fail-closed.
/// Authoritative setup metadata shared by Run and independent combat entry.
/// Table order is the existing frozen encounter RNG scope key minus one.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum EncounterKind{Monster,Elite,Boss}
impl EncounterKind{pub fn label(self)->&'static str{match self{Self::Monster=>"Monster",Self::Elite=>"Elite",Self::Boss=>"Boss"}}}
pub const A0_ENCOUNTER_SETUPS:&[(crate::content::EncounterId,u8,EncounterKind)]=&[
    (crate::content::EncounterId::Cultist,1,EncounterKind::Monster), // RNG scope key 1
    (crate::content::EncounterId::JawWorm,1,EncounterKind::Monster), // RNG scope key 2
    (crate::content::EncounterId::TwoLouse,1,EncounterKind::Monster), // RNG scope key 3
    (crate::content::EncounterId::SmallSlimes,1,EncounterKind::Monster), // RNG scope key 4
    (crate::content::EncounterId::LargeSlime,1,EncounterKind::Monster), // RNG scope key 5
    (crate::content::EncounterId::LotsOfSlimes,1,EncounterKind::Monster), // RNG scope key 6
    (crate::content::EncounterId::SlimeBoss,1,EncounterKind::Boss), // RNG scope key 7
    (crate::content::EncounterId::ThreeCultists,2,EncounterKind::Monster), // RNG scope key 8
    (crate::content::EncounterId::JawWormHorde,3,EncounterKind::Monster), // RNG scope key 9
    (crate::content::EncounterId::GremlinNob,1,EncounterKind::Elite), // RNG scope key 10
    (crate::content::EncounterId::Lagavulin,1,EncounterKind::Elite), // RNG scope key 11
    (crate::content::EncounterId::ThreeSentries,1,EncounterKind::Elite), // RNG scope key 12
    (crate::content::EncounterId::ThreeLouse,1,EncounterKind::Monster), // RNG scope key 13
    (crate::content::EncounterId::Guardian,1,EncounterKind::Boss), // RNG scope key 14
    (crate::content::EncounterId::Hexaghost,1,EncounterKind::Boss), // RNG scope key 15
    (crate::content::EncounterId::SphericGuardian,2,EncounterKind::Monster), // RNG scope key 16
    (crate::content::EncounterId::ThreeByrds,2,EncounterKind::Monster), // RNG scope key 17
    (crate::content::EncounterId::ChosenAndByrds,2,EncounterKind::Monster), // RNG scope key 18
    (crate::content::EncounterId::Snecko,2,EncounterKind::Monster), // RNG scope key 19
    (crate::content::EncounterId::CenturionAndHealer,2,EncounterKind::Monster), // RNG scope key 20
    (crate::content::EncounterId::GremlinLeader,2,EncounterKind::Elite), // RNG scope key 21
    (crate::content::EncounterId::Slavers,2,EncounterKind::Elite), // RNG scope key 22
    (crate::content::EncounterId::BookOfStabbing,2,EncounterKind::Elite), // RNG scope key 23
    (crate::content::EncounterId::Champ,2,EncounterKind::Boss), // RNG scope key 24
    (crate::content::EncounterId::Collector,2,EncounterKind::Boss), // RNG scope key 25
    (crate::content::EncounterId::BronzeAutomaton,2,EncounterKind::Boss), // RNG scope key 26
    (crate::content::EncounterId::OrbWalker,3,EncounterKind::Monster), // RNG scope key 27
    (crate::content::EncounterId::ThreeShapes,3,EncounterKind::Monster), // RNG scope key 28
    (crate::content::EncounterId::FourShapes,3,EncounterKind::Monster), // RNG scope key 29
    (crate::content::EncounterId::SphereAndTwoShapes,3,EncounterKind::Monster), // RNG scope key 30
    (crate::content::EncounterId::ThreeDarklings,3,EncounterKind::Monster), // RNG scope key 31
    (crate::content::EncounterId::SpireGrowth,3,EncounterKind::Monster), // RNG scope key 32
    (crate::content::EncounterId::Transient,3,EncounterKind::Monster), // RNG scope key 33
    (crate::content::EncounterId::Maw,3,EncounterKind::Monster), // RNG scope key 34
    (crate::content::EncounterId::WrithingMass,3,EncounterKind::Monster), // RNG scope key 35
    (crate::content::EncounterId::GiantHead,3,EncounterKind::Elite), // RNG scope key 36
    (crate::content::EncounterId::Nemesis,3,EncounterKind::Elite), // RNG scope key 37
    (crate::content::EncounterId::Reptomancer,3,EncounterKind::Elite), // RNG scope key 38
    (crate::content::EncounterId::AwakenedOne,3,EncounterKind::Boss), // RNG scope key 39
    (crate::content::EncounterId::TimeEater,3,EncounterKind::Boss), // RNG scope key 40
    (crate::content::EncounterId::DonuAndDeca,3,EncounterKind::Boss), // RNG scope key 41
    (crate::content::EncounterId::ShieldAndSpear,4,EncounterKind::Elite), // RNG scope key 42
    (crate::content::EncounterId::CorruptHeart,4,EncounterKind::Boss), // RNG scope key 43
];

pub fn setup(encounter: crate::content::EncounterId, act: u8, ascension: u8,
    rng: &mut crate::run::RunRng, context: SetupContext)
    -> Result<super::CombatEntry, crate::run::RunError> {
    use crate::content::EncounterId;
    if ascension > 20 || context.cards.is_empty() || context.opening_draw > 10
        || context.player.hp <= 0 || context.player.hp > context.player.max_hp {
        return Err(crate::run::RunError::Unsupported("enemy setup facts unavailable for this act/ascension"));
    }
    let (index,(_,expected_act,_))=A0_ENCOUNTER_SETUPS.iter().enumerate().find(|(_, (id,_,_))|*id==encounter)
        .ok_or(crate::run::RunError::Registry("encounter setup not implemented"))?;
    let encounter_key=index as u64+1;
    let expected_act=*expected_act;
    if act != expected_act {
        return Err(crate::run::RunError::Unsupported("encounter is unavailable in the supplied act"));
    }
    if ascension != 0 && !matches!(encounter,
        EncounterId::Cultist | EncounterId::JawWorm | EncounterId::ThreeCultists
            | EncounterId::JawWormHorde | EncounterId::GremlinNob) {
        return Err(crate::run::RunError::Unsupported("enemy setup facts unavailable for this act/ascension"));
    }
    let visit = context.encounter_visit;
    let mut enemies = Vec::new();
    let mut damage_bases = std::collections::BTreeMap::new();
    let ids = match encounter {
        EncounterId::ShieldAndSpear=>vec![EnemyId::SpireShield,EnemyId::SpireSpear],
        EncounterId::Cultist => vec![EnemyId::Cultist],
        EncounterId::JawWorm => vec![EnemyId::JawWorm],
        EncounterId::ThreeCultists => vec![EnemyId::Cultist; 3],
        EncounterId::JawWormHorde => vec![EnemyId::JawWorm; 3],
        EncounterId::GremlinNob => vec![EnemyId::GremlinNob],
        EncounterId::Lagavulin=>vec![EnemyId::Lagavulin],EncounterId::ThreeSentries=>vec![EnemyId::Sentry;3],
        EncounterId::Guardian=>vec![EnemyId::Guardian],
        EncounterId::Hexaghost=>vec![EnemyId::Hexaghost],
        EncounterId::SphericGuardian=>vec![EnemyId::SphericGuardian],EncounterId::ThreeByrds=>vec![EnemyId::Byrd;3],
        EncounterId::ChosenAndByrds=>vec![EnemyId::Byrd,EnemyId::Chosen],EncounterId::Snecko=>vec![EnemyId::Snecko],
        EncounterId::CenturionAndHealer=>vec![EnemyId::Centurion,EnemyId::Mystic],
        EncounterId::Slavers=>vec![EnemyId::SlaverBlue,EnemyId::Taskmaster,EnemyId::SlaverRed],EncounterId::BookOfStabbing=>vec![EnemyId::BookOfStabbing],
        EncounterId::Champ=>vec![EnemyId::Champ],
        EncounterId::Collector=>vec![EnemyId::Collector],
        EncounterId::BronzeAutomaton=>vec![EnemyId::BronzeAutomaton],
        EncounterId::OrbWalker=>vec![EnemyId::OrbWalker],
        EncounterId::ThreeDarklings=>vec![EnemyId::Darkling;3],
        EncounterId::SpireGrowth=>vec![EnemyId::SpireGrowth],EncounterId::Transient=>vec![EnemyId::Transient],EncounterId::Maw=>vec![EnemyId::Maw],
        EncounterId::WrithingMass=>vec![EnemyId::WrithingMass],
        EncounterId::GiantHead=>vec![EnemyId::GiantHead],
        EncounterId::Nemesis=>vec![EnemyId::Nemesis],
        EncounterId::Reptomancer=>vec![EnemyId::Dagger,EnemyId::Reptomancer,EnemyId::Dagger],
        EncounterId::AwakenedOne=>vec![EnemyId::Cultist,EnemyId::Cultist,EnemyId::AwakenedOne],
        EncounterId::TimeEater=>vec![EnemyId::TimeEater],
        EncounterId::DonuAndDeca=>vec![EnemyId::Deca,EnemyId::Donu],
        EncounterId::CorruptHeart=>vec![EnemyId::CorruptHeart],
        EncounterId::ThreeShapes=>vec![EnemyId::Repulsor;3],EncounterId::FourShapes=>vec![EnemyId::Repulsor;4],
        EncounterId::SphereAndTwoShapes=>vec![EnemyId::Repulsor,EnemyId::Repulsor,EnemyId::SphericGuardian],
        EncounterId::GremlinLeader=>{
            let mut ids=Vec::new();for slot in 0..2{let ticket=rng.bounded_scoped(crate::run::RunRngScope::CombatEnemySetup,&[act as u64,encounter_key,visit,slot,0],8);ids.push(act2::gremlin_leader::POOL[ticket]);}ids.push(EnemyId::GremlinLeader);ids
        },
        EncounterId::TwoLouse|EncounterId::ThreeLouse => (0..if encounter==EncounterId::ThreeLouse{3}else{2}).map(|slot| {
            if rng.bounded_scoped(crate::run::RunRngScope::CombatEnemySetup,
                &[act as u64, encounter_key, visit, slot, 0], 2) == 0 { EnemyId::LouseRed }
            else { EnemyId::LouseGreen }
        }).collect(),
        EncounterId::SmallSlimes => if rng.bounded_scoped(crate::run::RunRngScope::CombatEnemySetup,
            &[act as u64, encounter_key, visit, 0, 0], 2) == 0 {
                vec![EnemyId::SpikeSlimeS, EnemyId::AcidSlimeM]
            } else { vec![EnemyId::AcidSlimeS, EnemyId::SpikeSlimeM] },
        EncounterId::LargeSlime => vec![if rng.bounded_scoped(crate::run::RunRngScope::CombatEnemySetup,
            &[act as u64, encounter_key, visit, 0, 0], 2) == 0 { EnemyId::AcidSlimeL } else { EnemyId::SpikeSlimeL }],
        EncounterId::LotsOfSlimes => {
            let mut pool = vec![EnemyId::SpikeSlimeS, EnemyId::SpikeSlimeS, EnemyId::SpikeSlimeS,
                EnemyId::AcidSlimeS, EnemyId::AcidSlimeS];
            let mut ordered = Vec::with_capacity(5);
            for draw in 0..5 {
                let index = rng.bounded_scoped(crate::run::RunRngScope::CombatEnemySetup,
                    &[act as u64, encounter_key, visit, draw, 0], pool.len() as u64);
                ordered.push(pool.remove(index));
            }
            ordered
        }
        EncounterId::SlimeBoss => vec![EnemyId::SlimeBoss],
        _ => unreachable!("encounter selected above"),
    };
    let mut shape_pool=vec![EnemyId::Repulsor,EnemyId::Repulsor,EnemyId::Exploder,EnemyId::Exploder,EnemyId::Spiker,EnemyId::Spiker];
    for (slot, mut id) in ids.into_iter().enumerate() {
        // Each chosen slot is constructed before the next composition request.
        if matches!(encounter,EncounterId::ThreeShapes|EncounterId::FourShapes){let index=rng.bounded_scoped(crate::run::RunRngScope::CombatEnemySetup,&[3,encounter_key,visit,slot as u64,0],shape_pool.len() as u64);id=shape_pool.remove(index);}
        else if encounter==EncounterId::SphereAndTwoShapes&&slot<2{let index=rng.bounded_scoped(crate::run::RunRngScope::CombatEnemySetup,&[3,encounter_key,visit,slot as u64,0],3);id=[EnemyId::Spiker,EnemyId::Repulsor,EnemyId::Exploder][index];}
        let louse = matches!(id, EnemyId::LouseRed | EnemyId::LouseGreen);
        let (min_hp, max_hp) = hp_range(id, ascension);
        let hp = if min_hp == max_hp { min_hp } else { min_hp + rng.bounded_scoped(
            crate::run::RunRngScope::CombatEnemySetup,
            &[act as u64, encounter_key, visit, slot as u64, 1], (max_hp - min_hp + 1) as u64) as i32 };
        if louse {
            let bite = 5 + rng.bounded_scoped(crate::run::RunRngScope::CombatEnemySetup,
                &[act as u64, encounter_key, visit, slot as u64, 3], 3) as i32;
            damage_bases.insert((slot as u32, FullIntentId::Bite), bite);
        }
        let mut powers = Vec::new();
        if matches!(id,EnemyId::Deca|EnemyId::Donu){powers.push(super::FullPowerEntry{id:FullPowerId("artifact"),state:super::PowerState{amount:2,skip_first:false,monster_source_grace:false}});}
        if encounter==EncounterId::ShieldAndSpear{powers.push(super::FullPowerEntry{id:FullPowerId("artifact"),state:super::PowerState{amount:1,skip_first:false,monster_source_grace:false}});powers.push(super::FullPowerEntry{id:FullPowerId("back_attack"),state:super::PowerState{amount:1,skip_first:false,monster_source_grace:false}});}
        if id==EnemyId::TimeEater{powers.push(super::FullPowerEntry{id:FullPowerId("time_warp"),state:super::PowerState{amount:0,skip_first:false,monster_source_grace:false}});}
        if id==EnemyId::AwakenedOne{for (name,amount) in [("curiosity",1),("regenerate",10),("unawakened",1)]{powers.push(super::FullPowerEntry{id:FullPowerId(name),state:super::PowerState{amount,skip_first:false,monster_source_grace:false}});}}
        if encounter==EncounterId::Reptomancer&&id!=EnemyId::Reptomancer{powers.push(super::FullPowerEntry{id:FullPowerId("minion"),state:super::PowerState{amount:1,skip_first:false,monster_source_grace:false}});}
        if id==EnemyId::GiantHead{powers.push(super::FullPowerEntry{id:FullPowerId("slow"),state:super::PowerState{amount:0,skip_first:false,monster_source_grace:false}});}
        if id==EnemyId::WrithingMass{for (name,amount) in [("reactive",1),("malleable",4)]{powers.push(super::FullPowerEntry{id:FullPowerId(name),state:super::PowerState{amount,skip_first:false,monster_source_grace:false}});}}
        if id==EnemyId::Transient{for (name,amount) in [("fading",5),("shifting",1)]{powers.push(super::FullPowerEntry{id:FullPowerId(name),state:super::PowerState{amount,skip_first:false,monster_source_grace:false}});}}
        for (species,name,amount) in [(EnemyId::OrbWalker,"strength_up",3),(EnemyId::Spiker,"thorns",3),(EnemyId::Exploder,"explosive",3)]{if id==species{powers.push(super::FullPowerEntry{id:FullPowerId(name),state:super::PowerState{amount,skip_first:false,monster_source_grace:false}});}}
        if id==EnemyId::BronzeAutomaton{powers.push(super::FullPowerEntry{id:FullPowerId("artifact"),state:super::PowerState{amount:3,skip_first:false,monster_source_grace:false}});}
        if id==EnemyId::CorruptHeart {for (name,amount) in [("beat_of_death",1),("invincible",300)] {powers.push(super::FullPowerEntry{id:FullPowerId(name),state:super::PowerState{amount,skip_first:false,monster_source_grace:false}});}}
        if id==EnemyId::BookOfStabbing{powers.push(super::FullPowerEntry{id:FullPowerId("painful_stabs"),state:super::PowerState{amount:1,skip_first:false,monster_source_grace:false}});}
        if encounter==EncounterId::GremlinLeader&&id!=EnemyId::GremlinLeader{powers.push(super::FullPowerEntry{id:FullPowerId("minion"),state:super::PowerState{amount:1,skip_first:false,monster_source_grace:false}});}
        if id==EnemyId::GremlinMad{powers.push(super::FullPowerEntry{id:FullPowerId("angry"),state:super::PowerState{amount:1,skip_first:false,monster_source_grace:false}});}
        if id==EnemyId::SphericGuardian {for (name,amount) in [("artifact",3),("barricade",1)] {powers.push(super::FullPowerEntry{id:FullPowerId(name),state:super::PowerState{amount,skip_first:false,monster_source_grace:false}});}}
        if id==EnemyId::Byrd {powers.push(super::FullPowerEntry{id:FullPowerId("flight"),state:super::PowerState{amount:3,skip_first:false,monster_source_grace:false}});}
        if id==EnemyId::Guardian {powers.push(super::FullPowerEntry{id:FullPowerId("mode_shift"),state:super::PowerState{amount:30,skip_first:false,monster_source_grace:false}});}
        if id==EnemyId::Sentry {powers.push(super::FullPowerEntry{id:FullPowerId("artifact"),state:super::PowerState{amount:1,skip_first:false,monster_source_grace:false}});}
        if louse {
            let curl_up = 3 + rng.bounded_scoped(crate::run::RunRngScope::CombatEnemySetup,
                &[act as u64, encounter_key, visit, slot as u64, 2], 5) as i32;
            powers.push(super::FullPowerEntry { id: FullPowerId("curl_up"),
                state: super::PowerState { amount: curl_up, skip_first: false, monster_source_grace: false } });
        }
        if matches!(id, EnemyId::AcidSlimeL | EnemyId::SpikeSlimeL | EnemyId::SlimeBoss) {
            powers.push(super::FullPowerEntry { id: FullPowerId("split"),
                state: super::PowerState { amount: 1, skip_first: false, monster_source_grace: false } });
        }
        let actor = format!("{}-{visit}-{slot}", id.as_str());
        let ai_rng = rng.derive_combat_stream(crate::combat::rng::Domain::RunCombatEnemyAi,
            act, encounter.as_str(), visit, Some(slot as u32), &actor)
            .map_err(|_| crate::run::RunError::Unsupported("enemy AI stream derivation failed"))?;
        let profile = if encounter == EncounterId::JawWormHorde {
            EnemyProfile::JawWormHorde
        } else if encounter==EncounterId::ThreeSentries&&slot==1{EnemyProfile::SentryCenter
        } else { EnemyProfile::Standard };
        let mut memory = EnemyMemory::for_profile(id, ascension, profile)
            .map_err(|_| crate::run::RunError::Unsupported("enemy memory unavailable"))?;
        if id==EnemyId::Darkling{let nip=7+rng.bounded_scoped(crate::run::RunRngScope::CombatEnemySetup,&[act as u64,encounter_key,visit,slot as u64,3],5) as i32;damage_bases.insert((slot as u32,FullIntentId::from_name("darkling_nip")),nip);memory=EnemyMemory::Darkling{center:slot==1,nip,first:true};powers.push(super::FullPowerEntry{id:FullPowerId("life_link"),state:super::PowerState{amount:1,skip_first:false,monster_source_grace:false}});}
        enemies.push(FullEnemySetup { initial_memory:Some(memory),instance_id: EnemyHandle::parse(&actor)
            .map_err(|_| crate::run::RunError::Unsupported("invalid generated enemy identity"))?,
            enemy_id: id, hp, max_hp: hp, block: 0, intent: FullIntentId::from_name("unselected_native_entry"), ascension, profile,
            powers, ai_rng });
    }
    let player_constricted=context.player_powers.iter().any(|p|p.id.0=="constricted"&&p.state.amount>0);
    if encounter==EncounterId::Reptomancer{let slots=[(1,enemies[0].instance_id.clone()),(0,enemies[2].instance_id.clone())].into_iter().collect();enemies[1].initial_memory=Some(EnemyMemory::Reptomancer{first:true,slots});}
    let summaries=enemies.iter().enumerate().map(|(slot,a)|act2::ActorSummary{owner:a.instance_id.clone(),species:a.enemy_id,hp:a.hp,max_hp:a.max_hp,slot:slot as u8}).collect::<Vec<_>>();
    for actor in &mut enemies {
        let context=act2::EnemyRollContext{owner:actor.instance_id.clone(),turn:1,actors:summaries.clone()};
        if actor.enemy_id==EnemyId::CorruptHeart {
            actor.intent=FullIntentId::HeartDebilitate;
        }else{
            let selected=if actor.enemy_id==EnemyId::SpireGrowth{act3::ordinary::select_move(actor.enemy_id,&[],actor.initial_memory.as_ref().unwrap(),&actor.ai_rng,Some(player_constricted))}else{act2::select_move(actor.enemy_id,&[],actor.initial_memory.as_ref().unwrap(),&actor.ai_rng,&context)}.map_err(|_|crate::run::RunError::Unsupported("initial enemy move selection failed"))?;
            actor.intent=selected.intent;actor.ai_rng=selected.stream;actor.initial_memory=Some(selected.memory);
        }
    }
    let shuffle_rng = rng.derive_combat_stream(crate::combat::rng::Domain::RunCombatShuffle,
        act, encounter.as_str(), visit, None, "shuffle")
        .map_err(|_| crate::run::RunError::Unsupported("combat shuffle stream derivation failed"))?;
    let card_rng = rng.derive_combat_stream(crate::combat::rng::Domain::RunCombatCardRandom,
        act, encounter.as_str(), visit, None, "card")
        .map_err(|_| crate::run::RunError::Unsupported("combat card-random stream derivation failed"))?;
    let mut player_powers=context.player_powers;
    if encounter==EncounterId::ShieldAndSpear{player_powers.push(super::FullPowerEntry{id:FullPowerId("surrounded"),state:super::PowerState{amount:1,skip_first:false,monster_source_grace:false}});}
    let setup = FullCombatSetup { player: context.player, player_powers,
        character: context.character, card_rng,
        shuffle_rng, enemies, enemy_damage_bases: damage_bases };
    Ok(super::CombatEntry { setup, opening_draw: context.opening_draw, cards: context.cards })
}
