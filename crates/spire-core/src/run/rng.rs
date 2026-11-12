use std::collections::BTreeMap;
use crate::combat::rng::{hmac_sha256, make_stream, Domain, Partition, RngError, StreamId, StreamState};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RunRngScope { NeowCategory0, NeowCategory1, NeowDrawback, NeowCategory2,
     MapTopology, MapRoomBucket, MapBurningElite, MapEncounter, MapQuestionRoom, CombatEnemySetup, CombatGoldReward, Act1BossOrder, BurningEliteBuff,Act2BossOrder,Act2Joust,Act3BossOrder,Act3Falling,M10CardOffer,M15PotionGeneration,M15CardOffer,M20CardOffer,M22RelicCapture,M23RelicCapture,M24RelicCapture,M25CardOffer,M25RelicCapture,M26CardOffer,M26CardTransform,M26CardCurse,M27RelicPools,M27RelicTier,M27PotionOffer,M28EventBranch,M28EventPick,M28EventBody,M28CardRarity,M28ShopStock,M28ShopPrice,M28Chest }
impl RunRngScope {
    fn as_str(self)->&'static str { match self {
        Self::Act3Falling=>"run.act3.falling",
        Self::M10CardOffer=>"run.m10.card.offer",
        Self::M15PotionGeneration=>"run.m15.potion.generate",
        Self::M15CardOffer=>"run.m15.card.offer",
        Self::M20CardOffer=>"run.m20.card.offer",
        Self::M22RelicCapture=>super::rooms::m22_content::RELIC_CAPTURE_SCOPE_NAME,
        Self::M23RelicCapture=>super::rooms::m23_content::ACT2_RELIC_CAPTURE_SCOPE_NAME,
        Self::M24RelicCapture=>super::rooms::m24_content::ACT3_RELIC_CAPTURE_SCOPE_NAME,
        Self::M25CardOffer=>super::rooms::m25_content::CARD_OFFER_SCOPE_NAME,
        Self::M25RelicCapture=>super::rooms::m25_content::ACT4_RELIC_CAPTURE_SCOPE_NAME,
        Self::M26CardOffer=>"run.m26.card.offer",
        Self::M26CardTransform=>"run.m26.card.transform",
        Self::M26CardCurse=>"run.m26.card.curse",
        Self::M27RelicPools=>"run.m27.relic.pools",
        Self::M27RelicTier=>"run.m27.relic.tier",
        Self::M27PotionOffer=>"run.m27.potion.offer",
        Self::M28EventBranch=>"run.m28.event.branch",
        Self::M28EventPick=>"run.m28.event.pick",
        Self::M28EventBody=>"run.m28.event.body",
        Self::M28CardRarity=>"run.m28.card.rarity",
        Self::M28ShopStock=>"run.m28.shop.stock",
        Self::M28ShopPrice=>"run.m28.shop.price",
        Self::M28Chest=>"run.m28.chest",
        Self::NeowCategory0=>"run.neow.category0", Self::NeowCategory1=>"run.neow.category1",
        Self::NeowDrawback=>"run.neow.drawback", Self::NeowCategory2=>"run.neow.category2",
        Self::MapTopology=>"run.map.topology", Self::MapRoomBucket=>"run.map.room_bucket",
        Self::MapBurningElite=>"run.map.burning_elite", Self::MapEncounter=>"run.map.encounter",
        Self::MapQuestionRoom=>"run.map.question_room", Self::CombatEnemySetup=>"run.combat.enemy.setup",
        Self::CombatGoldReward=>"run.reward.gold",
        Self::Act1BossOrder=>"run.act1.boss_order",
        Self::BurningEliteBuff=>"run.combat.burning_elite.buff",
        Self::Act2BossOrder=>"run.act2.boss_order",
        Self::Act2Joust=>"run.act2.joust",
        Self::Act3BossOrder=>"run.act3.boss_order",
    } }
    pub fn parse(value:&str)->Result<Self,RunRngScopeError> { match value {
        "run.act3.falling"=>Ok(Self::Act3Falling),
        "run.m10.card.offer"=>Ok(Self::M10CardOffer),
        "run.m15.potion.generate"=>Ok(Self::M15PotionGeneration),
        "run.m15.card.offer"=>Ok(Self::M15CardOffer),
        "run.m20.card.offer"=>Ok(Self::M20CardOffer),
        super::rooms::m22_content::RELIC_CAPTURE_SCOPE_NAME=>Ok(Self::M22RelicCapture),
        super::rooms::m23_content::ACT2_RELIC_CAPTURE_SCOPE_NAME=>Ok(Self::M23RelicCapture),
        super::rooms::m24_content::ACT3_RELIC_CAPTURE_SCOPE_NAME=>Ok(Self::M24RelicCapture),
        super::rooms::m25_content::CARD_OFFER_SCOPE_NAME=>Ok(Self::M25CardOffer),
        super::rooms::m25_content::ACT4_RELIC_CAPTURE_SCOPE_NAME=>Ok(Self::M25RelicCapture),
        "run.m26.card.offer"=>Ok(Self::M26CardOffer),
        "run.m26.card.transform"=>Ok(Self::M26CardTransform),
        "run.m26.card.curse"=>Ok(Self::M26CardCurse),
        "run.m27.relic.pools"=>Ok(Self::M27RelicPools),
        "run.m27.relic.tier"=>Ok(Self::M27RelicTier),
        "run.m27.potion.offer"=>Ok(Self::M27PotionOffer),
        "run.m28.event.branch"=>Ok(Self::M28EventBranch),
        "run.m28.event.pick"=>Ok(Self::M28EventPick),
        "run.m28.event.body"=>Ok(Self::M28EventBody),
        "run.m28.card.rarity"=>Ok(Self::M28CardRarity),
        "run.m28.shop.stock"=>Ok(Self::M28ShopStock),
        "run.m28.shop.price"=>Ok(Self::M28ShopPrice),
        "run.m28.chest"=>Ok(Self::M28Chest),
        "run.neow.category0"=>Ok(Self::NeowCategory0), "run.neow.category1"=>Ok(Self::NeowCategory1),
        "run.neow.drawback"=>Ok(Self::NeowDrawback), "run.neow.category2"=>Ok(Self::NeowCategory2),
        "run.map.topology"=>Ok(Self::MapTopology), "run.map.room_bucket"=>Ok(Self::MapRoomBucket),
        "run.map.burning_elite"=>Ok(Self::MapBurningElite), "run.map.encounter"=>Ok(Self::MapEncounter),
        "run.map.question_room"=>Ok(Self::MapQuestionRoom), "run.combat.enemy.setup"=>Ok(Self::CombatEnemySetup),
        "run.reward.gold"=>Ok(Self::CombatGoldReward),
        "run.act1.boss_order"=>Ok(Self::Act1BossOrder),
        "run.combat.burning_elite.buff"=>Ok(Self::BurningEliteBuff),
        "run.act2.boss_order"=>Ok(Self::Act2BossOrder),
        "run.act2.joust"=>Ok(Self::Act2Joust),
        "run.act3.boss_order"=>Ok(Self::Act3BossOrder),
        _=>Err(RunRngScopeError::Unregistered(value.to_owned())),
    }}
}

#[cfg(test)]
mod m15_content_rng_tests {
    use super::*;

    #[test]
    fn m15_content_checked_rng_exhaustion_is_atomic() {
        let key = vec![17, 1];
        let mut rng = RunRng::new([27; 32], Partition::Test, 3);
        rng.counters.insert((RunRngScope::M15CardOffer, key.clone()), u64::MAX);
        let before = rng.clone();
        let error = crate::run::rooms::m15_content::sample_capture_row(
            &mut rng,
            &[17],
            &[crate::content::CardRarity::Common, crate::content::CardRarity::Common],
        )
        .unwrap_err();
        assert_eq!(error, super::super::RunError::Unsupported("run scoped RNG exhausted"));
        assert_eq!(rng, before, "a later-slot failure rolls back earlier detached draws");
    }
}

#[cfg(test)]
mod m8_falling_tests{
    use super::*;
    use crate::{content::{CardId,Character,EventId},run::*};
    #[test]
    fn m8_falling_scope_frontier_is_checked_before_event_publication(){
        assert_eq!(RunRngScope::parse("run.act3.falling"),Ok(RunRngScope::Act3Falling));
        let mut profile=CreationProfile::experienced();profile.neow_eligible=false;
        let mut run=Run::new_room_scenario(RunConfig{character:Character::Ironclad,ascension:0},profile,RunRng::new([7;32],Partition::Test,0),RoomScenario::M4Act1Prefix).unwrap();
        let id=run.decision().unwrap().room.unwrap().choices.iter().find(|c|matches!(c.kind,RoomChoiceKind::NeowMaxHp{..})).unwrap().id;run.step(&GameAction::Choose(id)).unwrap();run.state.act=3;run.state.map.as_mut().unwrap().act=3;run.state.deck=vec![CardInstance{ misc:0,id:CardId::Strike,instance:1,upgrades:0}];
        let semantic=vec![3,10,0,0];run.state.rng.counters.insert((RunRngScope::Act3Falling,semantic.clone()),u64::MAX);let frozen=run.clone();let mut next=run.next_choice;let token=next;
        assert!(crate::run::rooms::events::act3::prepare(&run,EventId::Falling,NodeId(10),0,&mut next).is_err());assert!(run==frozen);assert_eq!(next,token);
        run.state.rng.counters.insert((RunRngScope::Act3Falling,semantic.clone()),u64::MAX-1);let prepared=crate::run::rooms::events::act3::prepare(&run,EventId::Falling,NodeId(10),0,&mut next).unwrap();assert_eq!(prepared.rng.counters.get(&(RunRngScope::Act3Falling,semantic)),Some(&u64::MAX));
    }
}
#[derive(Clone, Debug, PartialEq, Eq)] pub enum RunRngScopeError { Unregistered(String) }

/// Root-keyed HMAC domain labels of the Run streams (one Run root key, disjoint labels).
const RUN_RNG_LABEL:&[u8]=b"spire-lab-run-rng-v1\0";
const RUN_COMBAT_STREAM_LABEL:&str="spire-lab/run-combat-stream/v1/";
/// M34c reservation (design §10): the root-keyed HMAC domain label for later research draws (M34s public-information
/// resampling). No gameplay stream derives from it and M34 adds no draw API, so research draws can never move a
/// gameplay stream.
pub const RESERVED_RESEARCH_RNG_DOMAIN:&str="spire-lab-research-rng-v1";
/// M34s-b research labels (design §4.6.4), all under the reserved domain: sample `k` of a public root uses
/// `HMAC(research_root, RESEARCH_SAMPLE_LABEL || root_digest || k_be)`; its R1/R2/R3 sub-keys are HMACs of that key.
pub const RESEARCH_SAMPLE_LABEL:&[u8]=b"spire-lab-research-rng-v1\0belief-v1\0sample\0";
/// The 32-byte research root of a caller `research_seed` (adapter `spire-gym-run-u64-le-v1`, as gameplay seeds).
pub fn research_root(research_seed:u64)->[u8;32]{let mut root=[0;32];root[..8].copy_from_slice(&research_seed.to_le_bytes());root}
/// M34s-c (design §4.8): the research label of a `reachable_recipe` start, under the reserved domain.
pub const RESEARCH_START_LABEL:&[u8]=b"spire-lab-research-rng-v1\0belief-v1\0start\0";
/// The sample key of a recipe reset: keyed by the reset's seed (adapter `spire-gym-run-u64-le-v1`), partition and episode
/// ordinal, and the recipe's public root digest (Gym66 seed semantics; a reset never reads the record's hidden state).
pub fn research_start_key(seed:u64,partition:crate::combat::rng::Partition,ordinal:u64,root_digest:&[u8;32])->[u8;32]{
    let mut message=RESEARCH_START_LABEL.to_vec();message.extend_from_slice(partition.as_str().as_bytes());message.push(0);
    message.extend_from_slice(&ordinal.to_be_bytes());message.extend_from_slice(root_digest);
    crate::combat::rng::hmac_sha256(&research_root(seed),&message)
}
/// The key of sample `k` of the public root with digest `root_digest`.
pub fn research_sample_key(research_root:&[u8;32],root_digest:&[u8;32],k:u64)->[u8;32]{
    let mut message=RESEARCH_SAMPLE_LABEL.to_vec();message.extend_from_slice(root_digest);message.extend_from_slice(&k.to_be_bytes());
    crate::combat::rng::hmac_sha256(research_root,&message)
}
#[cfg(test)]
mod m34c_reserved_domain{
    #[test]
    fn reserved_research_domain_is_disjoint_from_gameplay_labels(){
        let reserved=format!("{}\0",super::RESERVED_RESEARCH_RNG_DOMAIN);
        // Every label keyed directly by a Run/combat root: Run scopes, Run combat streams, combat streams (combat/rng.rs).
        let labels=[std::str::from_utf8(super::RUN_RNG_LABEL).unwrap(),super::RUN_COMBAT_STREAM_LABEL,"spire-lab/rng/v1/stream\0"];
        for label in labels{assert!(!label.starts_with(&reserved)&&!reserved.starts_with(label),"{label} overlaps the reserved research domain");}
    }
    /// M34s-b: the research sample label lies inside the reserved domain, so a research seed equal to a gameplay seed
    /// still keys disjoint HMAC messages (`research_label_is_disjoint`).
    #[test]
    fn research_label_is_disjoint(){
        let reserved=format!("{}\0",super::RESERVED_RESEARCH_RNG_DOMAIN);
        assert!(super::RESEARCH_SAMPLE_LABEL.starts_with(reserved.as_bytes())&&super::RESEARCH_START_LABEL.starts_with(reserved.as_bytes()));
        assert!(!super::RESEARCH_START_LABEL.starts_with(super::RESEARCH_SAMPLE_LABEL)&&!super::RESEARCH_SAMPLE_LABEL.starts_with(super::RESEARCH_START_LABEL));
        let start=|seed,part,ordinal|super::research_start_key(seed,part,ordinal,&[3;32]);
        let (t,v)=(crate::combat::rng::Partition::Train,crate::combat::rng::Partition::Test);
        assert!(start(7,t,0)!=start(8,t,0)&&start(7,t,0)!=start(7,v,0)&&start(7,t,0)!=start(7,t,1)&&start(7,t,0)!=super::research_start_key(7,t,0,&[4;32]));
        for label in [super::RUN_RNG_LABEL,super::RUN_COMBAT_STREAM_LABEL.as_bytes(),b"spire-lab/rng/v1/stream\0"]{
            assert!(!super::RESEARCH_SAMPLE_LABEL.starts_with(label)&&!label.starts_with(super::RESEARCH_SAMPLE_LABEL));
        }
        let (root,digest)=(super::research_root(7),[3;32]);
        assert_eq!(root,{let mut r=[0;32];r[0]=7;r});
        assert_ne!(super::research_sample_key(&root,&digest,0),super::research_sample_key(&root,&digest,1));
        assert_ne!(super::research_sample_key(&root,&digest,0),super::research_sample_key(&root,&[4;32],0));
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunRng { root:[u8;32], partition:Partition, ordinal:u64, counters:BTreeMap<(RunRngScope,Vec<u64>),u64> }
impl RunRng {
    pub fn new(root:[u8;32], partition:Partition, ordinal:u64)->Self { Self{root,partition,ordinal,counters:BTreeMap::new()} }
    #[cfg(test)] pub(crate) fn root_for_test(&self)->[u8;32]{self.root}
    pub fn draw(&mut self, scope:RunRngScope)->u64 { self.draw_scoped(scope,&[]) }
    pub fn draw_scoped(&mut self, scope:RunRngScope, semantic:&[u64])->u64 {
        let counter=self.counters.entry((scope,semantic.to_vec())).or_insert(0);
        let current=*counter; *counter=counter.saturating_add(1);
        // M38b: the HMAC reads the message in parts, in this order (the bytes `draw_message` builds; a test compares them).
        let (partition,ordinal)=(self.partition,self.ordinal);
        let digest=crate::combat::rng::hmac_sha256_parts(&self.root,|part|{
            part(RUN_RNG_LABEL); part(partition.as_str().as_bytes()); part(&[0]);
            part(&ordinal.to_be_bytes()); part(&[0]);
            part(scope.as_str().as_bytes()); part(&[0]);
            if !semantic.is_empty() {
                part(b"semantic-v1\0");
                part(&(semantic.len() as u64).to_be_bytes());
                for value in semantic { part(&value.to_be_bytes()); }
            }
            part(&current.to_be_bytes());
        });
        u64::from_be_bytes(digest[..8].try_into().unwrap())
    }
    /// The pre-M38b message of a draw (test reference for `draw_scoped`).
    #[cfg(test)]
    pub(crate) fn draw_message(&self,scope:RunRngScope,semantic:&[u64],counter:u64)->Vec<u8>{
        let mut message=RUN_RNG_LABEL.to_vec();
        message.extend_from_slice(self.partition.as_str().as_bytes()); message.push(0);
        message.extend_from_slice(&self.ordinal.to_be_bytes()); message.push(0);
        message.extend_from_slice(scope.as_str().as_bytes()); message.push(0);
        if !semantic.is_empty() {
            message.extend_from_slice(b"semantic-v1\0");
            message.extend_from_slice(&(semantic.len() as u64).to_be_bytes());
            for value in semantic { message.extend_from_slice(&value.to_be_bytes()); }
        }
        message.extend_from_slice(&counter.to_be_bytes());
        message
    }
    pub fn bounded(&mut self,scope:RunRngScope,bound:u64)->usize {
        assert!(bound>0,"bounded selection requires a nonzero domain");
        let zone=u64::MAX-(u64::MAX%bound);
        loop { let value=self.draw(scope); if value<zone { return (value%bound) as usize; } }
    }
    pub fn bounded_scoped(&mut self,scope:RunRngScope,semantic:&[u64],bound:u64)->usize {
        assert!(bound>0,"bounded selection requires a nonzero domain");
        let zone=u64::MAX-(u64::MAX%bound);
        loop { let value=self.draw_scoped(scope,semantic); if value<zone { return (value%bound) as usize; } }
    }
    /// Checked draws for detached Falling preparation; do not reuse a saturated
    /// ticket. Existing accepted scopes retain their established implementation.
    pub(crate) fn bounded_scoped_checked(&mut self,scope:RunRngScope,semantic:&[u64],bound:u64)->Result<usize,super::RunError>{
        if bound==0{return Err(super::RunError::InvalidAction)}
        let zone=u64::MAX-(u64::MAX%bound);
        loop{
            if self.counters.get(&(scope,semantic.to_vec()))==Some(&u64::MAX){return Err(super::RunError::Unsupported("run scoped RNG exhausted"))}
            let value=self.draw_scoped(scope,semantic);if value<zone{return Ok((value%bound) as usize)}
        }
    }
    /// Test-only counter placement for exhaustion/saturation controls.
    #[cfg(test)]
    pub(crate) fn set_counter_for_test(&mut self,scope:RunRngScope,semantic:&[u64],value:u64){self.counters.insert((scope,semantic.to_vec()),value);}
    /// Derive a native combat stream from the same run identity while keeping
    /// every actor's AI stream distinct from shuffle and peer actors.
    pub fn derive_combat_stream(&self, domain:Domain, act:u8, encounter:&str,
        visit:u64, actor_slot:Option<u32>, role:&str)->Result<StreamState,RngError> {
        let combat_identity=format!("run-{}-act-{}-{}-visit-{}",self.ordinal,act,encounter,visit);
        let scope=match domain {
            Domain::RunCombatShuffle if actor_slot.is_none()=>vec![combat_identity.clone()],
            Domain::RunCombatCardRandom if actor_slot.is_none()=>vec![combat_identity.clone()],
            Domain::RunCombatEnemyAi if actor_slot.is_some()=>vec![combat_identity.clone(),format!("slot-{}-{role}",actor_slot.unwrap())],
            _=>return Err(RngError::Invalid("combat stream requires run shuffle/card-random/actor AI domain and matching owner".into())),
        };
        let id=StreamId::new(domain,scope)?;
        let key=hmac_sha256(&self.root,format!("{RUN_COMBAT_STREAM_LABEL}{combat_identity}").as_bytes());
        make_stream(&key,self.partition,&combat_identity,&id)
    }
}

#[cfg(test)]
mod m20_content_rng_tests{
    use super::*;
    #[test]
    fn m20_defect_content_staged_sampler_rolls_back_earlier_slots_on_rng_exhaustion(){
        use crate::content::CardRarity;
        assert_eq!(RunRngScope::parse("run.m20.card.offer"),Ok(RunRngScope::M20CardOffer));
        let mut rng=RunRng::new([27;32],Partition::Test,3);rng.counters.insert((RunRngScope::M20CardOffer,vec![17,1]),u64::MAX);
        let before=rng.clone();assert_eq!(crate::run::rooms::m20_content::sample_capture_row(&mut rng,&[17],&[CardRarity::Common,CardRarity::Uncommon]),Err(crate::run::RunError::Unsupported("run scoped RNG exhausted")));assert_eq!(rng,before);
    }
}

#[cfg(test)]
mod m22_capture_tests{
    use super::*;
    use crate::{content::RelicId,run::{rooms::{m22_content as data,rewards::capture_m22_relic},*}};
    #[test]
    fn m22_relic_capture_uses_eligible_order_singleton_empty_and_exact_scoped_request(){
        assert_eq!(RunRngScope::parse("run.m22.relic.capture"),Ok(RunRngScope::M22RelicCapture));
        for pool in [&data::ELITE_RELICS[..],&data::CHEST_RELICS[..]]{
            for unowned in pool{
                let owned=pool.iter().copied().filter(|id|id!=unowned).collect::<Vec<_>>();
                let mut rng=RunRng::new([37;32],Partition::Test,0);let key=[1,8,42,6,0];let mut expected=rng.clone();
                assert_eq!(expected.bounded_scoped_checked(RunRngScope::M22RelicCapture,&key,1),Ok(0));
                assert_eq!(capture_m22_relic(&mut rng,&key,pool,&owned),Ok(Some(*unowned)));assert_eq!(rng,expected);
            }
            let mut rng=RunRng::new([37;32],Partition::Test,0);let before=rng.clone();assert_eq!(capture_m22_relic(&mut rng,&[1,8,42,6,0],pool,pool),Ok(None));assert_eq!(rng,before);
            let owned=[pool[0],RelicId::PureWater];let eligible=pool.iter().copied().filter(|id|!owned.contains(id)).collect::<Vec<_>>();
            let key=[1,8,42,6,0];let mut actual=before.clone();let mut expected=before.clone();let index=expected.bounded_scoped_checked(RunRngScope::M22RelicCapture,&key,eligible.len() as u64).unwrap();
            assert_eq!(capture_m22_relic(&mut actual,&key,pool,&owned),Ok(Some(eligible[index])));assert_eq!(actual,expected);
        }
        // Source-qualified checked owner uses strict '< zone'; MAX is rejected even at1.
        let zone=u64::MAX-(u64::MAX%1);assert!(!(u64::MAX<zone));
    }
    #[test]
    fn m22_relic_capture_saturation_is_atomic_and_semantic_keys_are_distinct(){
        let key=vec![1,8,42,6,0];let mut rng=RunRng::new([37;32],Partition::Test,0);
        rng.counters.insert((RunRngScope::M22RelicCapture,key.clone()),u64::MAX);let before=rng.clone();
        assert_eq!(capture_m22_relic(&mut rng,&key,&data::ELITE_RELICS,&[]),Err(RunError::Unsupported("run scoped RNG exhausted")));assert_eq!(rng,before);
        assert_eq!(capture_m22_relic(&mut rng,&key,&[RelicId::Vajra],&[]),Err(RunError::Unsupported("run scoped RNG exhausted")));assert_eq!(rng,before);
        for other in [[1,8,43,6,0],[1,8,42,7,0],[1,8,42,6,1]]{assert!(capture_m22_relic(&mut rng,&other,&data::ELITE_RELICS,&[]).unwrap().is_some());assert_eq!(rng.counters.get(&(RunRngScope::M22RelicCapture,other.to_vec())),Some(&1));}
        assert_eq!(rng.counters.get(&(RunRngScope::M22RelicCapture,key)),Some(&u64::MAX));
        for scope in [RunRngScope::M20CardOffer,RunRngScope::MapTopology,RunRngScope::CombatGoldReward]{assert_eq!(rng.clone().draw(scope),before.clone().draw(scope));}
    }
    #[test]
    fn m22_relic_capture_chest_failure_and_accepted_capture_are_retained(){
        let mut p=CreationProfile::experienced();p.neow_eligible=false;
        let mut r=Run::new_room_scenario(RunConfig{character:crate::content::Character::Watcher,ascension:0},p,RunRng::new([7;32],Partition::Test,0),RoomScenario::M22WatcherAct1).unwrap();
        let id=r.decision().unwrap().room.unwrap().choices[1].id;r.step(&GameAction::Choose(id)).unwrap();
        // Construct only map position; enter/open through real room owners.
        let map=r.state.map.as_mut().unwrap();let target=map.nodes.iter().find(|n|n.kind==MapNodeKind::Treasure).unwrap().id;
        let prev=map.nodes.iter_mut().find(|n|n.edges.contains(&target)).unwrap();prev.visited=true;r.state.floor=prev.floor;map.current=Some(prev.id);
        r.step(&GameAction::Move(target)).unwrap();let id=r.decision().unwrap().room.unwrap().choices.iter().find(|c|c.kind==RoomChoiceKind::OpenChest).unwrap().id;
        let key=vec![1,r.state.floor as u64,target.0,r.state.combat_progress.entered,1];r.state.rng.counters.insert((RunRngScope::M22RelicCapture,key.clone()),u64::MAX);let before=r.clone();
        assert_eq!(r.step(&GameAction::Choose(id)),Err(RunError::Unsupported("run scoped RNG exhausted")));assert_eq!(r,before);
        r.state.rng.counters.remove(&(RunRngScope::M22RelicCapture,key.clone()));let mut expected=r.state.rng.clone();let index=expected.bounded_scoped_checked(RunRngScope::M22RelicCapture,&key,3).unwrap();
        r.step(&GameAction::Choose(id)).unwrap();assert_eq!(r.state.rng,expected);let screen=r.decision().unwrap().room.unwrap();assert!(screen.choices.iter().any(|c|c.kind==RoomChoiceKind::Relic(data::CHEST_RELICS[index])));
        let before=r.clone();assert_eq!(r.decision().unwrap().room.unwrap(),screen);assert_eq!(r.decision().unwrap().room.unwrap(),screen);assert!(r.step(&GameAction::Choose(id)).is_err());assert_eq!(r,before);
        let proceed=screen.choices.iter().find(|c|c.kind==RoomChoiceKind::Proceed).unwrap().id;r.step(&GameAction::Choose(proceed)).unwrap();assert_eq!(r.state.rng,expected);
    }
}

/// M38b fix 5: `draw_scoped` feeds the HMAC exactly the pre-M38b message bytes, and its counters advance as before.
#[cfg(test)]
mod m38b_draw_message_tests {
    use super::*;
    #[test]
    fn streamed_draw_matches_built_message() {
        let mut rng = RunRng::new([7; 32], Partition::Test, 3);
        let reference = rng.clone();
        let scopes = [RunRngScope::MapTopology, RunRngScope::M28CardRarity, RunRngScope::NeowCategory0];
        let semantics: [&[u64]; 4] = [&[], &[0], &[1, u64::MAX], &[5, 6, 7, 8, 9]];
        let mut counters = BTreeMap::new();
        for round in 0..3 {
            for scope in scopes {
                for semantic in semantics {
                    let counter = counters.entry((scope, semantic.to_vec())).or_insert(0_u64);
                    let expected = u64::from_be_bytes(hmac_sha256(&reference.root, &reference.draw_message(scope, semantic, *counter))[..8].try_into().unwrap());
                    *counter += 1;
                    assert_eq!(rng.draw_scoped(scope, semantic), expected, "round {round} {scope:?} {semantic:?}");
                }
            }
        }
        assert_eq!(rng.counters, counters);
    }
}
