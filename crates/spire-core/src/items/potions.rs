use crate::{
    combat::{Actor, DamagePayload, Effect, EffectKind, EnemyHandle, Source},
    content::{Character, PotionId},
    run::RunError,
};

use super::{CombatItemContext, NativePotionUseContext, PotionRoomPhase, PotionUseEffects};

pub(super) fn use_potion(context: &CombatItemContext, slot: usize,
    target: Option<EnemyHandle>, facts: &NativePotionUseContext<'_>) -> Result<PotionUseEffects, RunError> {
    use_potion_inner(context, slot, target, facts, None)
}

pub(super) fn use_potion_for_character(context: &CombatItemContext, slot: usize,
    target: Option<EnemyHandle>, facts: &NativePotionUseContext<'_>, character: Character) -> Result<PotionUseEffects, RunError> {
    use_potion_inner(context, slot, target, facts, Some(character))
}

fn use_potion_inner(context: &CombatItemContext, slot: usize,
    target: Option<EnemyHandle>, facts: &NativePotionUseContext<'_>, character: Option<Character>) -> Result<PotionUseEffects, RunError> {
    let potion = context.potions.get(slot).copied().flatten().ok_or(RunError::InvalidAction)?;
    if context.m20_profile()&&character!=Some(Character::Defect){return Err(RunError::InvalidAction)}
    if matches!(potion,PotionId::PoisonPotion|PotionId::CunningPotion|PotionId::GhostInAJar)
        &&character.is_some_and(|c|c!=Character::Silent){return Err(RunError::InvalidAction)}
    if facts.we_meet_again { return Err(RunError::InvalidAction); }
    if facts.max_hp == 0 || facts.max_hp > 999 || facts.hp > facts.max_hp {
        return Err(RunError::InvalidAction);
    }
    if context.m21_profile(){
        if character!=Some(Character::Watcher)||target.is_some()||facts.phase!=PotionRoomPhase::Combat||facts.turn_ended||facts.live_targets.is_empty(){return Err(RunError::InvalidAction)}
        let potency=context.potion_potency(slot).ok_or(RunError::InvalidAction)?;
        use crate::combat::cards::watcher::WatcherEffect as W;
        let kind=match potion{
            PotionId::StancePotion=>EffectKind::Watcher(W::OpenStanceChoice),
            PotionId::Ambrosia=>EffectKind::Character(crate::combat::CharacterEffect::Stance(crate::combat::Stance::Divinity)),
            PotionId::BottledMiracle=>EffectKind::Watcher(W::CreateRequest{id:crate::content::CardId::Miracle,count:potency as u32}),
            PotionId::EnergyPotion=>EffectKind::Watcher(W::PotionEnergy{slot,amount:potency}),
            PotionId::BlockPotion=>EffectKind::GainBlock(potency),
            PotionId::DexterityPotion=>EffectKind::ApplyFullPower{id:crate::combat::FullPowerId("dexterity"),amount:potency},
            PotionId::AncientPotion=>EffectKind::ApplyFullPower{id:crate::combat::FullPowerId("artifact"),amount:potency},
            _=>return Err(RunError::InvalidAction),
        };
        let mut potions=context.potions.clone();potions[slot]=None;
        return Ok(PotionUseEffects{effects:vec![Effect{source:Source::Potion{slot,id:potion},target:Some(Actor::Player),kind}],potions})
    }
    if potion == PotionId::EntropicBrew {
        let character = character.ok_or(RunError::Unsupported("actual character required for Entropic Brew"))?;
        if !(context.m15_profile()&&character==Character::Silent||context.m20_profile()&&character==Character::Defect) || target.is_some()
            || facts.phase != PotionRoomPhase::Combat || facts.turn_ended
            || !context.valid_potion_capacity() {
            return Err(RunError::InvalidAction);
        }
        let mut potions = context.potions.clone();
        potions[slot] = None;
        return Ok(PotionUseEffects {
            effects: vec![Effect {
                source: Source::Potion { slot, id: potion },
                target: Some(Actor::Player),
                kind: EffectKind::Potion(crate::combat::PotionEffect::EntropicBrew {
                    character, count: context.potions.len(),
                }),
            }],
            potions,
        });
    }
    let target_required = matches!(potion, PotionId::FirePotion | PotionId::WeakPotion | PotionId::FearPotion | PotionId::PoisonPotion);
    if !target_required && target.is_some() {
        return Err(RunError::InvalidAction);
    }
    if potion==PotionId::SmokeBomb{
        if !context.shared_generated_profile()||facts.phase!=PotionRoomPhase::Combat||facts.turn_ended||facts.live_targets.is_empty(){return Err(RunError::InvalidAction)}
        let mut potions=context.potions.clone();potions[slot]=None;
        return Ok(PotionUseEffects{potions,effects:vec![Effect{source:Source::Potion{slot,id:potion},target:None,kind:EffectKind::SmokeEscape}]});
    }
    if matches!(potion,PotionId::AttackPotion|PotionId::SkillPotion|PotionId::PowerPotion|PotionId::ColorlessPotion){
        if !context.shared_generated_profile()||facts.phase!=PotionRoomPhase::Combat||facts.turn_ended||facts.live_targets.is_empty(){return Err(RunError::InvalidAction)}
        let pool=match potion{PotionId::AttackPotion=>crate::combat::GenerationPool::Attack,PotionId::SkillPotion=>crate::combat::GenerationPool::Skill,PotionId::PowerPotion=>crate::combat::GenerationPool::Power,_=>crate::combat::GenerationPool::Colorless};
        let copies=if context.owns_relic(crate::content::RelicId::SacredBark){2}else{1};
        let mut potions=context.potions.clone();potions[slot]=None;
        return Ok(PotionUseEffects{potions,effects:vec![Effect{source:Source::Potion{slot,id:potion},target:Some(Actor::Player),kind:crate::combat::offer_request(pool,potion!=PotionId::ColorlessPotion,copies)}]});
    }
    if matches!(potion,PotionId::FlexPotion|PotionId::SpeedPotion|PotionId::RegenPotion
        |PotionId::LiquidBronze|PotionId::EssenceOfSteel|PotionId::CultistPotion) {
        if !context.shared_generated_profile()||facts.phase!=PotionRoomPhase::Combat||facts.turn_ended||facts.live_targets.is_empty(){
            return Err(RunError::InvalidAction);
        }
        let factor=if context.owns_relic(crate::content::RelicId::SacredBark){2}else{1};
        let requests:Vec<(&'static str,i32)>=match potion {
            PotionId::FlexPotion=>vec![("strength",5),("lose_strength",5)],
            PotionId::SpeedPotion=>vec![("dexterity",5),("lose_dexterity",5)],
            PotionId::RegenPotion=>vec![("regeneration",5)],
            PotionId::LiquidBronze=>vec![("thorns",3)],
            PotionId::EssenceOfSteel=>vec![("plated_armor",4)],
            PotionId::CultistPotion=>vec![("ritual",1)],
            _=>unreachable!(),
        };
        let effects=requests.into_iter().map(|(id,amount)|Effect{
            source:Source::Potion{slot,id:potion},target:Some(Actor::Player),
            kind:EffectKind::ApplyFullPower{id:crate::combat::FullPowerId(id),amount:amount*factor},
        }).collect();
        let mut potions=context.potions.clone();potions[slot]=None;
        return Ok(PotionUseEffects{effects,potions});
    }
    if matches!(potion,PotionId::BlessingOfTheForge|PotionId::SneckoOil) {
        if !context.shared_generated_profile()||facts.phase!=PotionRoomPhase::Combat||facts.turn_ended||facts.live_targets.is_empty(){
            return Err(RunError::InvalidAction);
        }
        let source=Source::Potion{slot,id:potion};
        let mut effects=Vec::new();
        if potion==PotionId::SneckoOil {
            effects.push(Effect{source:source.clone(),target:Some(Actor::Player),kind:EffectKind::Draw(
                if context.owns_relic(crate::content::RelicId::SacredBark){10}else{5})});
        }
        effects.push(Effect{source,target:Some(Actor::Player),kind:EffectKind::Potion(
            if potion==PotionId::BlessingOfTheForge{crate::combat::PotionEffect::UpgradeHand}else{crate::combat::PotionEffect::RandomizeHandCosts})});
        let mut potions=context.potions.clone();potions[slot]=None;return Ok(PotionUseEffects{effects,potions});
    }
    if matches!(potion,PotionId::ExplosivePotion|PotionId::GamblersBrew|PotionId::LiquidMemories
        |PotionId::DuplicationPotion|PotionId::DistilledChaos) {
        if facts.phase!=PotionRoomPhase::Combat||facts.turn_ended||facts.live_targets.is_empty(){
            return Err(RunError::InvalidAction);
        }
        let bark=context.owns_relic(crate::content::RelicId::SacredBark);
        let source=Source::Potion{slot,id:potion};
        let effects=match potion {
            PotionId::ExplosivePotion=>facts.live_targets.iter().map(|owner|Effect{source:Source::None,
                target:Some(Actor::Enemy(owner.clone())),kind:EffectKind::Damage(
                    DamagePayload::Fixed(if bark{20}else{10}))}).collect(),
            PotionId::GamblersBrew=>vec![Effect{source,target:Some(Actor::Player),
                kind:EffectKind::Potion(crate::combat::PotionEffect::GamblersBrewSelectDiscard)}],
            PotionId::LiquidMemories=>vec![Effect{source,target:Some(Actor::Player),
                kind:EffectKind::Potion(crate::combat::PotionEffect::LiquidMemories{requested:
                    if bark{2}else{1}})}],
            PotionId::DuplicationPotion=>vec![Effect{source,target:Some(Actor::Player),
                kind:EffectKind::ApplyFullPower{id:crate::combat::FullPowerId("duplication"),amount:
                    if bark{2}else{1}}}],
            PotionId::DistilledChaos=>(0..if bark{6}else{3}).map(|_|Effect{source:source.clone(),target:None,
                kind:EffectKind::Potion(crate::combat::PotionEffect::DistilledChaosTopCard{target:None})}).collect(),
            _=>unreachable!("five shared potion action rows"),
        };
        let mut potions=context.potions.clone();potions[slot]=None;
        return Ok(PotionUseEffects{effects,potions});
    }
    if potion==PotionId::CunningPotion {
        if !context.m15_profile()||facts.phase!=PotionRoomPhase::Combat||facts.turn_ended||facts.live_targets.is_empty(){
            return Err(RunError::InvalidAction);
        }
        let count=if context.owns_relic(crate::content::RelicId::SacredBark){6}else{3};
        let effects=(0..count).map(|_|Effect{source:Source::Potion{slot,id:potion},target:Some(Actor::Player),
            kind:EffectKind::CreateCard{id:crate::content::CardId::Shiv,
                destination:crate::combat::CardDestination::Hand,upgrades:1}}).collect();
        let mut potions=context.potions.clone();potions[slot]=None;
        return Ok(PotionUseEffects{effects,potions});
    }
    let effect = match potion {
        PotionId::EssenceOfDarkness=>{
            if !context.m20_profile()||character!=Some(Character::Defect)||target.is_some()||facts.phase!=PotionRoomPhase::Combat||facts.turn_ended||facts.live_targets.is_empty(){return Err(RunError::InvalidAction)}
            Effect{source:Source::Potion{slot,id:potion},target:Some(Actor::Player),kind:EffectKind::Defect(crate::combat::cards::DefectEffect::EssenceOfDarkness{
                potency:if context.owns_relic(crate::content::RelicId::SacredBark){2}else{1}})}
        },
        PotionId::FocusPotion|PotionId::PotionOfCapacity=>{
            if character!=Some(Character::Defect)||target.is_some()||facts.phase!=PotionRoomPhase::Combat||facts.turn_ended||facts.live_targets.is_empty(){return Err(RunError::InvalidAction)}
            let amount=if context.owns_relic(crate::content::RelicId::SacredBark){4}else{2};
            Effect{source:Source::Potion{slot,id:potion},target:Some(Actor::Player),kind:EffectKind::Character(
                if potion==PotionId::FocusPotion{crate::combat::CharacterEffect::Focus(amount)}else{crate::combat::CharacterEffect::Slots(amount)})}
        },
        PotionId::Elixir => {
            if facts.phase != PotionRoomPhase::Combat || facts.turn_ended || facts.live_targets.is_empty() {
                return Err(RunError::InvalidAction);
            }
            Effect { source: Source::Potion { slot, id: potion }, target: Some(Actor::Player),
                kind: EffectKind::SelectHandCardsAfterDraw { required: 0,
                    disposition: crate::combat::SelectionDisposition::Exhaust,
                    empty: crate::combat::EmptySelection::Continue } }
        }
        PotionId::HeartOfIron => {
            if facts.phase != PotionRoomPhase::Combat || facts.turn_ended || facts.live_targets.is_empty() {
                return Err(RunError::InvalidAction);
            }
            Effect { source: Source::Potion { slot, id: potion }, target: Some(Actor::Player),
                kind: EffectKind::ApplyFullPower { id: crate::combat::FullPowerId("metallicize"), amount:
                    if context.owns_relic(crate::content::RelicId::SacredBark) { 12 } else { 6 } } }
        }
        PotionId::PoisonPotion => {
            if !context.m15_profile()||facts.phase!=PotionRoomPhase::Combat||facts.turn_ended||facts.live_targets.is_empty(){
                return Err(RunError::InvalidAction);
            }
            let enemy=target.ok_or(RunError::InvalidAction)?;
            if !facts.live_targets.contains(&enemy){return Err(RunError::InvalidAction)}
            Effect{source:Source::Potion{slot,id:potion},target:Some(Actor::Enemy(enemy)),
                kind:EffectKind::ApplyFullPower{id:crate::combat::FullPowerId("poison"),amount:
                    if context.owns_relic(crate::content::RelicId::SacredBark){12}else{6}}}
        }
        PotionId::GhostInAJar => {
            if !context.m15_profile()||facts.phase!=PotionRoomPhase::Combat||facts.turn_ended||facts.live_targets.is_empty(){
                return Err(RunError::InvalidAction);
            }
            Effect{source:Source::Potion{slot,id:potion},target:Some(Actor::Player),
                kind:EffectKind::ApplyFullPower{id:crate::combat::FullPowerId("intangible_player"),amount:
                    if context.owns_relic(crate::content::RelicId::SacredBark){2}else{1}}}
        }
        PotionId::FirePotion => {
            if facts.phase != PotionRoomPhase::Combat || facts.turn_ended {
                return Err(RunError::InvalidAction);
            }
            let target = target.ok_or(RunError::InvalidAction)?;
            if !facts.live_targets.contains(&target) { return Err(RunError::InvalidAction); }
            let amount = if context.owns_relic(crate::content::RelicId::SacredBark) { 40 } else { 20 };
            Effect { source: Source::Actor(Actor::Player), target: Some(Actor::Enemy(target)),
                kind: EffectKind::Damage(DamagePayload::Thorns(amount)) }
        }
        PotionId::BloodPotion => {
            if facts.phase == PotionRoomPhase::Combat && facts.turn_ended {
                return Err(RunError::InvalidAction);
            }
            let percent = if context.owns_relic(crate::content::RelicId::SacredBark) { 40.0_f32 } else { 20.0_f32 };
            let fraction = percent / 100.0_f32;
            let amount = ((facts.max_hp as f32) * fraction) as i32;
            Effect { source: Source::Actor(Actor::Player), target: Some(Actor::Player),
                kind: EffectKind::Heal(amount) }
        }
        PotionId::FruitJuice => {
            if facts.phase == PotionRoomPhase::Combat && facts.turn_ended {
                return Err(RunError::InvalidAction);
            }
            let amount: i32 = if context.owns_relic(crate::content::RelicId::SacredBark) { 10 } else { 5 };
            let hp_gain = u32::try_from(amount).map_err(|_| RunError::InvalidAction)?;
            if facts.max_hp.checked_add(hp_gain).map_or(true, |max_hp| max_hp > 999) {
                return Err(RunError::InvalidAction);
            }
            Effect { source: Source::Actor(Actor::Player), target: Some(Actor::Player),
                kind: EffectKind::IncreaseMaxHpThenHeal(amount) }
        }
        PotionId::BlockPotion | PotionId::EnergyPotion | PotionId::StrengthPotion
        | PotionId::SwiftPotion | PotionId::WeakPotion | PotionId::FearPotion
        | PotionId::DexterityPotion | PotionId::AncientPotion => {
            if facts.phase != PotionRoomPhase::Combat || facts.turn_ended || facts.live_targets.is_empty() {
                return Err(RunError::InvalidAction);
            }
            let bark = context.owns_relic(crate::content::RelicId::SacredBark);
            let actor_target = match potion {
                PotionId::WeakPotion | PotionId::FearPotion => {
                    let enemy = target.ok_or(RunError::InvalidAction)?;
                    if !facts.live_targets.contains(&enemy) { return Err(RunError::InvalidAction); }
                    Actor::Enemy(enemy)
                }
                _ => Actor::Player,
            };
            let kind = match potion {
                PotionId::BlockPotion => EffectKind::GainBlock(if bark { 24 } else { 12 }),
                PotionId::EnergyPotion => EffectKind::GainEnergy(if bark { 4 } else { 2 }),
                PotionId::StrengthPotion => EffectKind::ApplyFullPower {
                    id: crate::combat::FullPowerId("strength"), amount: if bark { 4 } else { 2 } },
                PotionId::DexterityPotion => EffectKind::ApplyFullPower {
                    id: crate::combat::FullPowerId("dexterity"), amount: if bark { 4 } else { 2 } },
                PotionId::AncientPotion => EffectKind::ApplyFullPower {
                    id: crate::combat::FullPowerId("artifact"), amount: if bark { 2 } else { 1 } },
                PotionId::SwiftPotion => EffectKind::Draw(if bark { 6 } else { 3 }),
                PotionId::WeakPotion => EffectKind::ApplyFullPower {
                    id: crate::combat::FullPowerId("weak"), amount: if bark { 6 } else { 3 } },
                PotionId::FearPotion => EffectKind::ApplyFullPower {
                    id: crate::combat::FullPowerId("vulnerable"), amount: if bark { 6 } else { 3 } },
                _ => unreachable!("finite combat potion arm"),
            };
            let source=if potion==PotionId::EnergyPotion&&context.m20_profile(){Source::Potion{slot,id:potion}}else{Source::Actor(Actor::Player)};
            Effect { source, target: Some(actor_target), kind }
        }
        PotionId::CunningPotion => unreachable!("Cunning Potion returns its three or six work items above"),
        PotionId::FairyPotion => return Err(RunError::InvalidAction),
        PotionId::BlessingOfTheForge | PotionId::SneckoOil | PotionId::StancePotion | PotionId::Ambrosia | PotionId::BottledMiracle | PotionId::Unknown(_) | PotionId::FlexPotion | PotionId::SpeedPotion | PotionId::RegenPotion
        | PotionId::LiquidBronze | PotionId::EssenceOfSteel | PotionId::CultistPotion
        | PotionId::ExplosivePotion | PotionId::AttackPotion | PotionId::SkillPotion | PotionId::PowerPotion
        | PotionId::ColorlessPotion | PotionId::GamblersBrew | PotionId::DuplicationPotion
        | PotionId::DistilledChaos | PotionId::LiquidMemories | PotionId::SmokeBomb | PotionId::EntropicBrew =>
            return Err(RunError::Unsupported("unimplemented potion effect")),
    };
    let mut potions = context.potions.clone();
    potions[slot] = None;
    Ok(PotionUseEffects { effects: vec![effect], potions })
}
