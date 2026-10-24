//! Separate native-owned numeric Run schema. No change to combat-v1.
use crate::RunSessionError;
use spire_core::{content::*,run::*};
use serde_json::{Value,json};
use sha2::{Digest,Sha256};
pub const RUN_OBSERVATION_SCHEMA_ID:&str="spire-gym-run-observation-v1";
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct RunCapacities{
    pub map_nodes:usize,pub map_edges:usize,pub permanent_cards:usize,pub combat_cards:usize,
    pub enemies:usize,pub powers:usize,pub relics:usize,pub potions:usize,
    pub offers:usize,pub selection:usize,pub orbs:usize,pub actions:usize,
}
impl RunCapacities{
    pub fn m9_default()->Self{Self{map_nodes:106,map_edges:742,permanent_cards:256,combat_cards:1024,enemies:32,powers:2048,relics:256,potions:16,offers:512,selection:256,orbs:10,actions:2048}}
}
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct RunObservationSpec{pub(crate) descriptor:serde_json::Value,pub(crate) fingerprint:[u8;32]}
impl RunObservationSpec{
    pub fn descriptor(&self)->serde_json::Value{self.descriptor.clone()}
    pub fn fingerprint(&self)->[u8;32]{self.fingerprint}
}
pub(crate) struct GroupSpec{pub key:&'static str,pub rows:Option<usize>,pub fields:&'static str,pub units:&'static str,pub references:&'static [usize]}
pub(crate) fn groups(c:&RunCapacities)->Vec<GroupSpec>{
    let make=|key,rows,fields,units,references|GroupSpec{key,rows,fields,units,references};
    vec![
        make("context",None,"character,ascension,act,floor,room_kind,event_kind,terminal_kind,reward_total,ruby,sapphire,emerald,map_boss,winged_moves,combats_entered,combats_settled,last_encounter,last_outcome,content_profile","category,level,act,floor,category,category,category,reward,flag,flag,flag,category,moves,combats,combats,category,category,category",&[]),
        make("player",None,"hp,max_hp,gold,block,energy,max_energy,turn,draw_count,stance,focus,hp_before_healing,hp_after_cleanup","hp,hp,gold,block,energy,energy,turn,cards,category,power,hp,hp",&[]),
        make("map_nodes",Some(c.map_nodes),"floor,x,y,kind,resolved_room,burning,visited,revealed","floor,column,row,category,category,flag,flag,flag",&[]),
        make("map_edges",Some(c.map_edges),"source_row,target_row","map_node_row,map_node_row",&[0,1]),
        make("current_node",None,"row","map_node_row",&[0]),
        make("permanent_cards",Some(c.permanent_cards),"card_kind,upgrades","category,upgrades",&[]),
        make("combat_cards",Some(c.combat_cards),"zone,zone_position,card_kind,upgrades,effective_cost,permanent_row,stasis_enemy_row,selected","category,position,category,upgrades,energy,permanent_card_row,enemy_row,flag",&[5,6]),
        make("enemies",Some(c.enemies),"enemy_kind,hp,max_hp,block,intent_kind,intent_damage,replaced,escaped,half_dead,logical_slot","category,hp,hp,block,category,hp,flag,flag,flag,slot",&[]),
        make("powers",Some(c.powers),"owner_kind,owner_row,power_kind,amount","category,owner_row,category,power",&[1]),
        make("relics",Some(c.relics),"relic_kind,visible_counter","category,counter",&[]),
        make("potions",Some(c.potions),"potion_kind","category",&[]),
        make("offers",Some(c.offers),"choice_kind,status,price,amount,card_kind,upgrades,relic_kind,potion_kind,card_type,permanent_row,owner_wins,neow_category,neow_reward,neow_drawback,required_count,selected_count","category,category,gold,choice_specific,category,upgrades,category,category,category,permanent_card_row,flag,category,category,category,cards,cards",&[9]),
        make("selection",Some(c.selection),"source_group,source_row,eligible,selected","category,source_row,flag,flag",&[1]),
        make("selection_counts",None,"required,selected_count","cards,cards",&[]),
        make("orbs",Some(c.orbs),"kind","category",&[]),
        make("actions",Some(c.actions),"verb,source_group,source_row,target_group,target_row,choice_kind,scalar0,scalar1","category,category,source_row,category,target_row,category,verb_specific,verb_specific",&[2,4]),
    ]
}
pub(crate) fn validate(c:&RunCapacities)->Result<(),RunSessionError>{
    let mut bytes=0usize;
    for g in groups(c){
        if g.rows==Some(0){return Err(RunSessionError::Input("capacities must be positive"))}
        let count=g.rows.unwrap_or(1).checked_mul(g.fields.split(',').count()).ok_or(RunSessionError::Input("Run shape overflow"))?;
        bytes=bytes.checked_add(count.checked_mul(9).ok_or(RunSessionError::Input("Run shape overflow"))?).ok_or(RunSessionError::Input("Run shape overflow"))?;
    }
    if bytes>isize::MAX as usize{return Err(RunSessionError::Input("Run shape overflow"))}Ok(())
}
pub fn run_observation_spec(c:&RunCapacities)->Result<RunObservationSpec,RunSessionError>{
    validate(c)?;let mut tensors=vec![];let vocab=vocabularies();
    for g in groups(c){
        let cols=g.fields.split(',').count();let shape=g.rows.map_or_else(||vec![cols],|rows|vec![rows,cols]);
        let columns=g.fields.split(',').zip(g.units.split(',')).map(|(field,unit)|domain(g.key,field,unit,c,&vocab)).collect::<Vec<_>>();
        let low=columns.iter().map(|c|c["low"].clone()).collect::<Vec<_>>();let high=columns.iter().map(|c|c["high"].clone()).collect::<Vec<_>>();
        tensors.push(json!({"key":g.key,"dtype":"int64","shape":shape,"low":low,"high":high,"space":"Box","fields":g.fields.split(',').collect::<Vec<_>>(),"units":g.units.split(',').collect::<Vec<_>>(),"columns":columns,"padding":0,"known":format!("{}_known",g.key),"reference_columns":g.references}));
        tensors.push(json!({"key":format!("{}_known",g.key),"dtype":"int8","shape":shape,"low":0,"high":1,"space":"MultiBinary","padding":0}));
        if let Some(rows)=g.rows{tensors.push(json!({"key":format!("{}_occupied",g.key),"dtype":"int8","shape":[rows],"low":0,"high":1,"space":"MultiBinary","padding":0}));}
    }
    tensors.push(json!({"key":"action_mask","dtype":"int8","shape":[c.actions],"low":0,"high":1,"space":"MultiBinary","padding":0}));
    let descriptor=json!({"schema_id":RUN_OBSERVATION_SCHEMA_ID,"profile_id":"M9IroncladAct3","array_layout":"owned_c_contiguous_native_endian_row_major","capacities":{"map_nodes":c.map_nodes,"map_edges":c.map_edges,"permanent_cards":c.permanent_cards,"combat_cards":c.combat_cards,"enemies":c.enemies,"powers":c.powers,"relics":c.relics,"potions":c.potions,"offers":c.offers,"selection":c.selection,"orbs":c.orbs,"actions":c.actions},"action_space":{"type":"Discrete","n":c.actions,"start":0},"tensors":tensors,"vocabularies":vocabularies(),"semantics":{"padding":"zero,known0,occupied0","unknown_reference":"minus_one,known0 in occupied rows","counts":"only published public counts; no hidden draw rows","identity":"row references only; no raw resolver IDs","power_absence":"no row; listed signed amounts are public","potions":"belt slot rows; empty slot known1/occupied0/kind0; unused transport padding known0","orbs":"N/A for Ironclad; known empty slot differs from padding/N/A","selection":"Run selected_count does not reveal selected identities","actions":"legal candidate order; no paging, truncation or policy","scalar0":"potion belt slot for potion verbs; otherwise absent","scalar1":"reserved absent","offer_amount":"Neow HP/gold/heal/Joust result/Falling type/stage per choice_kind"}});
    let mut hash=Sha256::new();hash.update(b"spire-gym/run-schema/v1\0");hash.update(crate::canonical_json(&descriptor).map_err(|_|RunSessionError::Input("invalid Run descriptor"))?);
    Ok(RunObservationSpec{descriptor,fingerprint:hash.finalize().into()})
}

fn domain(group:&str,field:&str,unit:&str,c:&RunCapacities,vocab:&Value)->Value{
    let category=match field{
        "character"=>Some("character"),"room_kind"=>Some("room"),"event_kind"=>Some("event"),"terminal_kind"=>Some("terminal"),
        "map_boss"|"last_encounter"=>Some("encounter"),"last_outcome"=>Some("outcome"),"content_profile"=>Some("profile"),
        "card_kind"=>Some("card"),"enemy_kind"=>Some("enemy"),"intent_kind"=>Some("intent"),"power_kind"=>Some("power"),
        "relic_kind"=>Some("relic"),"potion_kind"=>Some("potion"),"choice_kind"=>Some("choice"),"status"=>Some("choice_status"),
        "card_type"=>Some("card_type"),"neow_reward"=>Some("neow_reward"),"neow_drawback"=>Some("neow_drawback"),
        "owner_kind"|"source_group"|"target_group"=>Some("row_group"),"zone"=>Some("zone"),"stance"=>Some("stance"),"verb"=>Some("verb"),
        "kind" if group=="orbs"=>Some("orb"),"kind"|"resolved_room" if group=="map_nodes"=>Some("map_kind"),_=>None,
    };
    if let Some(category)=category{return json!({"field":field,"unit":unit,"low":0,"high":vocab[category].as_array().unwrap().len()-1,"vocabulary":category})}
    let reference=match (group,field){
        ("map_edges",_)|("current_node",_)=>Some(("map_nodes",c.map_nodes)),
        ("combat_cards","permanent_row")|("offers","permanent_row")=>Some(("permanent_cards",c.permanent_cards)),
        ("combat_cards","stasis_enemy_row")=>Some(("enemies",c.enemies)),
        ("powers","owner_row")=>Some(("owner_kind",c.enemies.max(1))),
        ("selection","source_row")=>Some(("source_group",c.permanent_cards.max(c.combat_cards))),
        ("actions","source_row")=>Some(("source_group",[c.map_nodes,c.combat_cards,c.potions,c.offers,c.selection].into_iter().max().unwrap())),
        ("actions","target_row")=>Some(("target_group",c.enemies)),_=>None,
    };
    if let Some((reference,count))=reference{return json!({"field":field,"unit":unit,"low":-1,"high":count-1,"reference":reference,"unavailable":-1})}
    let (low,high)=if unit=="flag"{(0,1)}else if field=="neow_category"{(0,3)}else if field=="ascension"||field=="act"||field=="x"||field=="y"||field=="logical_slot"{(0,u8::MAX as i64)}
        else if matches!(unit,"power"|"energy"|"block"|"counter"|"reward"){(i32::MIN as i64,i32::MAX as i64)}
        else if matches!(unit,"gold"|"hp"|"floor"|"turn"|"upgrades"|"choice_specific"){(0,u32::MAX as i64)}
        else{(0,i64::MAX)};
    json!({"field":field,"unit":unit,"low":low,"high":high})
}

// Order is explicit transport vocabulary, not a Rust enum discriminant or hash.
macro_rules! vocabulary{
    ($fun:ident,$table:ident,$ty:ident,[$($v:ident),+ $(,)?])=>{
        pub(crate) const $table:&[$ty]=&[$($ty::$v),+];
        pub(crate) fn $fun(id:$ty)->Result<i64,RunSessionError>{
            match id{$($ty::$v=>{}),+,_=>return Err(RunSessionError::Input(concat!("unsupported ",stringify!($ty))))}
            Ok($table.iter().position(|v|*v==id).expect("exhaustive transport vocabulary") as i64+1)
        }
    }
}
vocabulary!(card,CARDS,CardId,[Strike,Defend,Bash,StrikeGreen,DefendGreen,Neutralize,Survivor,StrikeBlue,DefendBlue,Zap,Dualcast,StrikePurple,DefendPurple,Eruption,Vigilance,Anger,IronWave,Cleave,PommelStrike,ShrugItOff,TwinStrike,Backflip,DaggerThrow,Deflect,QuickSlash,CloakAndDagger,Shiv,Clothesline,HeavyBlade,PerfectedStrike,Thunderclap,BallLightning,ColdSnap,Coolheaded,Leap,Consecrate,EmptyBody,EmptyFist,EmptyMind,BeamCell,GoForTheEyes,Barrage,CompileDriver,SweepingBeam,DeadlyPoison,PoisonedStab,Catalyst,Bloodletting,BurningPact,Disarm,Bludgeon,Impervious,Offering,AscendersBane,Miracle,Wound,Madness,Regret,Slimed,Dazed,Burn,Parasite,Inflame,Void]);
vocabulary!(enemy,ENEMIES,EnemyId,[Cultist,JawWorm,GremlinNob,Lagavulin,Sentry,Guardian,Hexaghost,LouseRed,LouseGreen,AcidSlimeS,AcidSlimeM,AcidSlimeL,SpikeSlimeS,SpikeSlimeM,SpikeSlimeL,SlimeBoss,SphericGuardian,Byrd,Chosen,Snecko,Centurion,Mystic,GremlinLeader,GremlinMad,GremlinSneaky,GremlinFat,GremlinShield,GremlinWizard,SlaverBlue,Taskmaster,SlaverRed,BookOfStabbing,Champ,Collector,TorchHead,BronzeAutomaton,BronzeOrb,OrbWalker,Repulsor,Exploder,Spiker,Darkling,SpireGrowth,Transient,Maw,WrithingMass,GiantHead,Nemesis,Reptomancer,Dagger,AwakenedOne,TimeEater,Deca,Donu]);
vocabulary!(encounter,ENCOUNTERS,EncounterId,[Cultist,JawWorm,TwoLouse,SmallSlimes,BlueSlaver,RedSlaver,Looter,LargeSlime,ThreeLouse,TwoFungiBeasts,GremlinGang,LotsOfSlimes,ExordiumThugs,ExordiumWildlife,GremlinNob,Lagavulin,ThreeSentries,Guardian,Hexaghost,SlimeBoss,SphericGuardian,Chosen,ShelledParasite,ThreeByrds,TwoThieves,ChosenAndByrds,SentryAndSphere,SnakePlant,Snecko,CenturionAndHealer,CultistAndChosen,ThreeCultists,ShelledParasiteAndFungi,GremlinLeader,Slavers,BookOfStabbing,BronzeAutomaton,Collector,Champ,ThreeDarklings,OrbWalker,ThreeShapes,SpireGrowth,Transient,FourShapes,Maw,SphereAndTwoShapes,JawWormHorde,WrithingMass,GiantHead,Nemesis,Reptomancer,AwakenedOne,TimeEater,DonuAndDeca]);
vocabulary!(relic,RELICS,RelicId,[BurningBlood,RingOfTheSnake,CrackedCore,PureWater,BloodVial,Anchor,Lantern,Strawberry,Pear,Mango,SacredBark,MarkOfBloom,MagicFlower,ToyOrnithopter,NeowsLament,BlackBlood,EmptyCage,SlaversCollar,RedMask,Vajra,OddlySmoothStone,BagOfPreparation,BagOfMarbles,BronzeScales]);
vocabulary!(potion,POTIONS,PotionId,[FirePotion,BloodPotion,FruitJuice,BlockPotion,EnergyPotion,StrengthPotion,SwiftPotion,WeakPotion,FearPotion,DexterityPotion,AncientPotion]);
vocabulary!(event,EVENTS,EventId,[Neow,UpgradeShrine,Purifier,AncientWriting,Joust,Falling,TombOfLordRedMask]);
pub(crate) fn character(c:Character)->i64{match c{Character::Ironclad=>1,Character::Silent=>2,Character::Defect=>3,Character::Watcher=>4}}
pub(crate) fn node(k:MapNodeKind)->i64{match k{MapNodeKind::Monster=>1,MapNodeKind::Elite=>2,MapNodeKind::Shop=>3,MapNodeKind::Rest=>4,MapNodeKind::Question=>5,MapNodeKind::Treasure=>6,MapNodeKind::Boss=>7,MapNodeKind::Event=>8}}
pub(crate) const ROOMS:&[&str]=&["padding","neow","map","combat","rewards","card_reward","shop","campfire","event","treasure","checkpoint_complete","boss_treasure","boss_relic_choice","boss_relic_child","proceed_without_relic","act2_ready","act2_prefix_complete","act3_ready","act3_prefix_complete","act3_boss_complete","act3_ending","act4_ready"];
pub(crate) fn room(k:RoomViewKind)->i64{match k{RoomViewKind::Neow=>1,RoomViewKind::Map=>2,RoomViewKind::Combat=>3,RoomViewKind::Rewards=>4,RoomViewKind::CardReward=>5,RoomViewKind::Shop=>6,RoomViewKind::Campfire=>7,RoomViewKind::Event(_)=>8,RoomViewKind::Treasure=>9,RoomViewKind::CheckpointComplete=>10,RoomViewKind::BossTreasure=>11,RoomViewKind::BossRelicChoice=>12,RoomViewKind::BossRelicChild=>13,RoomViewKind::ProceedWithoutRelic=>14,RoomViewKind::Act2Ready=>15,RoomViewKind::Act2PrefixComplete=>16,RoomViewKind::Act3Ready=>17,RoomViewKind::Act3PrefixComplete=>18,RoomViewKind::Act3BossComplete=>19,RoomViewKind::Act3Ending=>20,RoomViewKind::Act4Ready=>21}}
pub(crate) fn profile(p:RoomScenario)->Result<i64,RunSessionError>{Ok(match p{RoomScenario::M4Act1Prefix=>1,RoomScenario::M5Act1Captured=>2,RoomScenario::M6Act2Prefix=>3,RoomScenario::M7Act2Captured=>4,RoomScenario::M8Act3Prefix=>5,RoomScenario::M9IroncladAct3=>6,RoomScenario::M10IroncladHeart=>return Err(RunSessionError::Input("M10 profile is not representable in the M9 v1 observation schema"))})}
pub(crate) const CHOICES:&[&str]=&["padding","neow_max_hp","neow_lament","gold","card_reward","card","potion","relic","sapphire","emerald","open_chest","rest","smith","recall","purge","shrine_upgrade","shrine_remove","ancient_elegance","ancient_simplicity","joust_murderer","joust_owner","joust_continue","joust_result","falling_lose","tomb_gold","tomb_offer_gold","leave","proceed","skip","confirm_skip","cancel_skip","act3_continue"];
pub(crate) fn choice(c:&RoomChoiceKind)->i64{match c{RoomChoiceKind::NeowMaxHp{..}=>1,RoomChoiceKind::NeowLament=>2,RoomChoiceKind::Gold(_)=>3,RoomChoiceKind::CardReward=>4,RoomChoiceKind::Card{..}=>5,RoomChoiceKind::Potion(_)=>6,RoomChoiceKind::Relic(_)=>7,RoomChoiceKind::Sapphire=>8,RoomChoiceKind::Emerald=>9,RoomChoiceKind::OpenChest=>10,RoomChoiceKind::Rest{..}=>11,RoomChoiceKind::Smith=>12,RoomChoiceKind::Recall=>13,RoomChoiceKind::Purge=>14,RoomChoiceKind::ShrineUpgrade=>15,RoomChoiceKind::ShrineRemove=>16,RoomChoiceKind::AncientElegance=>17,RoomChoiceKind::AncientSimplicity=>18,RoomChoiceKind::JoustMurderer=>19,RoomChoiceKind::JoustOwner=>20,RoomChoiceKind::JoustContinue=>21,RoomChoiceKind::JoustResult{..}=>22,RoomChoiceKind::FallingLose{..}=>23,RoomChoiceKind::TombGold=>24,RoomChoiceKind::TombOfferGold=>25,RoomChoiceKind::Leave=>26,RoomChoiceKind::Proceed=>27,RoomChoiceKind::Skip=>28,RoomChoiceKind::ConfirmSkip=>29,RoomChoiceKind::CancelSkip=>30,RoomChoiceKind::Act3Continue{..}=>31}}
pub(crate) fn reward(r:NeowReward)->i64{match r{NeowReward::ThreeCards=>1,NeowReward::RandomRare=>2,NeowReward::RemoveOne=>3,NeowReward::UpgradeOne=>4,NeowReward::TransformOne=>5,NeowReward::ThreeUncommonColorless=>6,NeowReward::ThreePotions=>7,NeowReward::CommonRelic=>8,NeowReward::MaxHp=>9,NeowReward::Lament=>10,NeowReward::Gold=>11,NeowReward::RareColorless=>12,NeowReward::RareRelic=>13,NeowReward::ThreeRareCards=>14,NeowReward::TransformTwo=>15,NeowReward::RemoveTwo=>16,NeowReward::Gold250=>17,NeowReward::MaxHpTwice=>18,NeowReward::BossSwap=>19}}
pub(crate) fn drawback(d:NeowDrawback)->i64{match d{NeowDrawback::MaxHpLoss=>1,NeowDrawback::NoGold=>2,NeowDrawback::RandomCurse=>3,NeowDrawback::CurrentHpLoss=>4}}
pub(crate) const POWERS:&[&str]=&["padding","strength","dexterity","weak","vulnerable","frail","artifact","ritual","curl_up","split","enrage","metallicize","mode_shift","sharp_hide","barricade","flight","hex","confused","minion","angry","entangled","painful_stabs","stasis","regeneration","thorns","explosive","life_link","regrow","fading","shifting","shackled","constricted","reactive","malleable","slow","intangible","regenerate","curiosity","unawakened","time_warp","draw_reduction","poison","strength_up"];
pub(crate) const INTENTS:&[&str]=&["padding","incantation","attack","chomp","thrash","bellow","bite","grow","spit_web","skull_bash","bull_rush","acid_tackle","acid_spit","acid_lick","acid_split","spike_tackle","spike_flame_tackle","spike_lick","spike_split","sticky","prep_slam","slam","split","sleep","siphon","stun","lag_attack","sentry_bolt","sentry_beam","charge_up","fierce_bash","vent_steam","whirlwind","defensive_mode","roll_attack","twin_slam","activate","divider","sear","tackle","inflame","inferno","sphere_activate","sphere_debuff","sphere_slam","sphere_harden","byrd_caw","byrd_peck","byrd_swoop","byrd_headbutt","byrd_stunned","byrd_go_airborne","chosen_poke","chosen_hex","chosen_debilitate","chosen_drain","chosen_zap","snecko_glare","snecko_bite","snecko_tail_whip","centurion_slash","centurion_protect","centurion_fury","mystic_attack","mystic_buff","mystic_heal","leader_rally","leader_stab","leader_encourage","gremlin_scratch","gremlin_puncture","gremlin_smash","gremlin_bash","gremlin_protect","gremlin_charge","gremlin_blast","gremlin_escape","slaver_stab","slaver_rake","slaver_scrape","slaver_entangle","taskmaster_whip","book_big","book_multi","champ_anger","champ_execute","champ_taunt","champ_stance","champ_gloat","champ_slap","champ_slash","collector_spawn","collector_ultimate","collector_revive","collector_rake","collector_buff","torch_attack","automaton_spawn","automaton_beam","automaton_stun","automaton_flail","automaton_boost","orb_stasis","orb_support","orb_beam","orb_claw","orb_laser","repulsor_attack","repulsor_daze","exploder_attack","exploder_explode","spiker_attack","spiker_buff","darkling_reincarnate","darkling_harden","darkling_nip","darkling_chomp","darkling_regrowing","growth_quick","growth_constrict","growth_smash","transient_attack","maw_roar","maw_nom","maw_drool","maw_slam","writhing_multi","writhing_block","writhing_debuff","writhing_parasite","writhing_big","head_count","head_time","head_glare","nemesis_tri","nemesis_scythe","nemesis_burn","repto_spawn","repto_snake","repto_bite","dagger_wound","dagger_explode","awa_rebirth","awa_slash","awa_echo","awa_soul","awa_sludge","awa_tackle","time_haste","time_reverb","time_slam","time_ripple","deca_beam","deca_square","donu_beam","donu_circle"];
pub(crate) fn named(table:&[&str],name:&str)->Result<i64,RunSessionError>{table.iter().position(|v|*v==name).filter(|i|*i>0).map(|i|i as i64).ok_or(RunSessionError::Input("unsupported public power or intent"))}
// Exact additional Act1 producer names. Append IDs; never alias a spelling to
// another intent. Transport vocabularies do not claim implementation coverage.
const ACT1_INTENTS:&[&str]=&["charging_up","beam","bolt","stunned"];
pub(crate) fn intent(name:&str)->Result<i64,RunSessionError>{
    INTENTS.iter().chain(ACT1_INTENTS).position(|v|*v==name).filter(|i|*i>0).map(|i|i as i64).ok_or(RunSessionError::Input("unsupported public intent"))
}
fn vocabularies()->Value{
    let padded=|names:Vec<&'static str>|std::iter::once("padding").chain(names).collect::<Vec<_>>();
    json!({"card":padded(CARDS.iter().map(|id|id.as_str()).collect()),"enemy":padded(ENEMIES.iter().map(|id|id.as_str()).collect()),"encounter":padded(ENCOUNTERS.iter().map(|id|id.as_str()).collect()),"relic":padded(RELICS.iter().map(|id|id.as_str()).collect()),"potion":padded(POTIONS.iter().map(|id|id.as_str()).collect()),"event":padded(EVENTS.iter().map(|id|id.as_str()).collect()),"power":POWERS,"intent":INTENTS.iter().chain(ACT1_INTENTS).copied().collect::<Vec<_>>(),"room":ROOMS,"choice":CHOICES,"character":["padding","ironclad","silent","defect","watcher"],"map_kind":["padding","monster","elite","shop","rest","question","treasure","boss","event"],"terminal":["ongoing","death","empty_deck","act3_no_keys","heart_victory"],"outcome":["ongoing","victory","death"],"profile":["padding","M4Act1Prefix","M5Act1Captured","M6Act2Prefix","M7Act2Captured","M8Act3Prefix","M9IroncladAct3"],"choice_status":["padding","offered","claimed","skipped","disabled"],"zone":["padding","hand","discard","exhaust","played_power","stasis","selection_only"],"row_group":["none","player","map_nodes","permanent_cards","combat_cards","enemies","potions","offers","selection"],"verb":["padding","move","choose","select","confirm","cancel","use_potion","discard_potion","play_card","combat_select_card","end_turn"],"card_type":["padding","attack","skill","power","status","curse"],"stance":["padding","neutral","wrath","calm"],"orb":["empty","lightning","frost"],"neow_reward":["padding","three_cards","random_rare","remove_one","upgrade_one","transform_one","three_uncommon_colorless","three_potions","common_relic","max_hp","lament","gold","rare_colorless","rare_relic","three_rare_cards","transform_two","remove_two","gold250","max_hp_twice","boss_swap"],"neow_drawback":["none","max_hp_loss","no_gold","random_curse","current_hp_loss"]})
}
