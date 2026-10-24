//! Owned public-only Run transport. Resolver identities never become features.
use crate::{RunCapacities,RunSessionError};
use spire_core::run::{PublicDecision,map::PublicMap};
use spire_core::{combat::{CombatAction,CombatCardRef,LiveCardView,Outcome,Stance,OrbKind},content::{CardType,Character},run::*};
use crate::run_schema as schema;
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct RunFeatureGroup{pub key:&'static str,pub shape:Vec<usize>,pub values:Vec<i64>,pub known:Vec<i8>,pub occupied:Option<Vec<i8>>}
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct RunObservation{pub groups:Vec<RunFeatureGroup>,pub action_mask:Vec<i8>}
impl RunObservation{pub fn group(&self,key:&str)->Option<&RunFeatureGroup>{self.groups.iter().find(|g|g.key==key)}}
fn number<T:TryInto<i64>>(v:T)->Result<i64,RunSessionError>{v.try_into().map_err(|_|RunSessionError::Input("public integer exceeds int64"))}
fn reserved<T:Clone>(len:usize,value:T)->Result<Vec<T>,RunSessionError>{let mut v=Vec::new();v.try_reserve_exact(len).map_err(|_|RunSessionError::DeliveryFault("Run encoding allocation failed"))?;v.resize(len,value);Ok(v)}
fn capacity(group:&'static str,required:usize,cap:usize)->Result<(),RunSessionError>{if required>cap{Err(RunSessionError::Capacity{group,required,capacity:cap})}else{Ok(())}}
impl RunFeatureGroup{
    fn row(&mut self,index:usize,values:&[Option<i64>])->Result<(),RunSessionError>{
        let cols=*self.shape.last().unwrap();let rows=if self.occupied.is_some(){self.shape[0]}else{1};
        capacity(self.key,index+1,rows)?;
        if values.len()!=cols{return Err(RunSessionError::Input("internal Run column mismatch"))}
        for (column,value) in values.iter().enumerate(){if let Some(value)=value{self.values[index*cols+column]=*value;self.known[index*cols+column]=1;}}
        if let Some(occupied)=&mut self.occupied{occupied[index]=1;}Ok(())
    }
}
impl RunObservation{
    fn group_mut(&mut self,key:&str)->&mut RunFeatureGroup{self.groups.iter_mut().find(|g|g.key==key).expect("declared Run group")}
    pub(crate) fn owned(&self)->Result<Self,RunSessionError>{
        fn copy<T:Clone>(source:&[T])->Result<Vec<T>,RunSessionError>{let mut v=Vec::new();v.try_reserve_exact(source.len()).map_err(|_|RunSessionError::DeliveryFault("Run output allocation failed"))?;v.extend_from_slice(source);Ok(v)}
        let mut groups=Vec::new();groups.try_reserve_exact(self.groups.len()).map_err(|_|RunSessionError::DeliveryFault("Run output allocation failed"))?;
        for g in &self.groups{groups.push(RunFeatureGroup{key:g.key,shape:copy(&g.shape)?,values:copy(&g.values)?,known:copy(&g.known)?,occupied:g.occupied.as_deref().map(copy).transpose()?});}
        Ok(Self{groups,action_mask:copy(&self.action_mask)?})
    }
}
fn terminal(t:Option<RunTerminal>)->i64{match t{None=>0,Some(RunTerminal::Death)=>1,Some(RunTerminal::EmptyDeck)=>2,Some(RunTerminal::Act3NoKeys)=>3,Some(RunTerminal::HeartVictory)=>4}}
fn outcome(o:Outcome)->i64{match o{Outcome::Victory=>1,Outcome::Death=>2}}
fn card_type(t:CardType)->i64{match t{CardType::Attack=>1,CardType::Skill=>2,CardType::Power=>3,CardType::Status=>4,CardType::Curse=>5}}
fn permanent(d:&PublicDecision,id:u64)->Result<usize,RunSessionError>{d.objects.iter().position(|c|c.instance==id).ok_or(RunSessionError::Input("public permanent reference missing"))}
fn position<T:PartialEq>(rows:&[T],id:&T)->Result<usize,RunSessionError>{rows.iter().position(|v|v==id).ok_or(RunSessionError::Input("public row reference missing"))}

pub fn encode_run(d:&PublicDecision,map:Option<&PublicMap>,caps:&RunCapacities)->Result<RunObservation,RunSessionError>{
    schema::validate(caps)?;
    if d.combat.as_ref().and_then(|c|c.selection.as_ref()).is_some_and(|s|
        matches!(s.parent,spire_core::combat::SelectionParent::Potion{..})
            || !matches!(s.operation,spire_core::combat::SelectionOperation::Move(
            spire_core::combat::SelectionDisposition::Hand|
            spire_core::combat::SelectionDisposition::Discard|
            spire_core::combat::SelectionDisposition::Exhaust))
            || s.minimum!=s.required || s.maximum!=s.required){
        return Err(RunSessionError::UnsupportedBoundary("Run observation v1 cannot encode combat selection"))
    }
    capacity("permanent_cards",d.objects.len(),caps.permanent_cards)?;
    capacity("actions",d.candidates.len(),caps.actions)?;
    let offers=d.room.as_ref().map(|r|r.choices.as_slice()).unwrap_or(&[]);capacity("offers",offers.len(),caps.offers)?;
    if let Some(map)=map{
        capacity("map_nodes",map.nodes.len(),caps.map_nodes)?;
        let edges=map.nodes.iter().try_fold(0usize,|n,row|n.checked_add(row.edges.len())).ok_or(RunSessionError::Input("map edge count overflow"))?;
        capacity("map_edges",edges,caps.map_edges)?;
    }
    let mut cards:Vec<(&LiveCardView,i64,usize,Option<usize>)>=vec![];
    if let Some(c)=&d.combat{
        capacity("enemies",c.enemies.len(),caps.enemies)?;
        if c.enemies.iter().any(|enemy|matches!(enemy.enemy_id,spire_core::content::EnemyId::SpireShield|spire_core::content::EnemyId::SpireSpear)){
            return Err(RunSessionError::UnsupportedBoundary("Run observation v1 cannot encode the Shield-and-Spear encounter"));
        }
        let powers=c.enemies.iter().try_fold(c.player.powers.len(),|n,e|n.checked_add(e.powers.len())).ok_or(RunSessionError::Input("power count overflow"))?;
        capacity("powers",powers,caps.powers)?;capacity("orbs",c.character.orbs.len(),caps.orbs)?;
        for (zone,pile) in [(1,&c.hand),(2,&c.discard),(3,&c.exhaust),(4,&c.played_powers)]{
            for (pos,card) in pile.iter().enumerate(){if cards.iter().any(|(v,_,_,_)|v.instance==card.instance){return Err(RunSessionError::Input("duplicate visible card"))}cards.push((card,zone,pos,None));}
        }
        for (pos,stasis) in c.stasis.iter().enumerate(){
            if cards.iter().any(|(v,_,_,_)|v.instance==stasis.card.instance){return Err(RunSessionError::Input("duplicate Stasis card"))}
            let owner=c.enemies.iter().position(|e|e.instance_id==stasis.orb).ok_or(RunSessionError::Input("public Stasis owner missing"))?;
            cards.push((&stasis.card,5,pos,Some(owner)));
        }
        if let Some(s)=&c.selection{for (pos,card) in s.candidates.iter().enumerate(){
            if let Some((existing,_,_,_))=cards.iter().find(|(v,_,_,_)|v.instance==card.instance){if **existing!=*card{return Err(RunSessionError::Input("conflicting public selection card"))}}
            else{cards.push((card,6,pos,None));}
        }}
        for card in &c.playing_cards{
            match cards.iter().find(|(existing,_,_,_)|existing.instance==card.instance){
                Some((existing,_,_,_)) if **existing==*card=>{},
                Some(_)=>return Err(RunSessionError::Input("conflicting public playing card")),
                None=>return Err(RunSessionError::UnsupportedBoundary("Run observation v1 cannot encode unrepresented playing cards")),
            }
        }
    }
    capacity("combat_cards",cards.len(),caps.combat_cards)?;
    let (relics,counters,potions)=if let Some(c)=&d.combat{let i=c.inventory.as_ref().ok_or(RunSessionError::Input("combat inventory missing"))?;(&i.relics,&i.relic_counters,&i.potions)}else{(&d.player.relics,&d.player.relic_counters,&d.player.potions)};
    capacity("relics",relics.len(),caps.relics)?;capacity("potions",potions.len(),caps.potions)?;
    let selection_count=if let Some(s)=&d.selection{s.eligible.len()}else{d.combat.as_ref().and_then(|c|c.selection.as_ref()).map_or(0,|s|s.candidates.len())};
    capacity("selection",selection_count,caps.selection)?;
    let mut o=RunObservation{groups:vec![],action_mask:reserved(caps.actions,0)?};
    for spec in schema::groups(caps){
        let cols=spec.fields.split(',').count();let rows=spec.rows.unwrap_or(1);let mut values=reserved(rows*cols,0)?;
        // Unavailable references in real rows are -1/known0; all padding remains0.
        let occupied_count=match spec.key{"combat_cards"=>cards.len(),"actions"=>d.candidates.len(),"offers"=>offers.len(),"selection"=>selection_count,"powers"=>d.combat.as_ref().map_or(0,|c|c.player.powers.len()+c.enemies.iter().map(|e|e.powers.len()).sum::<usize>()),"map_edges"=>map.map_or(0,|m|m.nodes.iter().map(|n|n.edges.len()).sum()),"current_node"=>1,_=>0};
        for row in 0..occupied_count{for col in spec.references{values[row*cols+col]=-1;}}
        o.groups.push(RunFeatureGroup{key:spec.key,shape:spec.rows.map_or_else(||vec![cols],|rows|vec![rows,cols]),values,known:reserved(rows*cols,0)?,occupied:spec.rows.map(|rows|reserved(rows,0)).transpose()?});
    }
    o.group_mut("context").row(0,&[
        Some(schema::character(d.player.character)),None,Some(i64::from(d.act)),Some(i64::from(d.floor)),d.room.as_ref().map(|r|schema::room(r.kind)),
        d.room.as_ref().and_then(|r|if let RoomViewKind::Event(e)=r.kind{Some(schema::event(e))}else{None}).transpose()?,
        Some(terminal(d.terminal)),Some(i64::from(d.reward_total)),Some(i64::from(d.player.keys[0])),Some(i64::from(d.player.keys[1])),Some(i64::from(d.player.keys[2])),
        map.and_then(|m|m.boss).map(schema::encounter).transpose()?,map.map(|m|i64::from(m.winged_moves)),Some(number(d.combat_progress.entered)?),Some(number(d.combat_progress.settled)?),
        d.last_combat.as_ref().map(|c|schema::encounter(c.encounter)).transpose()?,d.last_combat.as_ref().map(|c|outcome(c.outcome)),d.content_profile.map(schema::profile).transpose()?,
    ])?;
    // PublicDecision has no ascension field. M9's named profile fixes A0; never
    // infer it for an unknown/unrelated public profile.
    if d.content_profile==Some(RoomScenario::M9IroncladAct3){let context=o.group_mut("context");context.values[1]=0;context.known[1]=1;}
    o.group_mut("player").row(0,&[Some(i64::from(d.player.hp)),Some(i64::from(d.player.max_hp)),Some(i64::from(d.player.gold)),None,None,None,None,None,None,None,None,None])?;
    for (row,card) in d.objects.iter().enumerate(){
        if d.objects[..row].iter().any(|c|c.instance==card.instance){return Err(RunSessionError::Input("duplicate public permanent identity"))}
        o.group_mut("permanent_cards").row(row,&[Some(schema::card(card.id)?),Some(i64::from(card.upgrades))])?;
    }
    if let Some(map)=map{
        let ids=map.nodes.iter().map(|n|n.id).collect::<Vec<_>>();let mut edge=0;
        for (row,n) in map.nodes.iter().enumerate(){
            if ids[..row].contains(&n.id){return Err(RunSessionError::Input("duplicate public node identity"))}
            o.group_mut("map_nodes").row(row,&[Some(i64::from(n.floor)),Some(i64::from(n.x)),Some(i64::from(n.y)),Some(schema::node(n.kind)),n.resolved_room.map(schema::node),n.burning.map(i64::from),Some(i64::from(n.visited)),Some(i64::from(n.revealed))])?;
            for target in &n.edges{o.group_mut("map_edges").row(edge,&[Some(number(row)?),Some(number(position(&ids,target)?)?)])?;edge+=1;}
        }
        if let Some(id)=map.current{o.group_mut("current_node").row(0,&[Some(number(position(&ids,&id)?)?)])?;}
    }
    let card_row=|instance:u64|cards.iter().position(|(c,_,_,_)|c.instance.0==instance).ok_or(RunSessionError::Input("public combat card reference missing"));
    for (row,(card,zone,pos,stasis)) in cards.iter().enumerate(){
        let permanent_row=match card.origin{CombatCardRef::Permanent(id)=>Some(number(permanent(d,id.0)?)?),CombatCardRef::CombatLocal(_)=>None};
        let selected=d.combat.as_ref().and_then(|c|c.selection.as_ref()).map(|s|i64::from(s.selected.contains(&card.instance)));
        o.group_mut("combat_cards").row(row,&[Some(*zone),Some(number(*pos)?),Some(schema::card(card.id)?),Some(i64::from(card.upgrades)),card.effective_cost.map(i64::from),permanent_row,stasis.map(number).transpose()?,selected])?;
    }
    if let Some(c)=&d.combat{
        o.group_mut("player").row(0,&[Some(i64::from(c.player.hp)),Some(i64::from(c.player.max_hp)),None,Some(i64::from(c.player.block)),Some(i64::from(c.player.energy)),Some(i64::from(c.player.max_energy)),Some(i64::from(c.turn)),Some(number(c.draw_count)?),
            (d.player.character==Character::Watcher).then_some(match c.character.stance{Stance::Neutral=>1,Stance::Wrath=>2,Stance::Calm=>3}),
            (d.player.character==Character::Defect).then_some(i64::from(c.character.focus)),c.hp_before_healing.map(i64::from),c.hp_after_cleanup.map(i64::from)])?;
        let mut power_row=0;
        for (owner,row,powers) in std::iter::once((1,0,&c.player.powers)).chain(c.enemies.iter().enumerate().map(|(i,e)|(5,i,&e.powers))){
            for power in powers{o.group_mut("powers").row(power_row,&[Some(owner),Some(number(row)?),Some(schema::named(schema::POWERS,power.id.0)?),Some(i64::from(power.amount))])?;power_row+=1;}
        }
        for (row,e) in c.enemies.iter().enumerate(){o.group_mut("enemies").row(row,&[Some(schema::enemy(e.enemy_id)?),Some(i64::from(e.hp)),Some(i64::from(e.max_hp)),Some(i64::from(e.block)),Some(schema::intent(e.intent.as_str())?),e.intent_damage.map(i64::from),Some(i64::from(e.replaced)),Some(i64::from(e.escaped)),Some(i64::from(e.half_dead)),Some(i64::from(e.logical_slot))])?;}
        if d.player.character==Character::Defect{for (row,orb) in c.character.orbs.iter().enumerate(){o.group_mut("orbs").row(row,&[Some(match orb{None=>0,Some(OrbKind::Lightning)=>1,Some(OrbKind::Frost)=>2})])?;if orb.is_none(){o.group_mut("orbs").occupied.as_mut().unwrap()[row]=0;}}}
        if let Some(s)=&c.selection{
            o.group_mut("selection_counts").row(0,&[Some(number(s.required)?),Some(number(s.selected.len())?)])?;
            for (row,card) in s.candidates.iter().enumerate(){o.group_mut("selection").row(row,&[Some(4),Some(number(card_row(card.instance.0)?)?),Some(1),Some(i64::from(s.selected.contains(&card.instance)))])?;}
        }
    }
    for (row,relic) in relics.iter().enumerate(){o.group_mut("relics").row(row,&[Some(schema::relic(*relic)?),counters.iter().find(|v|v.relic==*relic).map(|v|i64::from(v.value))])?;}
    for (row,potion) in potions.iter().enumerate(){o.group_mut("potions").row(row,&[Some(potion.map(schema::potion).transpose()?.unwrap_or(0))])?;if potion.is_none(){o.group_mut("potions").occupied.as_mut().unwrap()[row]=0;}}
    if let Some(s)=&d.selection{
        o.group_mut("selection_counts").row(0,&[Some(number(s.required)?),Some(number(s.selected_count)?)])?;
        for (row,card) in s.eligible.iter().enumerate(){o.group_mut("selection").row(row,&[Some(3),Some(number(permanent(d,card.instance)?)?),Some(1),None])?;}
    }
    for (row,offer) in offers.iter().enumerate(){
        let mut values=[None;16];values[0]=Some(schema::choice(&offer.kind));values[1]=Some(match offer.status{RoomChoiceStatus::Offered=>1,RoomChoiceStatus::Claimed=>2,RoomChoiceStatus::Skipped=>3,RoomChoiceStatus::Disabled=>4});values[2]=offer.price.map(i64::from);
        match offer.kind{
            RoomChoiceKind::NeowMaxHp{amount}|RoomChoiceKind::Gold(amount)=>values[3]=Some(i64::from(amount)),
            RoomChoiceKind::Rest{heal}=>values[3]=Some(i64::from(heal)),RoomChoiceKind::Act3Continue{stage}=>values[3]=Some(i64::from(stage)),
            RoomChoiceKind::Card{id,upgrades}=>{values[4]=Some(schema::card(id)?);values[5]=Some(i64::from(upgrades));},
            RoomChoiceKind::Relic(id)=>values[6]=Some(schema::relic(id)?),RoomChoiceKind::Potion(id)=>values[7]=Some(schema::potion(id)?),
            RoomChoiceKind::FallingLose{card_type:t,instance}=>{values[8]=Some(card_type(t));values[9]=instance.map(|id|permanent(d,id).and_then(number)).transpose()?;},
            RoomChoiceKind::JoustResult{owner_wins}=>values[10]=Some(i64::from(owner_wins)),_=>{},
        }
        if let Some(neow)=d.candidates.iter().find(|c|c.id==offer.id).and_then(|c|c.neow_offer.as_ref()){
            values[11]=Some(i64::from(neow.category));values[12]=Some(schema::reward(neow.reward));values[13]=Some(neow.drawback.map(schema::drawback).unwrap_or(0));
        }
        if let Some(s)=&d.selection{values[14]=Some(number(s.required)?);values[15]=Some(number(s.selected_count)?);}
        o.group_mut("offers").row(row,&values)?;
    }
    for (row,candidate) in d.candidates.iter().enumerate(){
        let mut values=[None;8];
        let source=match &candidate.action{
            GameAction::Move(id)=>{values[0]=Some(1);let map=map.ok_or(RunSessionError::Input("map action without public map"))?;Some((2,map.nodes.iter().position(|n|n.id==*id).ok_or(RunSessionError::Input("move target missing"))?))},
            GameAction::Choose(id)=>{values[0]=Some(2);let index=offers.iter().position(|c|c.id==*id).ok_or(RunSessionError::Input("offer reference missing"))?;values[5]=Some(schema::choice(&offers[index].kind));Some((7,index))},
            GameAction::Select(_)=>{values[0]=Some(3);let object=candidate.object.as_ref().ok_or(RunSessionError::Input("selection object missing"))?;let s=d.selection.as_ref().ok_or(RunSessionError::Input("selection missing"))?;Some((8,s.eligible.iter().position(|c|c.instance==object.instance).ok_or(RunSessionError::Input("eligible reference missing"))?))},
            GameAction::Confirm=>{values[0]=Some(4);None},GameAction::Cancel=>{values[0]=Some(5);None},
            GameAction::UsePotion{slot,target}=>{values[0]=Some(6);values[6]=Some(i64::from(*slot));encode_target(d,*target,&mut values)?;Some((6,*slot as usize))},
            GameAction::DiscardPotion{slot}=>{values[0]=Some(7);values[6]=Some(i64::from(*slot));Some((6,*slot as usize))},
            GameAction::Combat(action)=>match action{
                CombatAction::PlayCard{instance,target}=>{values[0]=Some(8);encode_target(d,*target,&mut values)?;Some((4,card_row(*instance)?))},
                CombatAction::SelectCard{instance}=>{values[0]=Some(9);Some((4,card_row(*instance)?))},
                CombatAction::UsePotion{slot,target}=>{values[0]=Some(6);values[6]=Some(number(*slot)?);encode_target(d,*target,&mut values)?;Some((6,*slot))},
                CombatAction::EndTurn=>{values[0]=Some(10);None},
                CombatAction::ConfirmSelection=>return Err(RunSessionError::UnsupportedBoundary("Run observation v1 cannot encode combat selection")),
            },
        };
        if let Some((group,index))=source{
            if group==6&&potions.get(index).is_none_or(|p|p.is_none()){return Err(RunSessionError::Input("potion action references empty slot"))}
            values[1]=Some(group);values[2]=Some(number(index)?);
        }
        o.group_mut("actions").row(row,&values)?;o.action_mask[row]=1;
    }
    Ok(o)
}

#[cfg(test)]
mod m10_v1_boundary_tests{
    use super::*;
    use spire_core::{combat::{rng::Partition,enemies::{self,SetupContext},Combat,PlayerState,permanent_live_card,CardInstanceId},content::{Character,CardId,EncounterId},run::{CreationProfile,Run,RunConfig,RunRng,RoomScenario,PublicPlayer,CombatProgress}};

    #[test]
    fn actual_m9_v1_observation_encoder_rejects_m10_profile_without_vocabulary_alias(){
        let mut profile=CreationProfile::experienced();profile.neow_eligible=false;
        let run=Run::new_room_scenario(RunConfig{character:Character::Ironclad,ascension:0},profile,RunRng::new([10;32],Partition::Test,0),RoomScenario::M10IroncladHeart).unwrap();
        let decision=run.decision().unwrap();
        assert_eq!(encode_run(&decision,None,&RunCapacities::m9_default()),Err(RunSessionError::Input("M10 profile is not representable in the M9 v1 observation schema")));
    }

    #[test]
    fn actual_m9_v1_encoder_rejects_public_shield_spear_pair_from_native_run(){
        let mut profile=CreationProfile::experienced();profile.neow_eligible=false;
        let run=Run::new_room_scenario(RunConfig{character:Character::Ironclad,ascension:0},profile,RunRng::new([19;32],Partition::Test,0),RoomScenario::M10IroncladHeart).unwrap();
        let mut setup_rng=RunRng::new([21;32],Partition::Test,0);
        let entry=spire_core::combat::enemies::setup(EncounterId::ShieldAndSpear,4,0,&mut setup_rng,spire_core::combat::enemies::SetupContext{
            player:PlayerState{hp:80,max_hp:80,block:0,energy:3,max_energy:3,draw_per_turn:5,powers:vec![]},player_powers:vec![],character:Character::Ironclad,
            cards:(0..5).map(|i|permanent_live_card(CardId::Strike,CardInstanceId(i+1),0)).collect(),opening_draw:5,encounter_visit:0}).unwrap();
        let combat=Combat::enter(entry).unwrap();
        let view=combat.public_full().unwrap();
        assert_eq!(view.enemies.iter().map(|enemy|enemy.enemy_id).collect::<Vec<_>>(),[spire_core::content::EnemyId::SpireShield,spire_core::content::EnemyId::SpireSpear]);
        let mut decision=run.decision().unwrap();
        decision.content_profile=None;
        decision.combat=Some(view);
        decision.room=None;
        decision.objects.clear();decision.candidates.clear();decision.offers.clear();decision.selection=None;
        decision.player=PublicPlayer{relic_counters:vec![],character:Character::Ironclad,hp:80,max_hp:80,gold:0,relics:vec![],potions:vec![None;3],keys:[false;3]};
        decision.act=4;decision.floor=51;decision.combat_progress=CombatProgress::default();
        assert!(matches!(encode_run(&decision,None,&RunCapacities::m9_default()),Err(RunSessionError::UnsupportedBoundary("Run observation v1 cannot encode the Shield-and-Spear encounter"))));
    }

    #[test]
    fn actual_heart_public_combat_view_is_rejected_by_the_m9_v1_consumer_vocabulary(){
        let cards=(1..=10).map(|id|permanent_live_card(CardId::Strike,CardInstanceId(id),0)).collect();
        let context=SetupContext{player:PlayerState{hp:80,max_hp:80,block:0,energy:3,max_energy:3,draw_per_turn:5,powers:vec![]},
            player_powers:vec![],character:Character::Ironclad,cards,opening_draw:5,encounter_visit:0};
        let mut rng=RunRng::new([29;32],Partition::Test,0);
        let entry=enemies::setup(EncounterId::CorruptHeart,4,0,&mut rng,context).unwrap();
        let mut items=Some(spire_core::items::CombatItemContext::new(vec![],vec![None,None,None]));
        let combat=Combat::enter_with_item_context(entry,&mut items).unwrap();
        let public=combat.public_full().unwrap();

        let mut profile=CreationProfile::experienced();profile.neow_eligible=false;
        let run=Run::new_room_scenario(RunConfig{character:Character::Ironclad,ascension:0},profile,
            RunRng::new([10;32],Partition::Test,0),RoomScenario::M10IroncladHeart).unwrap();
        let mut decision=run.decision().unwrap();
        decision.content_profile=None;
        decision.combat=Some(public);
        assert_eq!(encode_run(&decision,None,&RunCapacities::m9_default()),
            Err(RunSessionError::Input("unsupported public power or intent")),
            "the real encoder rejects the actual Heart public payload, independent of its profile-only rejection");
    }
}
fn encode_target(d:&PublicDecision,target:Option<spire_core::combat::TargetId>,values:&mut[Option<i64>;8])->Result<(),RunSessionError>{
    if let Some(target)=target{
        let c=d.combat.as_ref().ok_or(RunSessionError::Input("target without combat"))?;
        let index=c.targets.iter().position(|(id,valid)|*id==target&&*valid).ok_or(RunSessionError::Input("target is not publicly legal"))?;
        if index>=c.enemies.len(){return Err(RunSessionError::Input("target row missing"))}values[3]=Some(5);values[4]=Some(number(index)?);
    }Ok(())
}
