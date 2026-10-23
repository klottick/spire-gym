use std::collections::BTreeMap;
use crate::combat::rng::{hmac_sha256, make_stream, Domain, Partition, RngError, StreamId, StreamState};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RunRngScope { NeowCategory0, NeowCategory1, NeowDrawback, NeowCategory2,
    MapTopology, MapRoomBucket, MapBurningElite, MapEncounter, MapQuestionRoom, CombatEnemySetup, CombatGoldReward, Act1BossOrder, BurningEliteBuff,Act2BossOrder,Act2Joust }
impl RunRngScope {
    fn as_str(self)->&'static str { match self {
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
    } }
    pub fn parse(value:&str)->Result<Self,RunRngScopeError> { match value {
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
        _=>Err(RunRngScopeError::Unregistered(value.to_owned())),
    }}
}
#[derive(Clone, Debug, PartialEq, Eq)] pub enum RunRngScopeError { Unregistered(String) }

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunRng { root:[u8;32], partition:Partition, ordinal:u64, counters:BTreeMap<(RunRngScope,Vec<u64>),u64> }
impl RunRng {
    pub fn new(root:[u8;32], partition:Partition, ordinal:u64)->Self { Self{root,partition,ordinal,counters:BTreeMap::new()} }
    pub fn draw(&mut self, scope:RunRngScope)->u64 { self.draw_scoped(scope,&[]) }
    pub fn draw_scoped(&mut self, scope:RunRngScope, semantic:&[u64])->u64 {
        let counter=self.counters.entry((scope,semantic.to_vec())).or_insert(0);
        let mut message=b"spire-lab-run-rng-v1\0".to_vec();
        message.extend_from_slice(self.partition.as_str().as_bytes()); message.push(0);
        message.extend_from_slice(&self.ordinal.to_be_bytes()); message.push(0);
        message.extend_from_slice(scope.as_str().as_bytes()); message.push(0);
        if !semantic.is_empty() {
            message.extend_from_slice(b"semantic-v1\0");
            message.extend_from_slice(&(semantic.len() as u64).to_be_bytes());
            for value in semantic { message.extend_from_slice(&value.to_be_bytes()); }
        }
        message.extend_from_slice(&counter.to_be_bytes()); *counter=counter.saturating_add(1);
        u64::from_be_bytes(hmac_sha256(&self.root,&message)[..8].try_into().unwrap())
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
        let key=hmac_sha256(&self.root,format!("spire-lab/run-combat-stream/v1/{combat_identity}").as_bytes());
        make_stream(&key,self.partition,&combat_identity,&id)
    }
}
